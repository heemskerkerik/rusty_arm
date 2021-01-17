use std::mem::size_of;

use ux::{self, u4};

pub struct CpuContext {
    registers: [u32; 16],
    memory: Box<[u8]>,
    status: StatusFlags,
    halted: bool
}

#[derive(Copy, Clone)]
pub struct StatusFlags {
    pub negative: bool,
    pub zero: bool,
    pub carry: bool,
    pub overflow: bool,
}

const PROGRAM_COUNTER_REGISTER: u8 = 15;

impl CpuContext {
    pub fn create() -> CpuContext {
        let registers = [0u32; 16];
        let memory = [0u8; 0x10000];

        CpuContext {
            registers: registers,
            memory: Box::from(memory),
            status: StatusFlags { negative: false, zero: false, carry: false, overflow: false },
            halted: false
        }
    }

    pub fn get_program_counter_register() -> u4 {
        u4::new(PROGRAM_COUNTER_REGISTER)
    }

    pub fn get_program_counter(&self) -> u32 {
        self.get_register(CpuContext::get_program_counter_register())
    }

    pub fn set_program_counter(&mut self, value: u32) {
        self.set_register(CpuContext::get_program_counter_register(), value)
    }

    pub fn get_register(&self, register: u4) -> u32 {
        let register: u8 = register.into();
        self.registers[register as usize]
    }

    pub fn set_register(&mut self, register: u4, value: u32) {
        let register: u8 = register.into();
        self.registers[register as usize] = value
    }

    pub fn get_status(&self) -> StatusFlags {
        self.status
    }

    pub fn set_status(&mut self, negative: Option<bool>, zero: Option<bool>, carry: Option<bool>, overflow: Option<bool>) {
        match negative {
            Some(v) => self.status.negative = v,
            None => {}
        }
        match zero {
            Some(v) => self.status.zero = v,
            None => {}
        }
        match carry {
            Some(v) => self.status.carry = v,
            None => {}
        }
        match overflow {
            Some(v) => self.status.overflow = v,
            None => {}
        }
    }

    pub fn read_word(&self, address: u32) -> u32 {
        let start_address = address as usize;
        let end_address = start_address + size_of::<u32>();
        let slice = &self.memory[start_address..end_address];
        let pointer = slice.as_ptr();
        let pointer_u32 = pointer as *const u32;

        unsafe {
            *pointer_u32
        }
    }

    pub fn write_word(&mut self, address: u32, value: u32) {
        let start_address = address as usize;
        let end_address = start_address + size_of::<u32>();
        let slice = &mut self.memory[start_address..end_address];
        let pointer = slice.as_mut_ptr();
        let pointer_u32 = pointer as *mut u32;

        unsafe {
            *pointer_u32 = value
        }
    }

    pub fn is_halted(&self) -> bool {
        self.halted
    }

    pub fn halt(&mut self) {
        self.halted = true
    }

    pub fn debug_get_registers(&self) -> String {
        let mut result = String::new();

        for i in 0..15 {
            result.push_str(&format!("R{}: {:0>8X}\n", i, self.registers[i]))
        }

        result
    }

    pub fn debug_get_status(&self) -> String {
        let mut result = String::from("(NZCV) ");

        result.push(if self.status.negative { '1' } else { '0' });
        result.push(if self.status.zero { '1' } else { '0' });
        result.push(if self.status.carry { '1' } else { '0' });
        result.push(if self.status.overflow { '1' } else { '0' });

        result
    }
}