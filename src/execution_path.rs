use crate::graph::Graph;

pub enum RegVal {
    Waiting(String),
    Ready(u32),
}

pub struct RStag {
    name: String,
    slot: usize,
}

pub enum ExecResult {
    Arth(u32),
}

pub trait ExecPath: Graph {
    fn issue(&mut self, inst: &str, vals: Vec<RegVal>) -> Result<RStag, ()>;
    fn next_cycle(&mut self);
    fn get_result(&mut self) -> Option<(RStag, ExecResult)>;
}

pub fn execution_path_factory(name: &str) -> Box<dyn ExecPath>{
    todo!();
}
