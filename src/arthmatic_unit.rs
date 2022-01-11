use crate::decoder::{InstFormat, TokenType};
use crate::display::into_table;
use crate::execution_path::{ArgState, ExecPath, ExecResult, RStag};
use crate::result_bus::ResultBus;
use std::fmt::{self, Display};

#[derive(Debug)]
pub struct Unit {
    name: String,
    station: ReservationStation,
    exec: Option<ExecUnit>,
}

impl ExecPath for Unit {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn function(&self) -> String {
        String::from("arth")
    }
    fn list_insts(&self) -> Vec<InstFormat> {
        vec![
            InstFormat::create("add")
                .add_syntax(TokenType::Writeback)
                .add_syntax(TokenType::Register)
                .add_syntax(TokenType::Register)
                .done(),
            InstFormat::create("addi")
                .add_syntax(TokenType::Writeback)
                .add_syntax(TokenType::Register)
                .add_syntax(TokenType::Immediate)
                .done(),
        ]
    }
    fn forward(&mut self, tag: RStag, val: u32) {
        let inst_from = tag.station();
        if self.name() == inst_from {
            let idx = tag.slot();
            self.station.sloved(idx);
        }
        self.station.forwarding(&tag, val);
    }
    fn try_issue(&mut self, inst: String, renamed_args: &[ArgState]) -> Result<RStag, ()> {
        self.station
            .insert(inst, renamed_args)
            .map(|idx| {
                let tag = RStag::new(&self.name, idx);
                tag
            })
            .ok_or(())
    }
    fn next_cycle(&mut self, bus: &mut ResultBus) -> Result<(), String> {
        if let Some(unit) = self.exec.as_mut() {
            let done = unit.next_cycle(bus);
            if done {
                // execution done
                self.exec = None;
            }
        }
        if self.exec.is_none() {
            if let Some(id) = self.station.ready() {
                self.execute(id)?;
            }
        }
        Ok(())
    }
    fn pending(&self) -> usize {
        self.station.pending()
    }
    fn dump(&self) -> String {
        let mut info = format!("{}\n", self.name);
        let slots: Vec<String> = self
            .station
            .slots
            .iter()
            .map(|slot| match slot {
                SlotState::Empty => String::from("Empty"),
                SlotState::Executing(inst) => format!("Exe({})", inst),
                SlotState::Pending(inst) => format!("Pend({})", inst),
            })
            .collect();
        info.push_str(&into_table("Reservation station", slots));
        if let Some(exec) = self.exec.as_ref() {
            let exec = exec.to_string();
            let table = into_table("Executing", vec![exec]);
            info.push_str(&table);
        }
        info
    }
}

impl Unit {
    pub fn new() -> Self {
        static mut CNT: usize = 0;
        // Safety: the cnt will only be used in this function
        let idx = unsafe {
            CNT += 1;
            CNT - 1
        };
        Self {
            name: format!("arth{}", idx),
            station: ReservationStation::new(5),
            exec: None,
        }
    }
    /// Execute instruction in given slot.
    /// On failed, error message returned.
    fn execute(&mut self, slot_id: usize) -> Result<(), String> {
        let slot = self
            .station
            .slots
            .get(slot_id)
            .ok_or(format!("Slot {} not exist", slot_id))?;
        if let SlotState::Pending(inst) = slot {
            let name = inst.name.clone();
            let arg0 = inst
                .arg0
                .val()
                .ok_or("Argument 0 is not ready".to_string())?;
            let arg1 = inst
                .arg1
                .val()
                .ok_or("Argument 1 is not ready".to_string())?;
            let tag = RStag::new(&self.name(), slot_id);
            self.exec = Some(ExecUnit::exec(tag, name, arg0, arg1));
            self.station.slots[slot_id] = SlotState::Executing(inst.clone());
            Ok(())
        } else {
            Err(format!("Slot {} is not pending", slot_id))
        }
    }
}

#[derive(Debug)]
enum SlotState {
    Empty,
    Pending(ArthInst),
    Executing(ArthInst),
}

impl SlotState {
    fn is_empty(&self) -> bool {
        if let SlotState::Empty = self {
            true
        } else {
            false
        }
    }
}

