use crate::decoder::{ArgType, DecodedInst, Decoder};
use crate::display::into_table;
use crate::execution_path::{execution_path_factory, ArgState, ExecPath, RStag};
use crate::register::RegisterFile;
use crate::result_bus::ResultBus;
use std::collections::HashMap;
use std::fmt;

enum IssueResult {
    Issued(RStag),
    Stall,
}

#[derive(Debug)]
pub struct Processor {
    pc: usize,
    decoder: Decoder,
    paths: HashMap<String, Box<dyn ExecPath>>,
    register_file: RegisterFile,
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
            register_file: RegisterFile::new(),
            result_bus: ResultBus::new(),
        }
    }
    /// Add an execution path to the processor.
    pub fn add_path(&mut self, func: &str) -> Result<(), String> {
        let path = execution_path_factory(&func)?;
        let insts = path.list_insts();
        let name = path.name();

        if let Some(prev) = self.paths.insert(name.clone(), path) {
            let msg = format!("Already has a execution path with name {}", prev.name());
            Err(msg)
        } else {
            self.decoder.register(insts, name)
        }
    }
    /// Return fetching address.
    pub fn fetch_address(&self) -> usize {
        self.pc
    }
    /// Commit result and forward to reservation stations.
    /// If result bus is holding data to commit, then return `True`.
    /// Otherwise, return `False`.
    fn commit(&mut self) -> bool {
        let result = self.result_bus.take();
        let forward = |(tag, val): (RStag, u32)| -> Option<(RStag, u32)> {
            for (_, station) in self.paths.iter_mut() {
                station.forward(tag.clone(), val);
            }
            Some((tag, val))
        };
        result
            .map(|(tag, result)| (tag, result.val()))
            .and_then(forward)
            .and_then(|(tag, val)| {
                self.register_file.write(tag, val);
                Some(())
            })
            .is_some()
    }
    /// If issuable reservation found, the instruction issued and [IssueResult::Issued].
    /// Otherwise [IssueResult::Stall] returned.
    fn try_issue(&mut self, inst: &DecodedInst, renamed_args: &[ArgState]) -> IssueResult {
        let name_of_stations = inst.stations();
        // Order stations by pending instruction count.
        // Therefore, instructions can be execute more parallelly.
        let mut stations = name_of_stations
            .iter()
            .map(|name| {
                let station = self
                    .paths
                    .get(name)
                    .unwrap_or_else(|| panic!("Station '{}' not found", name));
                (name, station.pending())
            })
            .collect::<Vec<(&String, usize)>>();
        stations.sort_by_key(|(_, p)| *p);

        for (name, _) in stations.iter() {
            let station = self.paths.get_mut(*name);
            if let Some(station) = station {
                let slot_tag = station.try_issue(inst.name(), renamed_args);
                if let Ok(tag) = slot_tag {
                    return IssueResult::Issued(tag);
                }
            }
        }
        // Issuable reservation not found
        IssueResult::Stall
    }
    /// If instruction writeback, Rename destination register to tag of reservation station slot which holds the instruction.
    /// Otherwise, do nothing.
    fn register_renaming(&mut self, tag: RStag, inst: DecodedInst) -> Result<(), String> {
        let mut ret = Ok(());
        if let Some(dest) = inst.writeback() {
            match dest {
                ArgType::Reg(idx) => self.register_file.rename(idx, tag),
                _ => {
                    let msg = format!("{:?} is not a valid write back destination", dest);
                    ret = Err(msg);
                }
            };
        }
        ret
    }
    /// Return Err(`Error Message`) if error occur.
    pub fn next_cycle(&mut self, row_inst: &str) -> Result<(), String> {
        let mut next_pc = self.pc;
        self.commit();
        // TODO: Implement "NOP" as an instruction
        if row_inst != "NOP" {
            let inst = self.decoder.decode(row_inst)?;
            let args = inst.arguments();
            let mut renamed_args = Vec::with_capacity(args.len());

            // Mapping arguments from types to data
            for arg in args.iter() {
                let val = match *arg {
                    ArgType::Reg(idx) => self.register_file.read(idx),
                    ArgType::Imm(imm) => ArgState::Ready(imm),
                };
                renamed_args.push(val);
            }

            let result = self.try_issue(&inst, &renamed_args);
            if let IssueResult::Issued(tag) = result {
                next_pc += 1;
                self.register_renaming(tag, inst)?;
            }
        } else {
            next_pc += 1;
        }

        for (_, exec_unit) in self.paths.iter_mut() {
            exec_unit.next_cycle(&mut self.result_bus)?;
        }

        self.pc = next_pc;
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
        self.paths.iter().for_each(|(_, p)| {
            info.push_str(&p.dump());
            info.push('\n');
        });
        info.push_str(&format!("{:?}", self.result_bus));
        info
    }
}
