use std::mem::size_of;

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

const LINK_RETURN_REGISTER: u8 = 14;
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

    pub fn write_memory(&mut self, data: &Vec<u8>) {
        self.memory[..data.len()].copy_from_slice(data)
    }

    pub const fn get_link_return_register() -> u8 {
        LINK_RETURN_REGISTER
    }

    pub const fn get_program_counter_register() -> u8 {
        PROGRAM_COUNTER_REGISTER
    }

    pub fn get_program_counter(&self) -> u32 {
        self.registers[PROGRAM_COUNTER_REGISTER as usize]
    }

    pub fn set_program_counter(&mut self, value: u32) {
        self.set_register(CpuContext::get_program_counter_register(), value)
    }

    pub fn get_register(&self, register: u8) -> u32 {
        assert!(register <= PROGRAM_COUNTER_REGISTER);
        let value = self.registers[register as usize];

        match register {
            PROGRAM_COUNTER_REGISTER => value + 4,  // instructions reading from PC will get PC + 8, but since PC has already been advanced by 4, we need to add only another 4
            _ => value,
        }
    }

    pub fn set_register(&mut self, register: u8, value: u32) {
        assert!(register <= PROGRAM_COUNTER_REGISTER);
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

    pub fn read_byte(&self, address: u32) -> u8 {
        let start_address = address as usize;
        let end_address = start_address + size_of::<u32>();
        let slice = &self.memory[start_address..end_address];
        let pointer = slice.as_ptr();

        unsafe {
            *pointer
        }
    }

    pub fn read_half_word(&self, address: u32) -> u16 {
        let start_address = address as usize;
        let end_address = start_address + size_of::<u32>();
        let slice = &self.memory[start_address..end_address];
        let pointer = slice.as_ptr();
        let pointer_u16 = pointer as *const u16;

        unsafe {
            *pointer_u16
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

    pub fn write_byte(&mut self, address: u32, value: u8) {
        let start_address = address as usize;
        let end_address = start_address + size_of::<u32>();
        let slice = &mut self.memory[start_address..end_address];
        let pointer = slice.as_mut_ptr();

        unsafe {
            *pointer = value
        }
    }

    pub fn write_half_word(&mut self, address: u32, value: u16) {
        let start_address = address as usize;
        let end_address = start_address + size_of::<u32>();
        let slice = &mut self.memory[start_address..end_address];
        let pointer = slice.as_mut_ptr();
        let pointer_u16 = pointer as *mut u16;

        unsafe {
            *pointer_u16 = value
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

        for i in 0..=15 {
            result.push_str(&format!("R{}: {:0>8X}\n", i, self.registers[i]))
        }

        result
    }

    pub fn debug_get_status(&self) -> String {
        let mut result = String::from("(NZCV) ");

        let status = self.status;

        result.push(if status.negative { '1' } else { '0' });
        result.push(if status.zero { '1' } else { '0' });
        result.push(if status.carry { '1' } else { '0' });
        result.push(if status.overflow { '1' } else { '0' });

        result
    }
}