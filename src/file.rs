use std::{fs, mem::size_of, path::Path};

use crate::context::*;

const ELF_ENTRY_POINT_OFFSET: usize = 0x18;
const ELF_ENTRY_POINT_SIZE: usize = size_of::<u32>();

pub fn read_memory_from_file(context: &mut CpuContext, path: &str) {
    let path = Path::new(path);
    let bytes = fs::read(path).unwrap();

    println!("Read {} bytes from {}", bytes.len(), path.file_name().unwrap().to_str().unwrap());

    context.write_memory(&bytes);

    let elf_magic: [u8; 8] = [ 0x7F, 0x45, 0x4C, 0x46, 0x01, 0x01, 0x01, 0x00 ];
    if bytes.starts_with(&elf_magic)
    && bytes.len() > ELF_ENTRY_POINT_OFFSET + ELF_ENTRY_POINT_SIZE {
        let entry_point = context.read_word(ELF_ENTRY_POINT_OFFSET as u32);

        println!("File is ELF; entry point offset is {:0>8X}", entry_point);
        context.set_program_counter(entry_point);
    }
}