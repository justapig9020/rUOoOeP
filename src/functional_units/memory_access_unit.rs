use std::fmt::Display;
use std::ops::Range;

use crate::{
    core::{
        decoder::{InstFormat, TokenType},
        execution_path::{
            AccessPath, ArgState, BusAccessRequst, BusAccessResponse, BusAccessResult, ExecPath,
            ExecResult, RStag,
        },
        result_bus::ResultBus,
    },
    display::into_table,
    functional_units::reservation_station::SlotState,
    util::{queue::Queue, raw_to_u32_big_endian, u32_to_raw_big_endian},
};

use super::reservation_station::{RenamedInst, ReservationStation};

const FUNC: &str = "mem_access";
const LOAD_STATION_SIZE: usize = 4;
const STORE_STATION_SIZE: usize = 4;
const PENDING_CAPACITY: usize = LOAD_STATION_SIZE + STORE_STATION_SIZE;

#[derive(Clone, Copy)]
enum AccessType {
    Load,
    Store,
}

fn access_overlap(a: &Range<u32>, b: &Range<u32>) -> bool {
    !(a.end <= b.start || a.start >= b.end)
}

#[cfg(test)]
mod overlap {
    use super::access_overlap;

    #[test]
    fn overlap() {
        let first = 10;
        let second = 20;
        let third = 30;
        let fourth = 40;

        assert_eq!(
            access_overlap(&(third..fourth), &(first..second)),
            false,
            "A > B"
        );
        assert_eq!(
            access_overlap(&(first..second), &(third..fourth)),
            false,
            "A < B"
        );
        assert_eq!(
            access_overlap(&(second..fourth), &(first..third)),
            true,
            "A > B parital overlap"
        );
        assert_eq!(
            access_overlap(&(first..fourth), &(second..third)),
            true,
            "A covered B"
        );
        assert_eq!(
            access_overlap(&(first..third), &(second..fourth)),
            true,
            "A < B partial overlap"
        );
        assert_eq!(
            access_overlap(&(second..third), &(first..fourth)),
            true,
            "B covered A"
        );
    }
}

fn get_access_len(identifier: char) -> usize {
    match identifier {
        'w' => 4,
        _ => panic!("Unknow access len"),
    }
}

fn get_access_range(inst: &str, base: u32) -> Range<u32> {
    let (_, len) = AccessType::parse(inst);
    base..base + len as u32
}

impl AccessType {
    fn parse(inst: &str) -> (Self, usize) {
        let mut chars = inst.chars();
        let type_identifier = chars.next().expect("Missing access type");
        let ty = match type_identifier {
            'l' => AccessType::Load,
            's' => AccessType::Store,
            _ => panic!("Undefined access type {}", type_identifier),
        };
        let len_identifier = chars.next().expect("Missing access length");
        let len = get_access_len(len_identifier);
        (ty, len)
    }
}

#[derive(Debug)]
/// Memory address which is going to access
enum MemAddress {
    /// The address has evaluated to exact number
    Evaluated(u32),
    /// The address are waiting for argument
    Evaluating(ArgState, u32),
}

impl Display for MemAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemAddress::Evaluated(address) => write!(f, "{}", address),
            MemAddress::Evaluating(base, offset) => write!(f, "{} + {}", base, offset),
        }
    }
}

impl MemAddress {
    fn from_args(base: ArgState, offset: ArgState) -> Self {
        if let ArgState::Ready(offset) = offset {
            MemAddress::Evaluating(base, offset)
        } else {
            panic!("Offset of memory address must be immediat number");
        }
    }
    fn forwarding(&mut self, tag: &RStag, val: u32) {
        use MemAddress::*;
        if let Evaluating(arg, _) = self {
            arg.forwarding(tag, val);
        }
    }
    fn arguments(&self) -> Vec<ArgState> {
        use MemAddress::*;
        match self {
            Evaluated(base) => vec![ArgState::Ready(*base)],
            Evaluating(arg, _) => vec![arg.clone()],
        }
    }
    fn ready_for_evaluation(&self) -> Option<(u32, u32)> {
        match self {
            MemAddress::Evaluated(base) => Some((*base, 0)),
            MemAddress::Evaluating(base, offset) => {
                let base = base.val()?;
                Some((base, *offset))
            }
        }
    }
}

