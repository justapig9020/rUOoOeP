use crate::core::execution_path::{ArgState, RStag};
use std::fmt::{Debug, Display};
use std::{default, mem};

#[derive(Debug)]
pub enum SlotState {
    Empty,
    Pending(Box<dyn RenamedInst>),
    Executing(Box<dyn RenamedInst>),
    Reserved,
}

impl default::Default for SlotState {
    fn default() -> Self {
        SlotState::Empty
    }
}

impl SlotState {
    fn is_pending(&self) -> bool {
        matches!(self, SlotState::Pending(_))
    }
    fn is_empty(&self) -> bool {
        matches!(self, SlotState::Empty)
    }
    fn is_reserved(&self) -> bool {
        matches!(self, SlotState::Reserved)
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

impl Display for SlotState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let info = match self {
            SlotState::Empty => String::from("Empty"),
            SlotState::Executing(inst) => format!("Exe({})", inst),
            SlotState::Pending(inst) => format!("Pend({})", inst),
            SlotState::Reserved => String::from("Reserved"),
        };
        write!(f, "{info}")
    }
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
        assert_eq!(inst_cnt, station.occupied());
    }
}

impl ReservationStation {
    pub fn new(size: usize) -> Self {
        Self {
            slots: (0..size).map(|_| SlotState::Empty).collect(),
        }
    }
    /// Return the capacity of the reservation station
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }
    /// Insert a instruction to reservation station.
    /// Retuen Some(slot number) is the insert success.
    /// Return None if there is no empty slot.
    pub fn insert(&mut self, inst: Box<dyn RenamedInst>) -> Option<usize> {
        let empty_idx = self.find_empty()?;
        let slot = self.slots.get_mut(empty_idx)?;
        *slot = SlotState::Pending(inst);
        Some(empty_idx)
    }
    /// Find an empty slot
    /// This method a index of empty slot is exist any.
    /// None returned otherwise.
    fn find_empty(&self) -> Option<usize> {
        for (idx, slot) in self.slots.iter().enumerate() {
            if slot.is_empty() {
                return Some(idx);
            }
        }
        None
    }
    /// Turn a reservation station slot to reserve.
    /// Reserved slot will not be reserve or be insert.
    /// Use insert_into_reserved to insert a instruction to the reserved slot.
    /// This method reserved a slot and return its slot index on sucesse.
    /// Otherwise, None returned.
    pub fn reserve(&mut self) -> Option<usize> {
        let empty_idx = self.find_empty()?;
        let slot = self.slots.get_mut(empty_idx)?;
        *slot = SlotState::Reserved;
        Some(empty_idx)
    }
    /// The method insert a instruction to a given reserved slot.
    /// If the slot of given index isn't reserving, the method failed.
    /// On success, the index of the slot returned.
    /// Otherwise, return an Err which packed a error message.
    pub fn insert_into_reserved_slot(
        &mut self,
        inst: Box<dyn RenamedInst>,
        idx: usize,
    ) -> Result<usize, String> {
        let slot = self
            .slots
            .get_mut(idx)
            .ok_or_else(|| format!("Reservation station: Index {} out of bound", idx))?;
        if slot.is_reserved() {
            *slot = SlotState::Pending(inst);
            Ok(idx)
        } else {
            let msg = format!("Reservation station: Slot {} is not reserved", idx);
            Err(msg)
        }
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
        if let Some(slot) = self.slots.get_mut(idx) {
            *slot = SlotState::Empty;
        }
    }
    pub fn forwarding(&mut self, tag: &RStag, val: u32) {
        for slot in self.slots.iter_mut() {
            if let SlotState::Pending(slot) = slot {
                slot.forwarding(tag, val);
            }
        }
    }
    /// Return pending
    pub fn pending(&self) -> usize {
        self.slots.iter().filter(|s| s.is_pending()).count()
    }
    /// Return non-empty slot count.
    pub fn occupied(&self) -> usize {
        let empty: usize = self.slots.iter().filter(|s| s.is_empty()).count();
        self.slots.len() - empty
    }
    pub fn is_full(&self) -> bool {
        self.occupied() == self.capacity()
    }
    pub fn get_slot(&self, idx: usize) -> Option<&SlotState> {
        self.slots.get(idx)
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
                SlotState::Reserved => String::from("Reserved"),
            })
            .collect()
    }
}

impl<'b> IntoIterator for &'b ReservationStation {
    type IntoIter = std::slice::Iter<'b, SlotState>;
    type Item = &'b SlotState;
    fn into_iter(self) -> Self::IntoIter {
        self.slots.iter()
    }
}
