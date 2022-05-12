use super::arthmatic_unit;
use crate::core::execution_path::ExecPath;
use std::collections::HashMap;

#[derive(Hash, std::cmp::PartialEq, std::cmp::Eq, Clone, Copy)]
pub enum Function {
    Arthmatic,
}

/// A factory used to construct functional units
pub struct Factory {
    index: HashMap<Function, usize>,
}

impl Factory {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }
    /// Generate a execution path by function type
    pub fn new_unit(&mut self, func: Function) -> Box<dyn ExecPath> {
        use Function::*;
        let index = if let Some(i) = self.index.get_mut(&func) {
            *i += 1;
            *i
        } else {
            self.index.insert(func, 0);
            0
        };
        match func {
            Arthmatic => Box::new(arthmatic_unit::Unit::new(index)),
        }
    }
}

#[test]
fn factory_new_units() {
    let mut ff = Factory::new();
    let units: Vec<Box<dyn ExecPath>> = (0..10).map(|_| ff.new_unit(Function::Arthmatic)).collect();
    for (i, u) in units.iter().enumerate() {
        let expect = format!("{}{}", u.function(), i);
        assert_eq!(expect, u.name());
    }
}