#[derive(Debug)]
/// Arguments of different memory access instruction
enum AccessArgs {
    Load(MemAddress),
    Store(ArgState, MemAddress),
}

impl Display for AccessArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessArgs::Load(address) => {
                write!(f, "Load({})", address)
            }
            AccessArgs::Store(value, address) => {
                write!(f, "{}; {}", value, address)
            }
        }
    }
}

impl AccessArgs {
    fn new(access_type: AccessType, renamed_args: &[ArgState]) -> Self {
        match access_type {
            AccessType::Load => AccessArgs::new_load(renamed_args),
            AccessType::Store => AccessArgs::new_store(renamed_args),
        }
    }
    fn new_load(renamed_args: &[ArgState]) -> Self {
        let expect_arg_cnt = 2;
        if renamed_args.len() != expect_arg_cnt {
            panic!(
                "Load instruction expect {} arguments but {} got",
                expect_arg_cnt,
                renamed_args.len()
            );
        }

        let base = renamed_args[0].clone();
        let offset = renamed_args[1].clone();
        let source = MemAddress::from_args(base, offset);
        AccessArgs::Load(source)
    }
    fn new_store(renamed_args: &[ArgState]) -> Self {
        let expect_arg_cnt = 3;
        if renamed_args.len() != expect_arg_cnt {
            panic!(
                "Store instruction expect {} arguments but {} got",
                expect_arg_cnt,
                renamed_args.len()
            );
        }

        let source = renamed_args[0].clone();
        let base = renamed_args[1].clone();
        let offset = renamed_args[2].clone();

        let destination = MemAddress::from_args(base, offset);
        AccessArgs::Store(source, destination)
    }
    fn forwarding(&mut self, tag: &RStag, val: u32) {
        match self {
            AccessArgs::Load(src) => src.forwarding(tag, val),
            AccessArgs::Store(src, dest) => {
                src.forwarding(tag, val);
                dest.forwarding(tag, val);
            }
        }
    }
    fn arguments(&self) -> Vec<ArgState> {
        match self {
            AccessArgs::Load(src) => src.arguments(),
            AccessArgs::Store(src, dest) => {
                let mut args = vec![src.clone()];
                args.append(&mut dest.arguments());
                args
            }
        }
    }
    fn evaluated(&mut self, base: u32) {
        let address = match self {
            AccessArgs::Load(address) => address,
            AccessArgs::Store(_, address) => address,
        };
        if let MemAddress::Evaluating(_, _) = address {
            *address = MemAddress::Evaluated(base);
        }
    }
    fn read_for_evaluation(&self) -> Option<(u32, u32)> {
        let address = match self {
            AccessArgs::Load(address) => address,
            AccessArgs::Store(_, address) => address,
        };
        address.ready_for_evaluation()
    }
}

#[derive(Debug)]
struct AccessInst {
    name: String,
    args: AccessArgs,
    dependencies: Vec<RStag>,
}

impl Display for AccessInst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}; {}; {:?}", self.name(), self.args, self.dependencies)
    }
}

impl AccessInst {
    fn new(name: String, renamed_args: &[ArgState]) -> Self {
        let (access_type, _) = AccessType::parse(&name);
        let args = AccessArgs::new(access_type, renamed_args);
        Self {
            name,
            args,
            dependencies: vec![],
        }
    }
    fn access_type(&self) -> AccessType {
        match self.args {
            AccessArgs::Load(_) => AccessType::Load,
            AccessArgs::Store(_, _) => AccessType::Store,
        }
    }
    fn dependency_free(&self) -> bool {
        self.dependencies.is_empty()
    }
    fn evaluated(&mut self, base: u32, dependiencies: Vec<RStag>) {
        self.dependencies = dependiencies;
        self.args.evaluated(base);
    }
    fn read_for_evaluation(&self) -> Option<(u32, u32)> {
        self.args.read_for_evaluation()
    }
}

