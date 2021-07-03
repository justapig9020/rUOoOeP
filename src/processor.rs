use crate::decoder::{Decoder, DecodedInst, ArgType};
use crate::execution_path::{execution_path_factory, ExecPath, ArgVal};
use crate::register::RegFile;
use std::collections::HashMap;
use crate::result_bus::ResultBus;

#[derive(Debug)]
pub struct Processor {
    pc: usize,
    decoder: Decoder,
    paths: HashMap<String, Box<dyn ExecPath>>,
    register_file: RegFile,
    result_bus: ResultBus,
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
    pub fn next_cycle(&mut self, inst: &str) -> Result<(), String> {
        if let Some((tag, result)) = self.result_bus.take() {
            let val = result.val();
            for (_, station) in self.paths.iter_mut() {
                station.forwarding(tag.clone(), val);
            }
            self.register_file.write(tag, val);
        }
        let inst = self.decoder.decode(inst)?;
        let args = inst.get_args();
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
            let val;
            match *arg {
                ArgType::Reg(idx) => {
                    val = self.register_file.read(idx);
                },
                ArgType::Imm(imm) => {
                    val = ArgVal::Imm(imm);
                },
            }
            arg_vals.push(val);
        }

        let mut issued = false;
        // Searching for a suitable station to issue the instruction
        for name in inst.get_stations().iter() {
            // Find a reservation station by name
            if let Some(station) = self.paths.get_mut(name) {
                if let Ok(tag) = station.issue(inst.get_name(), &arg_vals) {
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
}
