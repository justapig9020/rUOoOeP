use crate::execution_path::{ RStag, ExecPath, ArgVal, ExecResult};
use crate::decoder::{ InstFormat, InstFormatCreater , SyntaxType};
use crate::result_bus::ResultBus;

#[derive(Debug)]
pub struct Unit {
    name: String,
    station: RStation,
    exec: Option<ExecUnit>,
}

impl ExecPath for Unit {
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_func(&self) -> String {
        String::from("arth")
    }
    fn list_inst(&self) -> Vec<InstFormat> {
        vec![
            InstFormat::create("add")
                .add_syntax(SyntaxType::Register)
                .add_syntax(SyntaxType::Register)
                .add_syntax(SyntaxType::Register)
                .set_writeback(true)
                .done(),
            InstFormat::create("addi")
                .add_syntax(SyntaxType::Register)
                .add_syntax(SyntaxType::Register)
                .add_syntax(SyntaxType::Immediate)
                .set_writeback(true)
                .done(),]
    }
    fn forwarding(&mut self, tag: RStag, val: u32) {
        let station = tag.get_station();
        if self.get_name() == station {
            let idx = tag.get_slot();
            self.station.sloved(idx);
        }
        self.station.forwarding(&tag, val);
    }
    fn issue(&mut self, inst: String, vals:&[ArgVal]) -> Result<RStag, ()> {
        if let Some(idx) = self.station.insert(inst, vals) {
            let tag = RStag::new(&self.name, idx);
            Ok(tag)
        } else {
            Err(())
        }
    }
    fn next_cycle(&mut self, bus: &mut ResultBus) {
        if let Some(unit) = self.exec.as_mut() {
            let done = unit.next_cycle(bus);
            if done {
                // execution done
                self.exec = None;
            }
        } else {
            if let Some((idx, inst)) = self.station.ready() {
                let name = inst.inst;
                let arg0 = inst.arg0.val().unwrap_or(0);
                let arg1 = inst.arg1.val().unwrap_or(0);
                let tag = RStag::new(&self.get_name(), idx);
                self.exec = Some(ExecUnit::exec(tag, name, arg0, arg1));
            }
        }
    }
}

impl Unit {
    pub fn new() -> Self {
        static mut cnt: usize = 0;
        // Safety: the cnt will only be used in this function
        let idx = unsafe {
            cnt += 1;
            cnt - 1
        };
        Self {
            name: format!("arth{}", idx),
            station: RStation::new(5),
            exec: None,
        }
    }
}

#[derive(Debug)]
struct RStation {
    slots: Vec<Option<ArthInst>>,
    /// Due to the instruction can not go into execution in the cycle it just
    /// commit.
    /// Add the indicator to indicate the index that is just issued in this cycle.
    just_issued: Option<usize>,
}

impl RStation {
    fn new(size: usize) -> Self {
        let mut slots = Vec::with_capacity(size);
        for _ in 0..size {
            slots.push(None);
        }
        Self {
            slots,
            just_issued: None,
        }
    }
    /// Insert a instruction to reservation station.
    /// Retuen Some(slot number) is the insert success.
    /// Return None if there is no empty slot.
    fn insert(&mut self, inst: String, args: &[ArgVal]) -> Option<usize> {
        if args.len() != 2 {
            return None;
        }
        for (idx, slot) in self.slots.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(ArthInst {
                    inst,
                    arg0: args[0].clone(),
                    arg1: args[1].clone(),
                });
                self.just_issued = Some(idx);
                return Some(idx);
            }
        }
        None
    }
    /// Find a ready instruction.
    /// If found, remove it from reservation station and return it.
    /// Otherwise, return None.
    fn ready(&mut self) -> Option<(usize, ArthInst)> {
        let skip = if let Some(idx) = self.just_issued {
            idx
        } else {
            // This index will never reached
            self.slots.len()
        };
        self.just_issued = None;
        for (idx, slot) in self.slots.iter_mut().enumerate() {
            if idx == skip {
                continue;
            }
            if let Some(inst) = slot {
                if inst.is_ready() {
                    return Some((idx, inst.clone()));
                }
            }
        }
        None
    }
    /// The slot's instruction is solved, remove it.
    fn sloved(&mut self, idx: usize) {
        if idx < self.slots.len() {
            self.slots[idx] = None;
        }
    }
    fn forwarding(&mut self, tag: &RStag, val: u32) {
        for slot in self.slots.iter_mut() {
            if let Some(slot) = slot {
                slot.forwarding(tag, val);
            }
        }
    }
}

#[derive(Debug, Clone)]
struct ArthInst {
    inst: String,
    arg0: ArgVal,
    arg1: ArgVal,
}

impl ArthInst {
    /// An instruction is ready if it's not waiting result of another instruction.
    fn is_ready(&self) -> bool {
        if let ArgVal::Ready(_) = self.arg0 {
            if let ArgVal::Ready(_) = self.arg1 {
                return true;
            }
        }
        false
    }
    fn forwarding(&mut self, tag: &RStag, val: u32) {
        if let ArgVal::Waiting(wait) = self.arg0.clone() {
            if wait == *tag {
                self.arg0 = ArgVal::Ready(val);
            }
        }
        if let ArgVal::Waiting(wait) = self.arg1.clone() {
            if wait == *tag {
                self.arg1 = ArgVal::Ready(val);
            }
        }
    }
}

#[derive(Debug)]
struct ExecUnit {
    cycle: usize,
    tag: RStag,
    result: u32,
}

impl ExecUnit {
    fn exec(tag: RStag, inst: String, arg0: u32, arg1: u32) -> Self {
        let (cycle, result) = match inst.as_str() {
            "add" | "addi" => (1, arg0 + arg1),
            _ => (0, 0),
        };
        Self {
            cycle,
            tag,
            result,
        }
    }
    fn next_cycle(&mut self, bus: &mut ResultBus) -> bool {
        if self.cycle == 0 {
            let tag = self.tag.clone();
            let result = ExecResult::Arth(self.result);
            if bus.set(tag, result) {
                true
            } else {
                false
            }
        } else {
            self.cycle -= 1;
            false
        }
    }
}
