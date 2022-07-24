#![feature(trait_upcasting)]
mod core;
mod display;
mod functional_units;
mod graph;
mod memory_bus;
mod util;
mod virtual_machine;
use crate::core::processor::Processor;
use crate::functional_units::factory::{Factory, Function, MemFunction};
use std::io;

fn main() -> Result<(), String> {
    let program = vec![
        "addi R1, R0, #0",
        "addi R2, R0, #10",
        "sw R1, R2, #0", // j = 0, &j == 10
        "sw R1, R2, #4", // k = 0, &k == 14
        "addi R3, R0, #4",
        "addi R4, R0, #5",
        // First iteration
        "lw R1, R2, #0",
        "add R1, R3, R1", // j += 4
        "sw R1, R2, #0",
        "lw R1, R2, #4",
        "add R1, R4, R1", // k += 5
        "sw R1, R2, #4",
        // Second iteration
        "lw R1, R2, #0",
        "add R1, R3, R1", // j += 4
        "sw R1, R2, #0",
        "lw R1, R2, #4",
        "add R1, R4, R1", // k += 5
        "sw R1, R2, #4",
        // Third iteration
        "lw R1, R2, #0",
        "add R1, R3, R1", // j += 4
        "sw R1, R2, #0",
        "lw R1, R2, #4",
        "add R1, R4, R1", // k += 5
        "sw R1, R2, #4",
    ];

    let program = program.iter().map(|i| i.to_string()).collect();

    let mut p = Processor::new();
    let mut ff = Factory::new();
    for _ in 0..2 {
        let unit = ff.new_unit(Function::Arithmetic);
        p.add_path(unit)?;
    }
    let mut vm = virtual_machine::Machine::new(p, program, 20);

    let mut result = Ok(());
    while result.is_ok() {
        println!("{}", vm);
        pause();
        result = vm.next_cycle();
    }

    result = Ok(());
    while result.is_ok() {
        println!("{}", vm);
        pause();
        result = vm.next_flush_cycle();
    }

    println!("{:?}", result);

    let (p, dram) = vm.splite();
    println!("Emulation finished");
    println!("{:#?}", p);
    println!("{:?}", dram);
    Ok(())
}

fn pause() {
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
}
