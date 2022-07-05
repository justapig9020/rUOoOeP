use std::fmt::Display;

use crate::core::execution_path::BusAccess;
use crate::core::execution_path::BusAccessRequst;
use crate::core::execution_path::BusAccessResponse;
use crate::core::execution_path::BusAccessResult;

const ACCESS_LATENCY: usize = 5;

pub struct Dram {
    memory: Vec<u8>,
    /// (remaining cycles, request handler)
    request: Option<(usize, BusAccessRequst)>,
}

impl Display for Dram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.request)
    }
}

impl Dram {
    pub fn new(size: usize) -> Self {
        Self {
            memory: vec![0; size],
            request: None,
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
    pub fn is_idle(&self) -> bool {
        self.request.is_none()
    }
    pub fn access(&mut self, request: BusAccessRequst) -> Result<(), String> {
        if self.request.is_some() {
            let msg = String::from("Memory is busy");
            return Err(msg);
        }
        self.request = Some((ACCESS_LATENCY, request));
        Ok(())
    }
    pub fn next_cycle(&mut self) -> Option<BusAccessResponse> {
        let (remain_cycle, request) = self.request.take()?;

        if remain_cycle > 0 {
            self.request = Some((remain_cycle - 1, request));
            return None;
        }
        let result = match request.request() {
            BusAccess::Load(base, len) => {
                let base = *base as usize;
                let len = *len as usize;
                self.read(base, len).map(BusAccessResult::Load)
            }
            BusAccess::Store(base, data) => self
                .write(*base as usize, data)
                .map(|_| BusAccessResult::Store),
        };

        Some(request.into_respose(result))
    }
    /// Check wheither the address is in the bound of memory
    pub fn bound_check(&self, address: usize) -> Result<(), String> {
        if self.memory.len() <= address {
            let msg = format!("DRAM: address {} out of bound", address);
            return Err(msg);
        }
        Ok(())
    }
    /// Consume the DRAM and return raw data inside
    pub fn into_raw_data(self) -> Vec<u8> {
        self.memory
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
