use crate::core::processor::{BusAccess, Processor};
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
        if let Some(access) = p.bus_access() {
            let response = self.bus_access(access)?;
            self.core.resolve_access(response);
        }
        Ok(())
    }
    fn bus_access(&mut self, access: BusAccess) -> Result<u8, String> {
        match access {
            BusAccess::Read(addr) => self
                .dram
                .get(addr as usize)
                .map(|v| *v)
                .ok_or(format!("Memory addr: {} out of bound", addr)),
            BusAccess::Write(addr, val) => {
                let cell = self
                    .dram
                    .get_mut(addr as usize)
                    .ok_or(format!("Memory addr: {} out of bound", addr))?;
                *cell = val;
                Ok(0)
            }
        }
    }
    pub fn into_processor(self) -> Processor {
        self.core
    }
}
