#![feature(trait_upcasting)]
mod core;
mod display;
mod functional_units;
mod graph;
mod virtual_machine;
use crate::core::processor::Processor;
use crate::functional_units::factory::{Factory, Function};
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

    let program = program.iter().map(|i| i.to_string()).collect();

    let mut p = Processor::new();
    let mut ff = Factory::new();
    for _ in 0..2 {
        let unit = ff.new_unit(Function::Arthmatic);
        p.add_path(unit)?;
    }
    let mut vm = virtual_machine::Machine::new(p, program, 0);

    while vm.next_cycle().is_ok() {
        println!("{}", vm);
        pause();
    }
    let p = vm.into_processor();
    println!("Emulation finished");
    println!("{:#?}", p);
    Ok(())
}

fn pause() {
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
}
