use crate::execution_path::{ RStag, ExecPath, ArgVal };
use crate::decoder::{ InstFormat, InstFormatCreater , SyntaxType};
use crate::result_bus::ResultBus;

#[derive(Debug)]
pub struct Unit {
    name: String,
    station: RStation,
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
        }
    }
}

#[derive(Debug)]
struct RStation {
    slots: Vec<Option<ArthInst>>,
}

impl RStation {
    fn new(size: usize) -> Self {
        let mut slots = Vec::with_capacity(size);
        for _ in 0..size {
            slots.push(None);
        }
        Self {
            slots,
        }
    }
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
                return Some(idx);
            }
        }
        None
    }
}

#[derive(Debug)]
struct ArthInst {
    inst: String,
    arg0: ArgVal,
    arg1: ArgVal,
}
