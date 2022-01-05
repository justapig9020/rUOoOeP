use crate::decoder::{Decoder, DecodedInst, ArgType};
use crate::execution_path::{execution_path_factory, ExecPath, ArgVal, RStag};
use crate::register::RegFile;
use std::collections::HashMap;
use crate::result_bus::ResultBus;
use crate::display::into_table;
use std::fmt;

#[derive(Debug)]
pub struct Processor {
    pc: usize,
    decoder: Decoder,
    paths: HashMap<String, Box<dyn ExecPath>>,
    register_file: RegFile,
    result_bus: ResultBus,
}

impl fmt::Display for Processor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let info = self.print();
        write!(f, "{}", info)
    }
}

impl Processor {
    pub fn new() -> Self {
        Self {
            pc: 0,
            decoder: Decoder::new(),
            paths: HashMap::new(),
            register_file: RegFile::new(),
            result_bus: ResultBus::new(),
        }
    }
    /// Add an execution path to the processor.
    pub fn add_path(&mut self, func: &str) -> Result<(), String> {
        let path = execution_path_factory(&func)?;
        let insts = path.list_inst();
        let name = path.get_name();

        if let Some(prev) = self.paths.insert(name.clone(), path) {
            let msg = format!("Already has a execution path with name {}", prev.get_name());
            Err(msg)
        } else {
            self.decoder.register(insts, name)
        }
    }
    pub fn fetching(&self) -> usize {
        self.pc
    }
    fn commit(&mut self) -> bool {
        let result = self.result_bus.take();
        let forward = |(tag, val): (RStag, u32)| -> Option<(RStag, u32)>{
            for (_, station) in self.paths.iter_mut() {
                station.forwarding(tag.clone(), val);
            }
            Some((tag, val))
        };
        result.map(|(tag, result)| (tag, result.val()))
            .and_then(forward)
            .and_then(|(tag, val)|  {
                self.register_file.write(tag, val);
                Some(())
            })
            .is_some()

    }
    pub fn next_cycle(&mut self, inst: &str) -> Result<(), String> {
        self.commit();

        let inst = self.decoder.decode(inst)?;
        let args = inst.args();
        let mut arg_vals = Vec::with_capacity(args.len());
        let mut start = 0;
        let mut dest = None;
        if inst.is_writeback() {
            if args.len() == 0 {
                let msg = String::from("Expcet more than one argument");
                return Err(msg);
            }
            if let ArgType::Reg(idx) = args[0]{
                start = 1;
                dest = Some(idx);
            }
        }

        // Mapping arguments from types to data
        for arg in args[start..].iter() {
            let val = match *arg {
                ArgType::Reg(idx) => {
                    self.register_file.read(idx)
                },
                ArgType::Imm(imm) => {
                    ArgVal::Ready(imm)
                },
            };
            arg_vals.push(val);
        }

        let mut issued = false;
        // Searching for a suitable station to issue the instruction
        for name in inst.stations().iter() {
            // Find a reservation station by name
            if let Some(station) = self.paths.get_mut(name) {
                if let Ok(tag) = station.issue(inst.name(), &arg_vals) {
                    if let Some(idx) = dest {
                        self.register_file.rename(idx, tag);
                    }
                    // The instruction has been issued.
                    issued = true;
                    break;
                }
            }
        }

        for (_, exec_unit) in self.paths.iter_mut() {
            exec_unit.next_cycle(&mut self.result_bus);
        }

        // If the instruction not issued, stall the instruction fetch
        // untill there are some reservation station is ready.
        if issued {
            self.pc += 1;
        }
        Ok(())
    }
    fn print(&self) -> String {
        let mut info = String::new();
        let mut registers = vec![format!("PC: {}", self.pc)];
        let mut gpr = self.register_file.dump();
        registers.append(&mut gpr);
        let last_instruction = self.decoder.last_instruction().to_string();
        info.push_str(&into_table("Instruction", vec![last_instruction]));
        info.push_str(&into_table("Registers", registers));
        self.paths.iter().for_each(|(_, p)| info.push_str(&p.dump()));
        info
    }
}

