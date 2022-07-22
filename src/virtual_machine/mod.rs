use crate::core::processor::Processor;
use crate::memory_bus::dram::Dram;
use std::fmt;

pub struct Machine {
    core: Processor,
    iram: Vec<String>,
    dram: Dram,
}

impl fmt::Display for Machine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.core)
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
    /// Execute next machine cycle of virtual machine
    pub fn next_cycle(&mut self) -> Result<(), String> {
        let line = self.core.fetch_address();
        let inst = self
            .iram
            .get(line)
            .ok_or(format!("Inst addr: {} out of bound", line))?;
        let inst = inst.clone();
        self.do_next_cycle(&inst)
    }
    fn do_next_cycle(&mut self, inst: &str) -> Result<(), String> {
        let p = &mut self.core;
        p.next_cycle(inst)?;
        if self.dram.is_idle() {
            if let Some(request) = p.bus_access() {
                self.dram.access(request)?;
            }
        }
        if let Some(response) = self.dram.next_cycle() {
            p.resolve_access(response)?;
        }
        Ok(())
    }
    pub fn next_flush_cycle(&mut self) -> Result<(), String> {
        if self.core.is_idle() {
            let msg = String::from("This machine is idle");
            return Err(msg);
        }
        self.do_next_cycle("nop")
    }
    /// Splite virtual machine into components
    pub fn splite(self) -> (Processor, Vec<u8>) {
        (self.core, self.dram.into_raw_data())
    }
}

#[cfg(test)]
mod vm {
    use crate::core::execution_path::ArgState;
    use crate::functional_units::factory::Factory;
    use crate::functional_units::factory::Function;
    use crate::functional_units::factory::MemFunction;
    use crate::util::raw_to_u32_big_endian;

    use super::*;

    fn flush(vm: &mut Machine) {
        while vm.next_flush_cycle().is_ok() {}
    }
    #[test]
    fn sequential_execution() -> Result<(), String> {
        let program = vec![
            "addi R1, R0, #100", // R1 = 100
            "addi R2, R0, #200", // R2 = 200
            "add R3, R1, R2",    // R3 = 300
            "add R4, R1, R3",    // R4 = 400
            "add R3, R4, R3",    // R3 = 700
            "addi R1, R5, #400", // R1 = 400
            "add R5, R1, R2",    // R5 = 600
                                 /* R1: 400
                                  * R2: 200
                                  * R3: 700
                                  * R4: 400
                                  * R5: 600
                                  */
        ];
        let reg_expect = vec![0, 400, 200, 700, 400, 600, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let reg_expect: Vec<ArgState> = reg_expect.iter().map(|v| ArgState::Ready(*v)).collect();

        let program = program.iter().map(|i| i.to_string()).collect();

        let mut p = Processor::new();
        let mut ff = Factory::new();
        for _ in 0..2 {
            let unit = ff.new_unit(Function::Arithmetic);
            p.add_path(unit)?;
        }
        let mut vm = Machine::new(p, program, 0);

        while vm.next_cycle().is_ok() {}
        flush(&mut vm);
        let (p, _) = vm.splite();
        let result = p.peek_registers();
        for (r, e) in result.iter().zip(reg_expect.iter()) {
            assert_eq!(r, e);
        }
        Ok(())
    }

    #[test]
    fn memory_access() -> Result<(), String> {
        /*
         * j = 0
         * k = 0
         * for i in 0..3 {
         *     j += 4
         *     k += 5
         * }
         * assert(j, 12)
         * assert(k, 15)
         */
        let expect_j = (12u32, 10u32); // (Value, address)
        let expect_k = (15u32, 14u32);
        let program = vec![
            "addi R1, R0, #0",
            "addi R2, R0, #10",
            "sw R1, R2, #0", // j = 0, &j == 10
            "sw R1, R2, #4", // k = 0, &k == 15
            "addi R3, R0, #4",
            "addi R4, R0, #5",
            // First iteration
            "lw R1, R2, #0",
            "add R1, R3, R1", // j += 4
            "sw R1, R2, #0",
            "lw R1, R2, #4",
            "add R1, R4, R1", // k += 5
            "sw R1, R2, #4",
            // Second iteration
            "lw R1, R2, #0",
            "add R1, R3, R1", // j += 4
            "sw R1, R2, #0",
            "lw R1, R2, #4",
            "add R1, R4, R1", // k += 5
            "sw R1, R2, #4",
            // Third iteration
            "lw R1, R2, #0",
            "add R1, R3, R1", // j += 4
            "sw R1, R2, #0",
            "lw R1, R2, #4",
            "add R1, R4, R1", // k += 5
            "sw R1, R2, #4",
        ];
        let program = program.iter().map(|i| i.to_string()).collect();

        let mut p = Processor::new();

        let mut ff = Factory::new();
        for _ in 0..2 {
            let unit = ff.new_unit(Function::Arithmetic);
            p.add_path(unit)?;
        }

        let unit = ff.new_mem_unit(MemFunction::MemoryAccess);
        p.add_mem_path(unit)?;

        let mut vm = Machine::new(p, program, 200);
        while vm.next_cycle().is_ok() {}
        flush(&mut vm);

        let (_processor, dram) = vm.splite();

        let assert = |expect: (u32, u32)| {
            let expect_value = expect.0;
            let address = expect.1 as usize;
            let got = raw_to_u32_big_endian(&dram[address..address + 4]);
            assert_eq!(got, expect_value);
        };
        println!("{:?}", dram);
        assert(expect_j);
        assert(expect_k);
        Ok(())
    }
}
