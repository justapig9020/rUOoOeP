mod core;
mod display;
mod functional_units;
mod graph;
use crate::core::processor::Processor;
use crate::functional_units::execution_path_factory;
use std::io;

fn main() -> Result<(), String> {
    let program = vec![
        "addi R1, R0, #100", // R1 = 100
        "addi R2, R0, #200", // R2 = 200
        "add R3, R1, R2",    // R3 = 300
        "add R4, R1, R3",    // R4 = 400
        "add R3, R4, R3",    // R3 = 700
        "addi R1, R5, #400", // R1 = 400
        "add R5, R1, R2",    // R5 = 600
        /* R1: 400
         * R2: 200
         * R3: 700
         * R4: 400
         * R5: 600
         */
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
        "nop",
    ];
    let mut p = Processor::new();
    for _ in 0..2 {
        let unit = execution_path_factory("arth")?;
        p.add_path(unit)?;
    }
    loop {
        let line = p.fetch_address();
        let inst = if let Some(inst) = program.get(line) {
            inst
        } else {
            break;
        };
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
