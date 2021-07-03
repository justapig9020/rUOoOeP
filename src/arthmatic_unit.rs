use crate::execution_path::{ RStag, ExecPath, ArgVal };
use crate::decoder::{ InstFormat, InstFormatCreater , SyntaxType};
use crate::result_bus::ResultBus;

pub struct Unit {
    name: String,
}

impl ExecPath for Unit {
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_func(&self) -> String {
        String::from("arth")
    }
    fn list_inst(&self) -> Vec<InstFormat> {
        vec![
            InstFormat::create("add")
                .add_syntax(SyntaxType::Register)
                .add_syntax(SyntaxType::Register)
                .add_syntax(SyntaxType::Register)
                .set_writeback(true)
                .done(),
            InstFormat::create("addi")
                .add_syntax(SyntaxType::Register)
                .add_syntax(SyntaxType::Register)
                .add_syntax(SyntaxType::Immediate)
                .set_writeback(true)
                .done(),]
    }
    fn forwarding(&mut self, tag: RStag, val: u32) {

    }
    fn issue(&mut self, inst: String, vals:&[ArgVal]) -> Result<RStag, ()> {
        let tag = RStag::new(&self.name, 0);
        Ok(tag)
    }
    fn next_cycle(&mut self, bus: &mut ResultBus) {

    }
}

impl Unit {
    pub fn new() -> Self {
        static mut cnt: usize = 0;
        // Safety: the cnt will only be used in this function
        let idx = unsafe {
            cnt += 1;
            cnt - 1
        };
        Self {
            name: format!("arth{}", idx),
        }
    }
}
