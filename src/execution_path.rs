use crate::arthmatic_unit;
use std::clone::Clone;
use std::cmp::PartialEq;
use crate::decoder::InstFormat;
use crate::result_bus::ResultBus;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum ArgVal {
    Waiting(RStag),
    Ready(u32),
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
