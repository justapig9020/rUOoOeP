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
        String::from("arith")
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
            InstFormat::create("muli")
                .add_syntax(TokenType::Writeback)
                .add_syntax(TokenType::Register)
                .add_syntax(TokenType::Immediate)
                .done(),
            InstFormat::create("mul")
                .add_syntax(TokenType::Writeback)
                .add_syntax(TokenType::Register)
                .add_syntax(TokenType::Register)
                .done(),
        ]
    }
    fn forward(&mut self, tag: RStag, val: u32) {
        let inst_src = tag.station();
        if self.name() == inst_src {
            let idx = tag.slot();
            self.station.sloved(idx);
        }
        self.station.forward(&tag, val);
    }
    fn try_issue(&mut self, inst: String, renamed_args: &[ArgState]) -> Result<RStag, ()> {
        let inst = ArithInst::new(inst, renamed_args).map_err(|_| ())?;
        self.station
            .insert(inst as Box<dyn RenamedInst>)
            .map(|idx| RStag::new(&self.name, idx))
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
    fn is_idle(&self) -> bool {
        self.station.occupied() == 0
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.name)?;

        let slots: Vec<String> = self
            .station
            .into_iter()
            .map(|slot| format!("{}", slot))
            .collect();
        writeln!(f, "{}", into_table("Reservation station", slots))?;
        let exec = if let Some(exec) = self.exec.as_ref() {
            exec.to_string()
        } else {
            String::new()
        };
        let table = into_table("Executing", vec![exec]);
        writeln!(f, "{table}")?;
        Ok(())
    }
}

impl Unit {
    pub fn new(index: usize) -> Self {
        Self {
            name: format!("arith{}", index),
            station: ReservationStation::new(2),
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
            let name = inst.command();
            let args = inst.arguments();
            let value_of = |idx: usize| -> Result<u32, String> {
                args.get(idx)
                    .ok_or_else(|| format!("There is no argument {}", idx))?
                    .val()
                    .ok_or_else(|| format!("Argument {} is not ready", idx))
            };
            let arg0 = value_of(0)?;
            let arg1 = value_of(1)?;
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
struct ArithInst {
    name: String,
    arg0: ArgState,
    arg1: ArgState,
}

impl Display for ArithInst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}, {}", self.name, self.arg0, self.arg1)
    }
}

impl ArithInst {
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

impl RenamedInst for ArithInst {
    fn command(&self) -> &str {
        &self.name
    }
    fn arguments(&self) -> Vec<ArgState> {
        vec![self.arg0.clone(), self.arg1.clone()]
    }
    fn is_ready(&self) -> bool {
        use ArgState::Ready;
        matches!(self.arg0, Ready(_)) && matches!(self.arg1, Ready(_))
    }
    fn forward(&mut self, tag: &RStag, val: u32) {
        self.arg0.forwarding(tag, val);
        self.arg1.forwarding(tag, val);
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
            "muli" | "mul" => (3, arg0 * arg1),
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
            let result = ExecResult::Arith(self.result);
            bus.set(tag, result)
        } else {
            self.cycle -= 1;
            false
        }
    }
}