#[derive(Debug)]
struct ReservationStation {
    slots: Vec<SlotState>,
}

#[cfg(test)]
mod resrvation_station {
    use super::*;

    fn new_instruction() -> (String, &'static [ArgState]) {
        (
            String::from("inst"),
            &[ArgState::Ready(1), ArgState::Ready(2)],
        )
    }
    #[test]
    fn pending() {
        let size = 10;
        let mut station = ReservationStation::new(size);
        let inst_cnt = 5;
        for _ in 0..inst_cnt {
            let (inst, args) = new_instruction();
            station.insert(inst, args);
        }
        assert_eq!(inst_cnt, station.pending());
    }
}

impl ReservationStation {
    fn new(size: usize) -> Self {
        Self {
            slots: (0..size).map(|_| SlotState::Empty).collect(),
        }
    }
    /// Insert a instruction to reservation station.
    /// Retuen Some(slot number) is the insert success.
    /// Return None if there is no empty slot.
    fn insert(&mut self, inst: String, renamed_args: &[ArgState]) -> Option<usize> {
        if renamed_args.len() != 2 {
            return None;
        }
        for (idx, slot) in self.slots.iter_mut().enumerate() {
            if slot.is_empty() {
                *slot = SlotState::Pending(ArthInst {
                    name: inst,
                    arg0: renamed_args[0].clone(),
                    arg1: renamed_args[1].clone(),
                });
                return Some(idx);
            }
        }
        None
    }
    /// Find a ready instruction.
    /// If found, its index returned.
    /// Otherwise, return None.
    fn ready(&mut self) -> Option<usize> {
        for (idx, slot) in self.slots.iter_mut().enumerate() {
            if let SlotState::Pending(inst) = slot {
                if inst.is_ready() {
                    return Some(idx);
                }
            }
        }
        None
    }
    /// The slot's instruction is solved, remove it.
    fn sloved(&mut self, idx: usize) {
        if idx < self.slots.len() {
            self.slots[idx] = SlotState::Empty;
        }
    }
    fn forwarding(&mut self, tag: &RStag, val: u32) {
        for slot in self.slots.iter_mut() {
            if let SlotState::Pending(slot) = slot {
                slot.forwarding(tag, val);
            }
        }
    }
    fn pending(&self) -> usize {
        let empty: usize = self.slots.iter().map(|a| a.is_empty() as usize).sum();
        self.slots.len() - empty
    }
}

#[derive(Debug, Clone)]
struct ArthInst {
    name: String,
    arg0: ArgState,
    arg1: ArgState,
}

impl Display for ArthInst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}, {}", self.name, self.arg0, self.arg1)
    }
}

impl ArthInst {
    /// An instruction is ready if it's not waiting result of another instruction.
    fn is_ready(&self) -> bool {
        use ArgState::Ready;
        matches!(self.arg0, Ready(_)) && matches!(self.arg1, Ready(_))
    }
    fn forwarding(&mut self, tag: &RStag, val: u32) {
        if let ArgState::Waiting(wait) = self.arg0.clone() {
            if wait == *tag {
                self.arg0 = ArgState::Ready(val);
            }
        }
        if let ArgState::Waiting(wait) = self.arg1.clone() {
            if wait == *tag {
                self.arg1 = ArgState::Ready(val);
            }
        }
    }
}

#[derive(Debug)]
struct ExecUnit {
    instruction: String,
    cycle: usize,
    tag: RStag,
    result: u32,
}

impl Display for ExecUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: Remain {} cycles, Destination: {}",
            self.instruction, self.cycle, self.tag
        )
    }
}

impl ExecUnit {
    fn exec(tag: RStag, inst: String, arg0: u32, arg1: u32) -> Self {
        let (cycle, result) = match inst.as_str() {
            "add" | "addi" => (1, arg0 + arg1),
            _ => (0, 0),
        };
        Self {
            instruction: inst,
            cycle,
            tag,
            result,
        }
    }
    fn next_cycle(&mut self, bus: &mut ResultBus) -> bool {
        if self.cycle == 0 {
            let tag = self.tag.clone();
            let result = ExecResult::Arth(self.result);
            bus.set(tag, result)
        } else {
            self.cycle -= 1;
            false
        }
    }
}
