use core::panic;

use ux::{self, u12, u4, u5};

pub type Register = ux::u4;

#[derive(Debug)]
pub enum Condition {
    Always,
    NotEqual,
    Equal,
}

#[derive(Debug)]
pub enum ShiftOperand {
    Immediate(ux::u5),
    Register(Register),
}

#[derive(Debug)]
pub enum ShiftType {
    LogicalShiftLeft,
    LogicalShiftRight,
    ArithmeticShiftRight,
    RotateRight,
}

#[readonly::make]
#[derive(Debug)]
pub struct ReadWriteImmediateDataArguments {
    pub source_register: Register,
    pub destination_register: Register,
    pub immediate: u32,
    pub carry: bool,
    pub rotate: u8,
}

#[readonly::make]
#[derive(Debug)]
pub struct ReadWriteRegisterDataArguments {
    pub source_register: Register,
    pub destination_register: Register,
    pub operand_register: Register,
    pub shift_type: ShiftType,
    pub shift_operand: ShiftOperand,
}

#[derive(Debug)]
pub enum ReadWriteDataArguments {
    Immediate(ReadWriteImmediateDataArguments),
    Register(ReadWriteRegisterDataArguments),
}

#[readonly::make]
#[derive(Debug)]
pub struct ImmediateDataArguments {
    pub register: Register,
    pub immediate: u32,
    pub carry: bool,
    pub rotate: u8,
}

#[readonly::make]
#[derive(Debug)]
pub struct RegisterDataArguments {
    pub register: Register,
    pub operand_register: Register,
    pub shift_type: ShiftType,
    pub shift_operand: ShiftOperand,
}

#[derive(Debug)]
pub enum DataArguments {
    Immediate(ImmediateDataArguments),
    Register(RegisterDataArguments),
}

#[derive(Debug)]
pub enum UpdateStatusFlags {
    DoNotUpdateStatusFlags,
    UpdateStatusFlags,
}

#[derive(Debug)]
pub enum BranchLinkFlag {
    LinkReturnAddress,
    DoNotLinkReturnAddress,
}

#[derive(Debug)]
pub enum InstructionData {
    Add(ReadWriteDataArguments, UpdateStatusFlags),
    //AddWithCarry(ReadWriteDataArguments, UpdateStatusFlags),
    //And(ReadWriteDataArguments, UpdateStatusFlags),
    Branch(i32, BranchLinkFlag),
    Compare(DataArguments),
    Move(DataArguments, UpdateStatusFlags),
}

pub type Instruction = (Condition, InstructionData);

pub fn decode(encoded_instruction: u32) -> Result<Instruction, String> {
    let condition = decode_condition(encoded_instruction);
    let instruction_class = encoded_instruction & INSTRUCTION_CLASS_MASK;

    match instruction_class {
        BRANCH_INSTRUCTION_CLASS => Ok((condition, decode_branch(encoded_instruction))),
        DATA_PROCESSING_IMMEDIATE_INSTRUCTION_CLASS | DATA_PROCESSING_REGISTER_INSTRUCTION_CLASS => {
            let instruction_data = decode_data_processing_instruction(encoded_instruction);

            match instruction_data {
                Ok(data) => Ok((condition, data)),
                Err(e) => Err(e),
            }
        }
        _ => {
            Err(format!("Unknown instruction {:0>8X}", encoded_instruction))
        }
    }
}

fn decode_condition(encoded_instruction: u32) -> Condition {
    let masked_condition = encoded_instruction & CONDITION_MASK;
    let condition_byte = (masked_condition >> 28) as u8;

    match condition_byte {
        EQUAL_CONDITION => Condition::Equal,
        NOT_EQUAL_CONDITION => Condition::NotEqual,
        ALWAYS_CONDITION => Condition::Always,
        _ => Condition::Always,
    }
}

