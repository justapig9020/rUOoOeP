use std::fmt::Display;

use crate::display::into_table;

use super::execution_path::{ExecResult, RStag};

#[derive(Debug)]
pub struct ResultBus {
    value: Option<(RStag, ExecResult)>,
}

impl Display for ResultBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let info = self
            .value
            .as_ref()
            .map_or(String::new(), |(tag, result)| -> String {
                format!("{result:?} from {tag}")
            });
        write!(f, "{}", into_table("Result Bus", vec![info]))
    }
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
    pub fn is_free(&self) -> bool {
        self.value.is_none()
    }
}
