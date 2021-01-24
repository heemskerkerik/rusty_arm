mod decoding;
mod context;
mod exec;
mod file;
mod instructions;

use std::{env, ops::RangeInclusive};

use exec::execute;
use decoding::decode;
use stopwatch::Stopwatch;

use crate::context::CpuContext;

fn main() {
    let mut context = CpuContext::create();

    let file_name = env::args().skip(1).next();

    let file_name = match file_name {
        Some(v) => v,
        None => {
            eprintln!("File name required.");
            return;
        }
    };

    file::read_memory_from_file(&mut context, &file_name);

    let breakpoints = [];
    let memory_ranges: &[RangeInclusive<u32>] = &[];

    let mut stopwatch = Stopwatch::start_new();

    while !context.is_halted() {
        let program_counter = context.get_program_counter();
        let word = context.read_word(program_counter);
        let instr = decode(word).unwrap();

        if cfg!(feature = "breakpoints") && breakpoints.contains(&program_counter) {
            println!("Breakpoint: {:0>8X}\nRegisters:\n{}\n{}", program_counter, context.debug_get_registers(), context.debug_get_status());
        }

        execute(&mut context, instr);

        if cfg!(feature = "memory_watch") {
            for range in memory_ranges.iter() {
                println!("{:0>8X}..{:0>8X} = {}", range.start(), range.end(), context.debug_get_memory_range(range));
            }
        }
    }

    stopwatch.stop();

    println!("Registers:\n{}\n{}", context.debug_get_registers(), context.debug_get_status());

    println!("Took {} ns ({} ms) to execute.", stopwatch.elapsed().as_nanos(), stopwatch.elapsed().as_millis());
}