fn decode_data_processing_instruction(encoded_instruction: u32) -> Result<InstructionData, String> {
    let update_status_flag = if (encoded_instruction & UPDATE_STATUS_BIT) != 0 {
        UpdateStatusFlags::UpdateStatusFlags
    } else {
        UpdateStatusFlags::DoNotUpdateStatusFlags
    };
    let opcode = encoded_instruction & OPCODE_MASK;

    match opcode {
        ADD_OPCODE => Ok(InstructionData::Add(decode_read_write_arguments(encoded_instruction), update_status_flag)),
        MOVE_OPCODE => Ok(InstructionData::Move(decode_write_arguments(encoded_instruction), update_status_flag)),
        COMPARE_OPCODE => Ok(InstructionData::Compare(decode_read_arguments(encoded_instruction))),
        _ => Err(format!("Unknown opcode {:0>2X}", opcode >> 21))
    }
}

fn decode_branch(encoded_instruction: u32) -> InstructionData {
    let destination_address = (encoded_instruction & 0x00ffffff) as i32;
    let sign_extended_destination_address = if destination_address & 0x00800000 != 0 { destination_address | 0x3f000000 } else { destination_address };
    let shifted_destination_address = sign_extended_destination_address << 2;
    let adjusted_destination_address = shifted_destination_address + 8; // addresses are encoded as relative to PC + 8

    let link_flag = if encoded_instruction & 0x01000000 != 0 { BranchLinkFlag::LinkReturnAddress } else { BranchLinkFlag::DoNotLinkReturnAddress };

    InstructionData::Branch(adjusted_destination_address, link_flag)
}

fn decode_write_arguments(encoded_instruction: u32) -> DataArguments {
    let immediate_mode = encoded_instruction & IMMEDIATE_MODE_BIT != 0;
    let register: Register = u4::new(((encoded_instruction & 0x0000f000) >> 12) as u8);

    if immediate_mode {
        let (immediate, carry, rotate) = decode_shifted_immediate(encoded_instruction);

        DataArguments::Immediate(
            ImmediateDataArguments {
                register,
                immediate,
                carry,
                rotate
            }
        )
    } else {
        let (operand_register, shift_type, shift_operand) = decode_register_shift_arguments(encoded_instruction);

        DataArguments::Register(
            RegisterDataArguments {
                register,
                operand_register,
                shift_type,
                shift_operand
            }
        )
    }
}

fn decode_shifted_immediate(encoded_instruction: u32) -> (u32, bool, u8) {
    let rotate = ((encoded_instruction & 0x00000f00) >> 8) as u8;
    let immediate = (encoded_instruction & 0x000000ff) as u32;

    let immediate = immediate.wrapping_shr(rotate.into());
    let carry = immediate & 0x80000000 != 0;

    (immediate, carry, rotate)
}

fn decode_read_arguments(encoded_instruction: u32) -> DataArguments {
    let immediate_mode = encoded_instruction & IMMEDIATE_MODE_BIT != 0;

    let register: Register = u4::new(((encoded_instruction & 0x000f0000) >> 16) as u8);

    if immediate_mode {
        let (immediate, carry, rotate) = decode_shifted_immediate(encoded_instruction);

        DataArguments::Immediate(
            ImmediateDataArguments {
                register,
                immediate,
                carry,
                rotate,
            }
        )
    } else {
        let (operand_register, shift_type, shift_operand) = decode_register_shift_arguments(encoded_instruction);

        DataArguments::Register(
            RegisterDataArguments {
                register,
                operand_register,
                shift_type,
                shift_operand,
            }
        )
    }
}

