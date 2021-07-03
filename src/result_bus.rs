use crate::execution_path::{RStag, ExecResult};

#[derive(Debug)]
pub struct ResultBus {
    value: Option<(RStag, ExecResult)>,
}

impl ResultBus {
    pub fn new() -> Self {
        Self {
            value: None,
        }
    }
    pub fn set(&mut self, tag: RStag, result: ExecResult) {
        self.value = Some((tag, result));
    }
    pub fn take(&mut self) -> Option<(RStag, ExecResult)> {
        self.value.take()
    }
}
