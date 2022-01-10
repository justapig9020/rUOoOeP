use crate::execution_path::{ArgState, RStag};
use std::default::Default;

#[derive(Default, Debug)]
/// Renamable register file
pub struct RegisterFile {
    entry: [Entry; 16],
}

#[derive(Debug)]
struct Entry {
    val: u32,
    tag: Option<RStag>,
}

impl Default for Entry {
    fn default() -> Self {
        Self { val: 0, tag: None }
    }
}

#[cfg(test)]
mod regfile {
    use super::*;
    #[test]
    fn new() {
        let default_val = 0;
        let rf = RegisterFile::new();
        for e in rf.entry.iter() {
            assert_eq!(default_val, e.val);
            assert!(e.tag.is_none());
        }
    }
    #[test]
    fn write_match() {
        let mut rf = RegisterFile::new();
        let tag = RStag::new("name", 1);
        let write_val = 100;
        let to_write = [0, 10, 15];
        for idx in to_write.iter() {
            rf.entry[*idx].tag = Some(tag.clone());
        }

        rf.write(tag, write_val);

        for idx in to_write.iter() {
            let entry_ut = &rf.entry[*idx];
            assert_eq!(write_val, entry_ut.val);
            assert!(entry_ut.tag.is_none());
        }
    }
    #[test]
    fn write_not_match() {
        let mut rf = RegisterFile::new();
        let tag_set = RStag::new("name", 1);
        let tag_write = RStag::new("name", 2);
        let to_not_match = 5;
        let write_val = 100;
        rf.entry[to_not_match].tag = Some(tag_set);

        rf.write(tag_write, write_val);

        let entry_not_matched = &rf.entry[to_not_match];
        assert_eq!(0, entry_not_matched.val);
        assert!(entry_not_matched.tag.is_some());
    }
}

impl RegisterFile {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn read(&self, idx: usize) -> ArgState {
        let entry = &self.entry[idx];
        if let Some(tag) = entry.tag.as_ref() {
            ArgState::Waiting(tag.clone())
        } else {
            let val = entry.val;
            ArgState::Ready(val)
        }
    }
    pub fn write(&mut self, tag: RStag, val: u32) {
        for e in self.entry.as_mut() {
            if let Some(wait) = e.tag.as_ref() {
                if *wait == tag {
                    e.val = val;
                    e.tag = None;
                }
            }
        }
    }
    /// Rename register number `idx` with reservation station tag
    pub fn rename(&mut self, idx: usize, tag: RStag) {
        self.entry[idx].tag = Some(tag);
    }
    /// Dump content of registers as [String]
    pub fn dump(&self) -> Vec<String> {
        let size = self.entry.len();
        let mut ret = Vec::with_capacity(size);
        for reg in self.entry.iter() {
            let content = match reg.tag.as_ref() {
                Some(tag) => {
                    format!("{}", tag)
                }
                None => reg.val.to_string(),
            };
            ret.push(content);
        }
        ret
    }
}
