use crate::core::decoder::{InstFormat, TokenType};
use crate::core::execution_path::{ArgState, ExecPath, ExecResult, RStag};
use crate::core::result_bus::ResultBus;

use crate::display::into_table;

use super::reservation_station::*;
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
        let inst = ArthInst::new(inst, renamed_args).map_err(|_| ())?;
        self.station
            .insert(inst as Box<dyn RenamedInst>)
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
        let slots: Vec<String> = self.station.dump();
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
    pub fn new(index: usize) -> Self {
        Self {
            name: format!("arth{}", index),
            station: ReservationStation::new(5),
            exec: None,
        }
    }
    /// Execute instruction in given slot.
    /// On failed, error message returned.
    fn execute(&mut self, slot_id: usize) -> Result<(), String> {
        let slot = self
            .station
            .get_slot(slot_id)
            .ok_or(format!("Slot {} not exist", slot_id))?;
        if let SlotState::Pending(inst) = slot {
            let name = inst.name();
            let args = inst.arguments();
            let arg0 = args
                .get(0)
                .ok_or("There is no argument 0")?
                .val()
                .ok_or("Argument 0 is not ready".to_string())?;
            let arg1 = args
                .get(1)
                .ok_or("There is no argument 0")?
                .val()
                .ok_or("Argument 1 is not ready".to_string())?;
            let tag = RStag::new(&self.name(), slot_id);
            self.exec = Some(ExecUnit::exec(tag, name.to_string(), arg0, arg1));
            self.station.start_execute(slot_id)?;
            Ok(())
        } else {
            Err(format!("Slot {} is not pending", slot_id))
        }
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
    fn new(name: String, renamed_args: &[ArgState]) -> Result<Box<Self>, String> {
        if renamed_args.len() != 2 {
            Err(format!("Expect 2 arguments, {} got", renamed_args.len()))
        } else {
            Ok(Box::new(Self {
                name,
                arg0: renamed_args[0].clone(),
                arg1: renamed_args[1].clone(),
            }))
        }
    }
}

impl RenamedInst for ArthInst {
    fn name(&self) -> &str {
        &self.name
    }
    fn arguments(&self) -> Vec<ArgState> {
        vec![self.arg0.clone(), self.arg1.clone()]
    }
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
