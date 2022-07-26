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
use termion::terminal_size;

fn main() -> Result<(), String> {
    let program = vec![
        "addi R1, R0, #100",
        "muli R2, R1, #10",
        "addi R1, R0, #200",
        "add R3, R1, R2",
    ];

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
        println!("{}", tick_bar(tick));
        print!("{vm}");
        pause();
        result = vm.next_cycle();
        tick += 1;
    }

    result = Ok(());
    while result.is_ok() {
        result = vm.next_flush_cycle();
        println!("{}", tick_bar(tick));
        println!("{vm}");
        pause();
        tick += 1;
    }

    Ok(())
}

fn tick_bar(tick: usize) -> String {
    let tick = tick.to_string();
    let (col, _) = terminal_size().unwrap();
    let bar = col as usize - tick.len() - 2;
    let left = bar / 5;
    let right = bar - left;
    let make_bar = |len: usize| -> String {
        let mut bar = String::with_capacity(len);
        for _ in 0..len {
            bar.push('=');
        }
        bar
    };
    let left = make_bar(left);
    let right = make_bar(right);
    format!("{left} {tick} {right}")
}

fn pause() {
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
}
