mod decoding;
mod context;
mod exec;

use exec::execute;
use stopwatch::Stopwatch;

use crate::context::CpuContext;

fn main() {
    let mut context = CpuContext::create();

    context.write_word(0x0, 0xe3a00000);
    context.write_word(0x4, 0xe2800001);
    context.write_word(0x8, 0xe3500064);
    context.write_word(0xc, 0x1afffffc);
    context.write_word(0x10, 0xeafffffe);

    let mut stopwatch = Stopwatch::start_new();

    while !context.is_halted() {
        let program_counter = context.get_program_counter();
        let word = context.read_word(program_counter);
        let instr = decoding::decode(word).unwrap();

        execute(&mut context, instr);
    }

    stopwatch.stop();

    println!("Registers:\n{}\n{}", context.debug_get_registers(), context.debug_get_status());

    println!("Took {} ns ({} ms) to execute.", stopwatch.elapsed().as_nanos(), stopwatch.elapsed().as_millis());
}
