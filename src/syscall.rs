use std::io::{stdout, Write};

use crate::context::CpuContext;

pub fn execute_system_call(context: &mut CpuContext) {
    const SYSTEM_CALL_REGISTER: u8 = 7;
    let system_call = context.get_register(SYSTEM_CALL_REGISTER);

    match system_call {
        EXIT_SYSTEM_CALL => context.halt(),
        WRITE_SYSTEM_CALL => write(context),
        _ => panic!("Unsupported system call {:0>8X}", system_call)
    }
}

fn write(context: &CpuContext) {
    let file_descriptor = context.get_register(0);
    let address = context.get_register(1);

    assert!(file_descriptor == 1);

    let data = context.read_string(address);
    stdout().write(data.as_bytes()).unwrap();
}

const EXIT_SYSTEM_CALL: u32 = 0x1;
const WRITE_SYSTEM_CALL: u32 = 0x4;