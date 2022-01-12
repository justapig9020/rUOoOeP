use crate::decoder::InstFormat;
use crate::execution_path::{ArgState, ExecPath, RStag};
use crate::result_bus::ResultBus;

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
    /// Return pending instruction count
    fn pending(&self) -> usize {
        1
    }
    fn dump(&self) -> String {
        String::new()
    }
}
