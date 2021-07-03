mod execution_path;
mod graph;
mod register;
mod decoder;
mod processor;
mod result_bus;
mod arthmatic_unit;
use processor::Processor;

const program: [&str; 3] = [
    "addi R1, R0, #100",
    "addi R2, R0, #200",
    "add R3, R1, R2",
];

fn main() -> Result<(), String> {
    let mut p = Processor::new();
    p.add_path("arth")?;
    loop {
        let line = p.fetching();
        if line >= program.len() {
            break;
        }
        let inst = program[line];
        p.next_cycle(inst)?;
    }
    println!("Emulation finished");
    Ok(())
}
