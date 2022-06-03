use super::decoder::InstFormat;
use super::result_bus::ResultBus;
use std::clone::Clone;
use std::cmp::PartialEq;
use std::fmt::{self, Debug, Display};

/// State of argument of reservation stations
/// There are two states
/// 1. Waiting(tag): waiting for reault of `tag` to resolve dependency.
/// 2. Ready(value): all dependencies have been resolve and ready to go.
#[derive(Debug, Clone)]
pub enum ArgState {
    Waiting(RStag),
    Ready(u32),
}

impl Display for ArgState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let content = match self {
            ArgState::Waiting(tag) => {
                format!("{}", tag)
            }
            ArgState::Ready(val) => val.to_string(),
        };
        write!(f, "{}", content)
    }
}

impl ArgState {
    /// If argument is ready return value of the argument.
    /// Otherwise, return None.
    pub fn val(&self) -> Option<u32> {
        match self {
            ArgState::Waiting(_) => None,
            ArgState::Ready(val) => Some(*val),
        }
    }
}

/// Tag of Reservation station and slot.
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
        self.name == other.name && self.slot == other.slot
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
    /// Return the name of execute path of the station.
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
    fn name(&self) -> String;
    /// Return name of class of fucntional unit.
    fn function(&self) -> String;
    /// List all instructions that implemented by the path.
    fn list_insts(&self) -> Vec<InstFormat>;
    /// Forward result to reservation station to resolve dependency.
    fn forward(&mut self, tag: RStag, val: u32);
    /// Issue a instruction to the execution path.
    /// On success, [Ok] with tag of issued reservation station returned.
    /// Otherwise, [Err] returned.
    fn try_issue(&mut self, inst: String, vals: &[ArgState]) -> Result<RStag, ()>;
    fn next_cycle(&mut self, bus: &mut ResultBus) -> Result<(), String>;
    /// Return pending instruction count
    fn pending(&self) -> usize;
    fn dump(&self) -> String;
}

/// Bus access command
#[derive(Debug)]
pub enum BusAccess {
    /// Read(base address, length)
    Read(u32, usize),
    /// Write(base address, data string)
    Write(u32, Vec<u8>),
}

/// Handler of a Bus access
/// Each bus access request containted a handler. The handler will lead the corresponding response to correct execution path
#[derive(Debug)]
struct BusAccessHandler {
    path: String,
}

impl BusAccessHandler {
    fn paht_name(&self) -> String {
        self.path.clone()
    }
}

#[derive(Debug)]
pub struct BusAccessRequst {
    access: BusAccess,
    handler: BusAccessHandler,
}

impl BusAccessRequst {
    /// Get access command from the request
    pub fn request(&self) -> &BusAccess {
        &self.access
    }
    /// Submit a result and consume the BusAccess Request then construct corresponding BusAccessResponse
    pub fn into_respose(self, result: Result<Vec<u8>, String>) -> BusAccessResponse {
        BusAccessResponse {
            result,
            handler: self.handler,
        }
    }
}

#[derive(Debug)]
pub struct BusAccessResponse {
    result: Result<Vec<u8>, String>,
    handler: BusAccessHandler,
}

impl BusAccessResponse {
    pub fn path_name(&self) -> String {
        self.handler.paht_name()
    }
}

pub trait AccessPath: ExecPath {
    fn request(&mut self) -> Option<BusAccessRequst>;
    fn response(&mut self, result: BusAccessResponse);
}
