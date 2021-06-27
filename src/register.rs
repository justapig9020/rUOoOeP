use crate::execution_path::RStag;
use std::default::Default;

#[derive(Default)]
pub struct RegFile {
    entry: [Entry; 16],
}

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
}

impl RegFile {
    pub fn new() -> Self {
        Default::default()
    }
}