fn decode_read_write_arguments(encoded_instruction: u32) -> ReadWriteDataArguments {
    let immediate_mode = encoded_instruction & IMMEDIATE_MODE_BIT != 0;

    if immediate_mode {
        let source_register: Register = u4::new(((encoded_instruction & 0x000f0000) >> 16) as u8);
        let destination_register: Register = u4::new(((encoded_instruction & 0x0000f000) >> 12) as u8);
        let (immediate, carry, rotate) = decode_shifted_immediate(encoded_instruction);

        ReadWriteDataArguments::Immediate(
            ReadWriteImmediateDataArguments {
                source_register,
                destination_register,
                immediate,
                carry,
                rotate,
            }
        )
    } else {
        let source_register: Register = u4::new(((encoded_instruction & 0x000f0000) >> 16) as u8);
        let destination_register: Register = u4::new(((encoded_instruction & 0x0000f000) >> 12) as u8);
        let (operand_register, shift_type, shift_operand) = decode_register_shift_arguments(encoded_instruction);

        ReadWriteDataArguments::Register(
            ReadWriteRegisterDataArguments {
                source_register,
                destination_register,
                operand_register,
                shift_type,
                shift_operand,
            }
        )
    }
}

fn decode_register_shift_arguments(encoded_instruction: u32) -> (Register, ShiftType, ShiftOperand) {
    let operand_register: Register = u4::new((encoded_instruction & 0x0000000f) as u8);
    let shift_type = (encoded_instruction & SHIFT_TYPE_MASK) as u8;
    let immediate_shift = encoded_instruction & SHIFT_IMMEDIATE_BIT != 0;

    let shift_operand = match immediate_shift {
        true => {
            let immediate = ((encoded_instruction & 0x00000f80) >> 7) as u8;
            ShiftOperand::Immediate(u5::new(immediate))
        },
        false => {
            let register = ((encoded_instruction & 0x00000f00) >> 8) as u8;
            ShiftOperand::Register(u4::new(register))
        }
    };

    let shift_type= match shift_type {
        SHIFT_TYPE_LOGICAL_SHIFT_LEFT => ShiftType::LogicalShiftLeft,
        SHIFT_TYPE_LOGICAL_SHIFT_RIGHT => ShiftType::LogicalShiftRight,
        SHIFT_TYPE_ARITHMETIC_SHIFT_RIGHT => ShiftType::ArithmeticShiftRight,
        SHIFT_TYPE_ROTATE_RIGHT => ShiftType::RotateRight,
        _ => panic!("Unknown shift type {:0>2X}", shift_type),
    };

    (operand_register, shift_type, shift_operand)
}

const EQUAL_CONDITION: u8 = 0x0;
const NOT_EQUAL_CONDITION: u8 = 0x1;
const ALWAYS_CONDITION: u8 = 0xe;
const CONDITION_MASK: u32 = 0xf0000000;
const INSTRUCTION_CLASS_MASK: u32 = 0x0e000000;
const BRANCH_INSTRUCTION_CLASS: u32 = 0x0a000000;
const DATA_PROCESSING_REGISTER_INSTRUCTION_CLASS: u32 = 0x00000000;
const DATA_PROCESSING_IMMEDIATE_INSTRUCTION_CLASS: u32 = 0x02000000;
const LOAD_STORE_IMMEDIATE_INSTRUCTION_CLASS: u32 = 0x04000000;
const UPDATE_STATUS_BIT: u32 = 0x00100000;
const IMMEDIATE_MODE_BIT: u32 = 0x02000000;
const OPCODE_MASK: u32 = 0x01e00000;

const ADD_OPCODE: u32 = 0x00800000;
const MOVE_OPCODE: u32 = 0x01a00000;
const COMPARE_OPCODE: u32 = 0x01400000;

const SHIFT_TYPE_LOGICAL_SHIFT_LEFT: u8 =       0b0000000;
const SHIFT_TYPE_LOGICAL_SHIFT_RIGHT: u8 =      0b0100000;
const SHIFT_TYPE_ARITHMETIC_SHIFT_RIGHT: u8 =   0b1000000;
const SHIFT_TYPE_ROTATE_RIGHT: u8 =             0b1100000;
const SHIFT_TYPE_MASK: u32 = 0x00000060;
const SHIFT_IMMEDIATE_BIT: u32 = 0x00000010;