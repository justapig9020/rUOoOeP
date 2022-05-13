use crate::core::processor::Processor;
use std::fmt;

pub struct Machine {
    core: Processor,
    iram: Vec<String>,
    dram: Vec<u8>,
}

impl fmt::Display for Machine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.core)
    }
}

impl Machine {
    pub fn new(core: Processor, insts: Vec<String>, ram_size: usize) -> Self {
        Self {
            core,
            iram: insts,
            dram: vec![0; ram_size],
        }
    }
    pub fn next_cycle(&mut self) -> Result<(), String> {
        let p = &mut self.core;
        let line = p.fetch_address();
        let inst = self
            .iram
            .get(line)
            .ok_or(format!("Inst addr: {} out of bound", line))?;
        p.next_cycle(inst)?;
        Ok(())
    }
    pub fn into_processor(self) -> Processor {
        self.core
    }
}
