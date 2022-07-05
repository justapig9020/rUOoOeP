use super::decoder::{ArgType, DecodedInst, Decoder};
use super::execution_path::{
    AccessPath, ArgState, BusAccessRequst, BusAccessResponse, ExecPath, RStag,
};
use super::nop_unit;
use super::register::RegisterFile;
use super::result_bus::ResultBus;
use crate::display::into_table;
use std::collections::HashMap;
use std::fmt;

enum IssueResult {
    Issued(RStag),
    Stall,
}

#[derive(Debug)]
struct BusController {
    access_queue: Vec<BusAccessRequst>,
}

impl BusController {
    fn new() -> Self {
        Self {
            access_queue: Vec::new(),
        }
    }
    fn push(&mut self, request: BusAccessRequst) {
        self.access_queue.push(request);
    }
}
#[derive(Debug)]
pub struct Processor {
    pc: usize,
    decoder: Decoder,
    arthmatic_paths: HashMap<String, Box<dyn ExecPath>>,
    access_paths: HashMap<String, Box<dyn AccessPath>>,
    bus_controller: BusController,
    register_file: RegisterFile,
    result_bus: ResultBus,
}

impl fmt::Display for Processor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut registers = vec![format!("PC: {}", self.pc)];
        self.register_file
            .into_iter()
            .enumerate()
            .for_each(|(idx, reg)| {
                registers.push(format!("R{idx}: {reg}"));
            });
        let last_instruction = self.decoder.last_instruction().to_string();
        writeln!(f, "{}", into_table("Instruction", vec![last_instruction]))?;
        writeln!(f, "{}", into_table("Registers", registers))?;
        for (_, p) in self.arthmatic_paths.iter() {
            writeln!(f, "{}", p)?;
        }
        for (_, p) in self.access_paths.iter() {
            writeln!(f, "{}", p)?;
        }
        writeln!(f, "{}", self.result_bus)
    }
}

impl Processor {
    pub fn new() -> Self {
        let mut ret = Self {
            pc: 0,
            decoder: Decoder::new(),
            arthmatic_paths: HashMap::new(),
            access_paths: HashMap::new(),
            bus_controller: BusController::new(),
            register_file: RegisterFile::new(),
            result_bus: ResultBus::new(),
        };
        let nop_unit = Box::new(nop_unit::Unit::new());
        ret.add_path(nop_unit)
            .expect("Unable to add nop instruction path");
        ret
    }
    /// Add an execution path to the processor.
    pub fn add_path(&mut self, func: Box<dyn ExecPath>) -> Result<(), String> {
        let insts = func.list_insts();
        let name = func.name();

        if let Some(prev) = self.arthmatic_paths.insert(name.clone(), func) {
            let msg = format!("Already has a execution path with name {}", prev.name());
            Err(msg)
        } else {
            self.decoder.register(insts, name)
        }
    }
    pub fn add_mem_path(&mut self, func: Box<dyn AccessPath>) -> Result<(), String> {
        let insts = func.list_insts();
        let name = func.name();

        if let Some(prev) = self.access_paths.insert(name.clone(), func) {
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
            for (_, station) in self.arthmatic_paths.iter_mut() {
                station.forward(tag.clone(), val);
            }
            for (_, station) in self.access_paths.iter_mut() {
                station.forward(tag.clone(), val);
            }
            Some((tag, val))
        };
        result
            .map(|(tag, result)| (tag, result.val()))
            .and_then(forward)
            .map(|(tag, val)| {
                self.register_file.write(tag, val);
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
                let arth = self.arthmatic_paths.get(name);
                let access = self.access_paths.get(name);
                let station = if let Some(s) = arth {
                    &**s
                } else if let Some(s) = access {
                    &**s as &dyn ExecPath
                } else {
                    panic!("No path named {}", name);
                };
                (name, station.pending())
            })
            .collect::<Vec<(&String, usize)>>();
        stations.sort_by_key(|(_, p)| *p);

        for (name, _) in stations.iter() {
            let station = self.arthmatic_paths.get_mut(*name);
            if let Some(station) = station {
                let slot_tag = station.try_issue(inst.name(), renamed_args);
                if let Ok(tag) = slot_tag {
                    return IssueResult::Issued(tag);
                }
            }
            let station = self.access_paths.get_mut(*name);
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

        for (_, unit) in self.arthmatic_paths.iter_mut() {
            unit.next_cycle(&mut self.result_bus)?;
        }

        for (_, unit) in self.access_paths.iter_mut() {
            unit.next_cycle(&mut self.result_bus)?;
            if let Some(r) = unit.request() {
                self.bus_controller.push(r);
            }
        }

        self.pc = next_pc;
        Ok(())
    }
    pub fn bus_access(&mut self) -> Option<BusAccessRequst> {
        let controller = &mut self.bus_controller;
        let request = controller.access_queue.pop()?;
        Some(request)
    }
    pub fn resolve_access(&mut self, reponse: BusAccessResponse) -> Result<(), String> {
        let path = reponse.path_name();
        let slot = reponse.slot();

        let unit = self
            .access_paths
            .get_mut(&path)
            .ok_or(format!("Path {} not found", path))?;
        unit.response(slot, reponse.into_result());
        Ok(())
    }
    #[allow(dead_code)]
    /// This function is used to testing
    pub fn peek_registers(&self) -> Vec<ArgState> {
        let rf = &self.register_file;
        let size = rf.size();
        (0..size).map(|i| rf.read(i)).collect()
    }
}
