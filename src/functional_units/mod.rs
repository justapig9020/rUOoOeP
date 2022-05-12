mod arthmatic_unit;
mod reservation_station;

use crate::core::execution_path::ExecPath;

/// Generate a execution path by name of functional unit
pub fn execution_path_factory(func: &str) -> Result<Box<dyn ExecPath>, String> {
    match func {
        "arth" => {
            let unit = Box::new(arthmatic_unit::Unit::new());
            let unit = unit as Box<dyn ExecPath>;
            Ok(unit)
        }
        _ => {
            let msg = format!("Not support function unit {}", func);
            Err(msg)
        }
    }
}
