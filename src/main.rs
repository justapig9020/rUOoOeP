mod arthmatic_unit;
mod decoder;
mod display;
mod execution_path;
mod graph;
mod processor;
mod register;
mod result_bus;
use processor::Processor;
use std::io;

fn main() -> Result<(), String> {
    let program = vec![
        "addi R1, R0, #100",
        "addi R2, R0, #200",
        "add R3, R1, R2",
        "add R3, R1, R2",
        "add R3, R1, R2",
        "add R3, R1, R2",
        "add R3, R1, R2",
        "add R3, R1, R2",
        "add R3, R1, R2",
        "add R3, R1, R2",
        "add R3, R1, R2",
        "add R3, R1, R2",
        "add R3, R1, R2",
        "add R3, R1, R2",
        "add R3, R1, R2",
    ];
    let mut p = Processor::new();
    p.add_path("arth")?;
    loop {
        let line = p.fetching();
        if line >= program.len() {
            break;
        }
        let inst = program[line];
        println!("Line {}:", line);
        println!("{}", p);
        pause();
        p.next_cycle(inst)?;
    }
    println!("");
    println!("Emulation finished");
    println!("{:#?}", p);
    Ok(())
}

fn pause() {
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
}
