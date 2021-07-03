use crate::execution_path::{RStag, ArgVal};
use std::default::Default;

#[derive(Default, Debug)]
pub struct RegFile {
    entry: [Entry; 16],
}

#[derive(Debug)]
struct Entry {
    val: u32,
    tag: Option<RStag>,
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            val: 0,
            tag: None,
        }
    }
}

#[cfg(test)]
mod regfile {
    use super::*;
    #[test]
    fn new() {
        let default_val = 0;
        let rf = RegFile::new();
        for e in rf.entry.iter() {
            assert_eq!(default_val, e.val);
            assert!(e.tag.is_none());
        }
    }
    #[test]
    fn write_match() {
        let mut rf = RegFile::new();
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
        let mut rf = RegFile::new();
        let tag_set = RStag::new("name", 1);
        let tag_write = RStag::new("name", 2);
        let to_not_match = 5;
        let write_val = 100;
        rf.entry[to_not_match].tag
            = Some(tag_set);

        rf.write(tag_write, write_val);

        let entry_not_matched = &rf.entry[to_not_match];
        assert_eq!(0, entry_not_matched.val);
        assert!(entry_not_matched.tag.is_some());
    }
}

impl RegFile {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn read(&self, idx: usize) -> ArgVal {
        let entry = &self.entry[idx];
        if let Some(tag) = entry.tag.as_ref() {
            ArgVal::Waiting(tag.clone())
        } else {
            let val = entry.val;
            ArgVal::Ready(val)
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
    pub fn rename(&mut self, idx: usize, tag: RStag) {
        self.entry[idx].tag = Some(tag);
    }
}
