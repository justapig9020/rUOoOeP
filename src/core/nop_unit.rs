use std::fmt::Display;

use super::decoder::InstFormat;
use super::execution_path::{ArgState, ExecPath, RStag};
use super::result_bus::ResultBus;

const FUNC: &str = "nop";
const NAME: &str = "nop1";
#[derive(Debug)]
pub struct Unit {}

impl Unit {
    pub fn new() -> Self {
        Self {}
    }
}

impl ExecPath for Unit {
    fn name(&self) -> String {
        NAME.to_string()
    }
    /// Return name of class of fucntional unit.
    fn function(&self) -> String {
        FUNC.to_string()
    }
    /// List all instructions that implemented by the path.
    fn list_insts(&self) -> Vec<InstFormat> {
        vec![InstFormat::create("nop").done()]
    }
    /// Forward result to reservation station to resolve dependency.
    fn forward(&mut self, _tag: RStag, _val: u32) {}
    /// Issue a instruction to the execution path.
    /// On success, [Ok] with tag of issued reservation station returned.
    /// Otherwise, [Err] returned.
    fn try_issue(&mut self, _inst: String, _vals: &[ArgState]) -> Result<RStag, ()> {
        Ok(RStag::new(NAME, 0))
    }
    fn next_cycle(&mut self, _bus: &mut ResultBus) -> Result<(), String> {
        Ok(())
    }
    fn pending(&self) -> usize {
        0
    }
    fn is_idle(&self) -> bool {
        true
    }
}

impl Display for Unit {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