impl RenamedInst for AccessInst {
    fn name(&self) -> &str {
        &self.name
    }
    fn arguments(&self) -> Vec<ArgState> {
        self.args.arguments()
    }
    fn forwarding(&mut self, tag: &RStag, val: u32) {
        self.args.forwarding(tag, val);
    }
    fn is_ready(&self) -> bool {
        if !self.dependency_free() {
            return false;
        }
        let waiting = self
            .arguments()
            .iter()
            .filter(|arg| matches!(**arg, ArgState::Waiting(_)))
            .count();
        waiting == 0
    }
}

#[cfg(test)]
mod access_instruction {
    use super::*;
    #[test]
    fn ready_check_for_load() {
        let base = RStag::new("base", 10);
        let args = [ArgState::Waiting(base.clone()), ArgState::Ready(10)];
        let inst_name = String::from("lw");
        let mut inst = AccessInst::new(inst_name, &args);

        assert_eq!(false, inst.is_ready());

        inst.forwarding(&base, 10);

        assert_eq!(true, inst.is_ready());
    }
    #[test]
    fn ready_check_for_store() {
        let base = RStag::new("base", 10);
        let source = RStag::new("source", 10);
        let args = [
            ArgState::Waiting(source.clone()),
            ArgState::Waiting(base.clone()),
            ArgState::Ready(10),
        ];
        let inst_name = String::from("sw");
        let mut inst = AccessInst::new(inst_name, &args);

        assert_eq!(false, inst.is_ready());

        inst.forwarding(&base, 10);

        assert_eq!(false, inst.is_ready());

        inst.forwarding(&source, 10);

        assert_eq!(true, inst.is_ready());
    }
}

#[derive(Debug)]
struct EvaluationUnit {
    remain_cycle: usize,
    result: u32,
}

impl EvaluationUnit {
    fn exec(_inst: String, base: u32, offset: u32) -> Self {
        Self {
            remain_cycle: 1,
            result: base + offset,
        }
    }
    fn next_cycle(&mut self) -> Option<u32> {
        if self.remain_cycle == 0 {
            Some(self.result)
        } else {
            self.remain_cycle -= 1;
            None
        }
    }
}

#[derive(Debug)]
pub struct Unit {
    name: String,
    /// (logical slot id, evaluating instruction)
    evaluation_queue: Queue<(usize, AccessInst)>,
    evaluating: Option<EvaluationUnit>,
    load_station: ReservationStation,
    store_station: ReservationStation,
    /// (logical slot id, execution result)
    result: Option<(usize, ExecResult)>,
}

