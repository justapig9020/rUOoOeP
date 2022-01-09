use crate::execution_path::{ExecResult, RStag};

#[derive(Debug)]
pub struct ResultBus {
    value: Option<(RStag, ExecResult)>,
}

impl ResultBus {
    pub fn new() -> Self {
        Self { value: None }
    }
    pub fn set(&mut self, tag: RStag, result: ExecResult) -> bool {
        if self.value.is_none() {
            self.value = Some((tag, result));
            true
        } else {
            false
        }
    }
    pub fn take(&mut self) -> Option<(RStag, ExecResult)> {
        self.value.take()
    }
}
