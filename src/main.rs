mod decoding;
mod context;
mod exec;
mod file;
mod instructions;

use std::env;

use exec::execute;
use decoding::decode;
use stopwatch::Stopwatch;

use crate::context::CpuContext;

fn main() {
    let mut context = CpuContext::create();

    println!("Args: {:?}", env::args().collect::<Vec<String>>());
    let file_name = env::args().skip(1).next();

    let file_name = match file_name {
        Some(v) => v,
        None => {
            eprintln!("File name required.");
            return;
        }
    };

    file::read_memory_from_file(&mut context, &file_name);

    let mut stopwatch = Stopwatch::start_new();

    while !context.is_halted() {
        let program_counter = context.get_program_counter();
        let word = context.read_word(program_counter);
        let instr = decode(word).unwrap();

        execute(&mut context, instr);
    }

    stopwatch.stop();

    println!("Registers:\n{}\n{}", context.debug_get_registers(), context.debug_get_status());

    println!("Took {} ns ({} ms) to execute.", stopwatch.elapsed().as_nanos(), stopwatch.elapsed().as_millis());
}