impl Unit {
    pub fn new(idx: usize) -> Self {
        Self {
            name: format!("{}{}", FUNC, idx),
            evaluation_queue: Queue::new(PENDING_CAPACITY),
            evaluating: None,
            load_station: ReservationStation::new(LOAD_STATION_SIZE),
            store_station: ReservationStation::new(STORE_STATION_SIZE),
            result: None,
        }
    }
    fn physical_slot_id_to_logical(phy_id: usize, access_type: AccessType) -> usize {
        /* In register renaming, both load and store stations in a access unit shared a same slot index space.
         * The mapping policy from physical to logical id is:
         * Load => logical id = physical id
         * Store => logical id = Load station capacity + physical id
         */
        match access_type {
            AccessType::Load => phy_id,
            AccessType::Store => LOAD_STATION_SIZE + phy_id,
        }
    }
    fn logical_slot_id_to_physical(logical_id: usize) -> (AccessType, usize) {
        if logical_id >= LOAD_STATION_SIZE {
            (AccessType::Store, logical_id - LOAD_STATION_SIZE)
        } else {
            (AccessType::Load, logical_id)
        }
    }
    fn evaluating_queue_forward(&mut self, tag: &RStag, val: u32) {
        for (_, slot) in &mut self.evaluation_queue {
            slot.forwarding(tag, val);
        }
    }
    /// Check and list pending accesses which with access range overlaping with the given range
    /// This function return a vector of RStag of access range overlaping pending instruction
    fn dependency_check(&self, access_type: AccessType, target: Range<u32>) -> Vec<RStag> {
        /*
         * Type of dependencies:
         * - Load after Store
         * - Store after Load
         * - Store after Store
         * Therefore, for load access we check store station only.
         * In the other hand, both stations have to be checked in store request.
         */
        let mut dependencies = self.dependency_check_of_station(AccessType::Store, &target);
        if let AccessType::Store = access_type {
            let mut load_dependencies = self.dependency_check_of_station(AccessType::Load, &target);
            dependencies.append(&mut load_dependencies);
        }
        dependencies
    }
    fn dependency_check_of_station(
        &self,
        access_type: AccessType,
        target: &Range<u32>,
    ) -> Vec<RStag> {
        let station = match access_type {
            AccessType::Load => &self.load_station,
            AccessType::Store => &self.store_station,
        };
        let mut dependencies = Vec::new();
        for (phy_id, slot) in station.into_iter().enumerate() {
            if let SlotState::Pending(inst) = slot {
                let args = inst.arguments();
                let base = args.last().expect("Base address not found");
                if let ArgState::Ready(base) = base {
                    let previous = get_access_range(inst.name(), *base);
                    let log_id = Unit::physical_slot_id_to_logical(phy_id, access_type);
                    if access_overlap(&previous, target) {
                        dependencies.push(RStag::new(&self.name, log_id));
                    }
                }
            }
        }
        dependencies
    }
    /// Issue first instruction in the evaluation queue to corresponding reservation station with evaluated base address
    /// On success, this function returns the logical slot number that the instruction issued to
    /// Otherwise, Err which contents error message returned
    fn issue_evaluated_instruction_to_station(
        &mut self,
        evaluated_base: u32,
    ) -> Result<usize, String> {
        let (reserved_id, mut issuing) = self
            .evaluation_queue
            .pop()
            .ok_or(String::from("Expect instruction in evaluating queue while issuing instruction to reservation station"))?;

        let (access_type, len) = AccessType::parse(issuing.name());
        let access_range = evaluated_base..evaluated_base + len as u32;
        let dependiencies = self.dependency_check(access_type, access_range);

        issuing.evaluated(evaluated_base, dependiencies);

        let station = match access_type {
            AccessType::Load => &mut self.load_station,
            AccessType::Store => &mut self.store_station,
        };

        let issuing = Box::new(issuing) as Box<dyn RenamedInst>;
        station
            .insert_into_reserved_slot(issuing, reserved_id)
            .map(|phy_id| Unit::physical_slot_id_to_logical(phy_id, access_type))
    }
}

