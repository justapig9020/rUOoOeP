use crate::core::execution_path::{ArgState, RStag};
use std::fmt::{Debug, Display};
use std::{default, mem};

#[derive(Debug)]
pub enum SlotState {
    Empty,
    Pending(Box<dyn RenamedInst>),
    Executing(Box<dyn RenamedInst>),
}

impl default::Default for SlotState {
    fn default() -> Self {
        SlotState::Empty
    }
}

impl SlotState {
    fn is_empty(&self) -> bool {
        if let SlotState::Empty = self {
            true
        } else {
            false
        }
    }
}

pub trait RenamedInst: Display + Debug {
    /// Return a copy of instruction name
    fn name(&self) -> &str;
    fn arguments(&self) -> Vec<ArgState>;
    /// An instruction is ready if it's not waiting result of another instruction.
    fn is_ready(&self) -> bool;
    fn forwarding(&mut self, tag: &RStag, val: u32);
}

#[derive(Debug)]
pub struct ReservationStation {
    slots: Vec<SlotState>,
}

#[cfg(test)]
mod resrvation_station {
    use super::*;

    #[derive(Debug)]
    struct InstStub {}
    impl Display for InstStub {
        fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Ok(())
        }
    }
    impl RenamedInst for InstStub {
        fn name(&self) -> &str {
            "inst"
        }
        fn arguments(&self) -> Vec<ArgState> {
            vec![]
        }
        fn is_ready(&self) -> bool {
            true
        }
        fn forwarding(&mut self, _tag: &RStag, _val: u32) {}
    }
    fn new_inst() -> Box<dyn RenamedInst> {
        Box::new(InstStub {})
    }
    #[test]
    fn pending() {
        let size = 10;
        let mut station = ReservationStation::new(size);
        let inst_cnt = 5;
        for _ in 0..inst_cnt {
            station.insert(new_inst());
        }
        assert_eq!(inst_cnt, station.pending());
    }
}

impl ReservationStation {
    pub fn new(size: usize) -> Self {
        Self {
            slots: (0..size).map(|_| SlotState::Empty).collect(),
        }
    }
    /// Insert a instruction to reservation station.
    /// Retuen Some(slot number) is the insert success.
    /// Return None if there is no empty slot.
    pub fn insert(&mut self, inst: Box<dyn RenamedInst>) -> Option<usize> {
        for (idx, slot) in self.slots.iter_mut().enumerate() {
            if slot.is_empty() {
                *slot = SlotState::Pending(inst);
                return Some(idx);
            }
        }
        None
    }
    /// Find a ready instruction.
    /// If found, its index returned.
    /// Otherwise, return None.
    pub fn ready(&mut self) -> Option<usize> {
        for (idx, slot) in self.slots.iter_mut().enumerate() {
            if let SlotState::Pending(inst) = slot {
                if inst.is_ready() {
                    return Some(idx);
                }
            }
        }
        None
    }
    /// The slot's instruction is solved, remove it.
    pub fn sloved(&mut self, idx: usize) {
        if idx < self.slots.len() {
            self.slots[idx] = SlotState::Empty;
        }
    }
    pub fn forwarding(&mut self, tag: &RStag, val: u32) {
        for slot in self.slots.iter_mut() {
            if let SlotState::Pending(slot) = slot {
                slot.forwarding(tag, val);
            }
        }
    }
    pub fn pending(&self) -> usize {
        let empty: usize = self.slots.iter().map(|a| a.is_empty() as usize).sum();
        self.slots.len() - empty
    }
    pub fn get_slot(&self, id: usize) -> Option<&SlotState> {
        self.slots.get(id)
    }
    /// Change state of given slot into executing
    pub fn start_execute(&mut self, id: usize) -> Result<(), String> {
        let slot = self
            .slots
            .get_mut(id)
            .ok_or(format!("{} is out of index", id))?;
        let slot_state = mem::take(slot);
        if let SlotState::Pending(inst) = slot_state {
            *slot = SlotState::Executing(inst);
            Ok(())
        } else {
            *slot = slot_state;
            Err(format!("Slot {} isn't pending", id))
        }
    }
    pub fn dump(&self) -> Vec<String> {
        self.slots
            .iter()
            .map(|slot| match slot {
                SlotState::Empty => String::from("Empty"),
                SlotState::Executing(inst) => format!("Exe({})", inst),
                SlotState::Pending(inst) => format!("Pend({})", inst),
            })
            .collect()
    }
}
