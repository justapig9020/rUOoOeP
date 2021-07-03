mod execution_path;
mod graph;
mod register;
mod decoder;
mod processor;
mod result_bus;
mod arthmatic_unit;
use processor::Processor;
use std::io;

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
        println!("Line {}:", line);
        println!("{:#?}", p);
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