impl ExecPath for Unit {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn function(&self) -> String {
        String::from(FUNC)
    }
    fn list_insts(&self) -> Vec<InstFormat> {
        vec![
            InstFormat::create("lw")
                .add_syntax(TokenType::Writeback)
                .add_syntax(TokenType::Register)
                .add_syntax(TokenType::Immediate)
                .done(),
            InstFormat::create("sw")
                .add_syntax(TokenType::Register)
                .add_syntax(TokenType::Register)
                .add_syntax(TokenType::Immediate)
                .done(),
        ]
    }
    fn forward(&mut self, tag: RStag, val: u32) {
        let inst_src = tag.station();

        // If the forwarding result comes from local, reslove and free the corresponding reservation station slot
        if self.name == inst_src {
            let logical_id = tag.slot();
            let (acc_type, phy_id) = Unit::logical_slot_id_to_physical(logical_id);
            match acc_type {
                AccessType::Load => self.load_station.sloved(phy_id),
                AccessType::Store => self.store_station.sloved(phy_id),
            }
        }

        self.evaluating_queue_forward(&tag, val);
        self.load_station.forwarding(&tag, val);
        self.store_station.forwarding(&tag, val)
    }
    fn try_issue(&mut self, inst: String, vals: &[ArgState]) -> Result<RStag, ()> {
        if self.evaluation_queue.is_full() {
            return Err(());
        }
        let inst = AccessInst::new(inst, vals);
        let access_type = inst.access_type();
        let issue_dest = match access_type {
            AccessType::Load => &mut self.load_station,
            AccessType::Store => &mut self.store_station,
        };
        if issue_dest.is_full() {
            return Err(());
        }

        // Reserve might failed due to no empty slot
        let phy_id = issue_dest.reserve().ok_or(())?;

        /*
         * Evaluating queue's capacity equal to load station's capacity plus store station's capacity
         * Since we have allocated a slot in one of the station, there should has at least one slot in evaluating queue
         */
        self.evaluation_queue
            .insert((phy_id, inst))
            .expect("Evaluating queue never overflow");

        let logical_slot_id = Unit::physical_slot_id_to_logical(phy_id, access_type);
        Ok(RStag::new(&self.name, logical_slot_id))
    }
    fn next_cycle(&mut self, bus: &mut ResultBus) -> Result<(), String> {
        if let Some(evaluating) = &mut self.evaluating {
            let result = evaluating.next_cycle();
            if let Some(evaluated_base) = result {
                self.issue_evaluated_instruction_to_station(evaluated_base)?;
                self.evaluating = None;
            }
        } else {
            if let Some((_, to_evaluate)) = self.evaluation_queue.head() {
                if let Some((base, offset)) = to_evaluate.read_for_evaluation() {
                    let evaluation =
                        EvaluationUnit::exec(to_evaluate.name().to_string(), base, offset);
                    self.evaluating = Some(evaluation);
                }
            }
        }
        if bus.is_free() {
            if let Some((logical_id, result)) = self.result.take() {
                let tag = RStag::new(&self.name, logical_id);
                bus.set(tag, result);
            }
        }
        Ok(())
    }
    fn pending(&self) -> usize {
        self.evaluation_queue.len()
    }
    fn dump(&self) -> String {
        let mut info = format!("{}\n", self.name);
        let slots: Vec<String> = self
            .evaluation_queue
            .into_iter()
            .map(|(s, i)| format!("{}: {}", s, i))
            .collect();
        info.push_str(&into_table("Evaluating", slots));
        let slots: Vec<String> = self.load_station.dump();
        info.push_str(&into_table("Load station", slots));
        let slots: Vec<String> = self.store_station.dump();
        info.push_str(&into_table("Store station", slots));
        info
    }
}

impl AccessPath for Unit {
    fn request(&mut self) -> Option<BusAccessRequst> {
        let path = self.name();
        let store_pending = self.store_station.pending();
        let load_pending = self.load_station.pending();

        let (station, access_type) = if store_pending < load_pending {
            (&mut self.load_station, AccessType::Load)
        } else {
            (&mut self.store_station, AccessType::Store)
        };

        let slot_id = station.ready()?;
        let logical_id = Unit::physical_slot_id_to_logical(slot_id, access_type);
        let slot = station.get_slot(slot_id)?;
        if let SlotState::Pending(inst) = slot {
            let (access_type, len) = AccessType::parse(inst.name());
            /*
             * Argument format of instructions are:
             * - lw: [address]
             * - sw: [value, address]
             */
            let args: Vec<u32> = inst
                .arguments()
                .iter()
                .map(|arg| match arg {
                    ArgState::Ready(val) => *val,
                    ArgState::Waiting(_) => {
                        panic!("Ready instruction should not has waiting argument")
                    }
                })
                .collect();

            let request = match access_type {
                AccessType::Load => {
                    let address = args.get(0).expect("Address not found");
                    BusAccessRequst::new_load(path, logical_id, *address, len)
                }
                AccessType::Store => {
                    let value = args.get(0).expect("Value not found");
                    let value = u32_to_raw_big_endian(*value);
                    let address = args.get(1).expect("Address not found");
                    BusAccessRequst::new_store(path, logical_id, *address, value)
                }
            };
            station
                .start_execute(slot_id)
                .unwrap_or_else(|msg| panic!("{}", msg));
            Some(request)
        } else {
            None
        }
    }
    fn response(&mut self, slot: usize, response: Result<BusAccessResult, String>) {
        let result = response
            .map(|resp| match resp {
                BusAccessResult::Load(value) => ExecResult::MemLoad(value),
                BusAccessResult::Store => ExecResult::MemStore,
            })
            .or_else(|msg| -> Result<ExecResult, ()> { Ok(ExecResult::Err(msg)) })
            .expect("There is not path to Error");
        self.result = Some((slot, result));
    }
}
