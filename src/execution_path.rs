use crate::graph::Graph;
use std::marker::Copy;
use std::clone::Clone;
use std::cmp::PartialEq;
use std::rc::Rc;
use std::cell::RefCell;

pub enum ArgVal {
    Waiting(RStag),
    Ready(u32),
    Imm(u32),
}

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

pub enum ExecResult {
    Arth(u32),
}

pub trait ExecPath: Graph {
    fn list_inst(&self) -> Vec<&'static str>;
    fn issue(&mut self, inst: &str, vals: Vec<ArgVal>) -> Result<RStag, ()>;
    fn next_cycle(&mut self);
    fn get_result(&mut self) -> Option<(RStag, ExecResult)>;
}

pub fn execution_path_factory(name: &str) -> Box<dyn ExecPath> {
    todo!();
}
