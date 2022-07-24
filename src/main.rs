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
    let program = vec!["addi R1, R0, #100", "addi R1, R1, #200", "add R2, R1, R1"];

    let program = program.iter().map(|i| i.to_string()).collect();

    let mut p = Processor::new();
    let mut ff = Factory::new();
    for _ in 0..2 {
        let unit = ff.new_unit(Function::Arithmetic);
        p.add_path(unit)?;
    }
    let mut vm = virtual_machine::Machine::new(p, program, 20);

    let mut tick = 0;
    let mut result = Ok(());
    while result.is_ok() {
        tick += 1;
        println!("================ {tick} ================");
        print!("{vm}");
        pause();
        result = vm.next_cycle();
    }

    result = Ok(());
    while result.is_ok() {
        result = vm.next_flush_cycle();
        println!("================ {tick} ================");
        println!("{vm}");
        pause();
        tick += 1;
    }

    Ok(())
}

fn pause() {
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
}
