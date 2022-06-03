use crate::core::execution_path::BusAccess;
use crate::core::execution_path::BusAccessRequst;
use crate::core::execution_path::BusAccessResponse;
use crate::core::processor::Processor;
use std::fmt;

struct Dram {
    memory: Vec<u8>,
}

impl Dram {
    fn new(size: usize) -> Self {
        Self {
            memory: vec![0; size],
        }
    }
    /// Read len bytes from base adddress
    fn read(&self, base: usize, len: usize) -> Result<Vec<u8>, String> {
        let fin = base + len - 1;
        let memory = &self.memory;
        self.bound_check(fin)?;
        Ok(memory[base..=fin].to_vec())
    }
    /// Write data to base address
    fn write(&mut self, base: usize, data: &[u8]) -> Result<(), String> {
        let fin = base + data.len() - 1;
        self.bound_check(fin)?;
        self.memory.splice(base..=fin, data.to_vec());
        Ok(())
    }
    /// Check wheither the address is in the bound of memory
    fn bound_check(&self, address: usize) -> Result<(), String> {
        if self.memory.len() <= address {
            let msg = format!("DRAM: address {} out of bound", address);
            return Err(msg);
        }
        Ok(())
    }
}

#[cfg(test)]
mod dram {
    use super::*;
    #[test]
    fn write_in_bound() {
        let mut dram = Dram::new(5);
        let base = 2;
        let data = [1, 2, 3];
        let expect = [0, 0, 1, 2, 3];
        dram.write(base, &data).unwrap();
        assert_eq!(dram.memory, expect)
    }
    #[test]
    fn read_in_bound() {
        let len = 0x10;
        let mut dram = Dram::new(len);
        let expect: Vec<u8> = (0u8..len as u8).collect();
        for i in 0..len {
            dram.memory[i] = i as u8;
        }
        for i in (0..len).step_by(4) {
            let expect_slice = &expect[i..i + 4];
            let read = dram.read(i, 4).unwrap();
            assert_eq!(expect_slice, read);
        }
    }
}

pub struct Machine {
    core: Processor,
    iram: Vec<String>,
    dram: Dram,
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
            dram: Dram::new(ram_size),
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
            let response = self.bus_access(access);
            self.core.resolve_access(response)?;
        }
        Ok(())
    }
    fn bus_access(&mut self, access: BusAccessRequst) -> BusAccessResponse {
        let result = match access.request() {
            BusAccess::Read(base, len) => {
                let base = *base as usize;
                let len = *len as usize;
                self.dram.read(base, len)
            }
            BusAccess::Write(base, data) => self.dram.write(*base as usize, data).map(|_| vec![]),
        };
        access.into_respose(result)
    }
    pub fn into_processor(self) -> Processor {
        self.core
    }
}
