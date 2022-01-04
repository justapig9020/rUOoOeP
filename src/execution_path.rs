use crate::arthmatic_unit;
use std::clone::Clone;
use std::cmp::PartialEq;
use crate::decoder::InstFormat;
use crate::result_bus::ResultBus;
use std::fmt::{self, Debug, Display};

#[derive(Debug, Clone)]
pub enum ArgVal {
    Waiting(RStag),
    Ready(u32),
}

impl Display for ArgVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let content = match self {
            ArgVal::Waiting(tag) => {
                format!("{}", tag)
            },
            ArgVal::Ready(val) => {
                val.to_string()
            }
        };
        write!(f, "{}", content)
    }
}

impl ArgVal {
    pub fn val(&self) -> Option<u32> {
        match self {
            ArgVal::Waiting(_) => None,
            ArgVal::Ready(val) => Some(*val),
        }
    }
}

#[derive(Debug)]
pub struct RStag {
    name: String,
    slot: usize,
}

impl Display for RStag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.station(), self.slot())
    }
}
impl PartialEq for RStag {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name &&
        self.slot == other.slot
    }
}

impl Clone for RStag {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            slot: self.slot,
        }
    }
}

impl RStag {
    pub fn new(name: &str, slot: usize) -> Self {
        Self {
            name: name.to_string(),
            slot,
        }
    }
    pub fn station(&self) -> String {
        self.name.clone()
    }
    pub fn slot(&self) -> usize {
        self.slot
    }
}

#[derive(Debug)]
pub enum ExecResult {
    Arth(u32),
}

impl ExecResult {
    pub fn val(&self) -> u32 {
        match self {
            ExecResult::Arth(val) => *val,
        }
    }
}

pub trait ExecPath: Debug {
    fn get_name(&self) -> String;
    fn get_func(&self) -> String;
    fn list_inst(&self) -> Vec<InstFormat>;
    fn forwarding(&mut self, tag: RStag, val: u32);
    fn issue(&mut self, inst: String, vals:&[ArgVal]) -> Result<RStag, ()>;
    fn next_cycle(&mut self, bus: &mut ResultBus);
    fn dump(&self) -> String;
}

pub fn execution_path_factory(func: &str) -> Result<Box<dyn ExecPath>, String> {
    match func {
        "arth" => {
            let unit = arthmatic_unit::Unit::new();
            let unit = Box::new(unit) as Box<dyn ExecPath>;
            Ok(unit)
        }
        _ => {
            let msg = format!("Not support function unit {}", func);
            Err(msg)
        }
    }
}
