use core::panic;
use instructions::*;

use ux::{self, u12, u24, u4, u5};

use crate::instructions;

pub fn decode(encoded_instruction: u32) -> Result<Instruction, String> {
    let condition = decode_condition(encoded_instruction);
    let instruction_class = encoded_instruction & INSTRUCTION_CLASS_MASK;

    match instruction_class {
        BRANCH_INSTRUCTION_CLASS => Ok((condition, decode_branch(encoded_instruction))),
        DATA_PROCESSING_IMMEDIATE_INSTRUCTION_CLASS => {
            let extra_instructions = encoded_instruction & DATA_PROCESSING_IMMEDIATE_EXTRA_INSTRUCTIONS_MASK == DATA_PROCESSING_IMMEDIATE_EXTRA_INSTRUCTIONS_VALUE;
            let data = if !extra_instructions { decode_data_processing_instruction(encoded_instruction)? } else { decode_extra_data_processing_immediate_instruction(encoded_instruction)? };

            Ok((condition, data))
        },
        DATA_PROCESSING_REGISTER_INSTRUCTION_CLASS => {
            let extra_loads_stores = encoded_instruction & EXTRA_LOAD_STORES_FLAG == EXTRA_LOAD_STORES_FLAG;

            if !extra_loads_stores {
                let data = decode_data_processing_instruction(encoded_instruction)?;
                Ok((condition, data))
            } else {
                let data = decode_extra_load_store(encoded_instruction)?;
                Ok((condition, data))
            }
        },
        LOAD_STORE_IMMEDIATE_INSTRUCTION_CLASS | LOAD_STORE_REGISTER_INSTRUCTION_CLASS => {
            let data = decode_regular_load_store(encoded_instruction);

            Ok((condition, data))
        },
        SUPERVISOR_CALL_INSTRUCTION_CLASS if (encoded_instruction & 0x0f000000) == 0x0f000000 => {
            let immediate = u24::new(encoded_instruction & 0x00ffffff);
            Ok((condition, InstructionData::SupervisorCall(immediate)))
        },
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
        CARRY_SET_CONDITION => Condition::CarrySet,
        CARRY_CLEAR_CONDITION => Condition::CarryClear,
        NEGATIVE_CONDITION => Condition::Negative,
        POSITIVE_CONDITION => Condition::Positive,
        OVERFLOW_CONDITION => Condition::Overflow,
        NO_OVERFLOW_CONDITION => Condition::NoOverflow,
        UNSIGNED_HIGHER_CONDITION => Condition::UnsignedHigher,
        UNSIGNED_LOWER_OR_SAME_CONDITION => Condition::UnsignedLowerOrSame,
        GREATER_THAN_OR_EQUAL_CONDITION => Condition::GreaterThanOrEqual,
        LESS_THAN_CONDITION => Condition::LessThan,
        GREATER_THAN_CONDITION => Condition::GreaterThan,
        LESS_THAN_OR_EQUAL_CONDITION => Condition::LessThanOrEqual,
        ALWAYS_CONDITION => Condition::Always,
        _ => panic!("Unknown condition {:0>2X} (instruction: {:0>8X})", condition_byte, encoded_instruction),
    }
}

fn decode_data_processing_instruction(encoded_instruction: u32) -> Result<InstructionData, String> {
    let update_status_flag = if (encoded_instruction & UPDATE_STATUS_BIT) != 0 {
        UpdateStatusFlags::UpdateStatusFlags
    } else {
        UpdateStatusFlags::DoNotUpdateStatusFlags
    };
    let opcode = ((encoded_instruction & OPCODE_MASK) >> 21) as u8;

    match opcode {
        ADD_OPCODE => Ok(InstructionData::Add(decode_read_write_arguments(encoded_instruction), update_status_flag)),
        ADD_WITH_CARRY_OPCODE => Ok(InstructionData::AddWithCarry(decode_read_write_arguments(encoded_instruction), update_status_flag)),
        AND_OPCODE => Ok(InstructionData::And(decode_read_write_arguments(encoded_instruction), update_status_flag)),
        BRANCH_EXCHANGE_OPCODE => Ok(InstructionData::BranchExchange(decode_branch_exchange_arguments(encoded_instruction))),
        COMPARE_OPCODE => Ok(InstructionData::Compare(decode_read_arguments(encoded_instruction))),
        MOVE_OPCODE => Ok(InstructionData::Move(decode_write_arguments(encoded_instruction), update_status_flag)),
        MOVE_NOT_OPCODE => Ok(InstructionData::MoveNot(decode_write_arguments(encoded_instruction), update_status_flag)),
        OR_OPCODE => Ok(InstructionData::Or(decode_read_write_arguments(encoded_instruction), update_status_flag)),
        SUBTRACT_OPCODE => Ok(InstructionData::Subtract(decode_read_write_arguments(encoded_instruction), update_status_flag)),
        _ => Err(format!("Unknown data processing opcode {:0>2X} (instruction: {:0>8X})", opcode, encoded_instruction))
    }
}

fn decode_extra_data_processing_immediate_instruction(encoded_instruction: u32) -> Result<InstructionData, String> {
    let opcode = ((encoded_instruction & OPCODE_MASK) >> 21) as u8;

    match opcode {
        MOVE_HALFWORD_OPCODE => Ok(InstructionData::MoveHalfWord(decode_large_immediate_arguments(encoded_instruction))),
        MOVE_HALFWORD_TOP_OPCODE => Ok(InstructionData::MoveHalfWordTop(decode_large_immediate_arguments(encoded_instruction))),
        _ => Err(format!("Unknown extra data processing opcode {:0>2X} (instruction: {:0>8X})", opcode, encoded_instruction))
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

fn decode_regular_load_store(encoded_instruction: u32) -> InstructionData {
    let immediate_mode = encoded_instruction & 0x02000000 == 0;
    let indexing_type = if encoded_instruction & 0x01000000 != 0 { LoadStoreIndexingType::PreIndexed } else { LoadStoreIndexingType::PostIndexed };
    let offset_direction = if encoded_instruction & 0x00800000 != 0 { LoadStoreOffsetDirection::Positive } else { LoadStoreOffsetDirection::Negative };
    let data_size_is_byte = encoded_instruction & 0x00400000 != 0;
    let write_back = if encoded_instruction & 0x00200000 != 0 { LoadStoreWriteBackFlag::WriteBack } else { LoadStoreWriteBackFlag::DoNotWriteBack };
    let load_operation = encoded_instruction & 0x00100000 != 0;

    // if post-indexing is used, the write-back bit will be zero, but write-back should still be enabled
    let write_back = if let LoadStoreIndexingType::PostIndexed = indexing_type { LoadStoreWriteBackFlag::WriteBack } else { write_back };

    let address_register: Register = u4::new(((encoded_instruction & 0x000f0000) >> 16) as u8);
    let value_register: Register = u4::new(((encoded_instruction & 0x0000f000) >> 12) as u8);

    let offset = if immediate_mode {
        let immediate = u12::new((encoded_instruction & 0x00000fff) as u16);
        LoadStoreOffset::Immediate(immediate)
    } else {
        let shift_type = ((encoded_instruction & 0x00000060) >> 5) as u8;

        let shift_type = match shift_type {
            0b00 => ShiftType::LogicalShiftLeft,
            0b01 => ShiftType::LogicalShiftRight,
            0b10 => ShiftType::ArithmeticShiftRight,
            0b11 => ShiftType::RotateRight,
            _ => panic!("Impossible shift type {}", shift_type),
        };
        let shift_operand = u5::new(((encoded_instruction & 0x00000f80) >> 7) as u8);
        let register = u4::new((encoded_instruction & 0x0000000f) as u8);

        let shift_operand = match shift_type {
            ShiftType::LogicalShiftLeft => shift_operand,
            _ if shift_operand == u5::new(0) => u5::new(32),
            _ => shift_operand,
        };

        LoadStoreOffset::Register(
            LoadStoreRegisterOffset {
                register,
                shift_type,
                shift_operand,
            }
        )
    };

    let common_arguments = LoadStoreArguments {
        indexing_type,
        offset_direction,
        write_back,
        value_register,
        address_register,
        offset
    };

    if load_operation {
        InstructionData::Load(LoadArguments {
            data_size: if data_size_is_byte { LoadDataSize::Byte } else { LoadDataSize::Word },
            common_arguments
        })
    } else {
        InstructionData::Store(StoreArguments {
            data_size: if data_size_is_byte { StoreDataSize::Byte } else { StoreDataSize::Word },
            common_arguments
        })
    }
}

fn decode_extra_load_store(encoded_instruction: u32) -> Result<InstructionData, String> {
    let indexing_type = if encoded_instruction & 0x01000000 != 0 { LoadStoreIndexingType::PreIndexed } else { LoadStoreIndexingType::PostIndexed };
    let offset_direction = if encoded_instruction & 0x00800000 != 0 { LoadStoreOffsetDirection::Positive } else { LoadStoreOffsetDirection::Negative };
    let immediate_mode = encoded_instruction & 0x00400000 != 0;
    let write_back = if encoded_instruction & 0x00200000 != 0 { LoadStoreWriteBackFlag::WriteBack } else { LoadStoreWriteBackFlag::DoNotWriteBack };
    let load_operation = encoded_instruction & 0x00100000 != 0;
    let data_size = (encoded_instruction & 0x00000060) >> 5;

    // if post-indexing is used, the write-back bit will be zero, but write-back should still be enabled
    let write_back = if let LoadStoreIndexingType::PostIndexed = indexing_type { LoadStoreWriteBackFlag::WriteBack } else { write_back };

    let address_register: Register = u4::new(((encoded_instruction & 0x000f0000) >> 16) as u8);
    let value_register: Register = u4::new(((encoded_instruction & 0x0000f000) >> 12) as u8);

    let offset = if immediate_mode {
        let immediate_low = (encoded_instruction & 0x0000000f) as u8;
        let immediate_high = ((encoded_instruction & 0x00000f00) >> 4) as u8;
        let immediate = u12::new((immediate_low | immediate_high) as u16);
        LoadStoreOffset::Immediate(immediate)
    } else {
        let register = u4::new((encoded_instruction & 0x0000000f) as u8);

        LoadStoreOffset::Register(
            LoadStoreRegisterOffset {
                register,
                shift_type: ShiftType::LogicalShiftLeft,
                shift_operand: u5::new(0),
            }
        )
    };

    let common_arguments = LoadStoreArguments {
        indexing_type,
        offset_direction,
        write_back,
        value_register,
        address_register,
        offset
    };

    return match (load_operation, data_size) {
        (false, 0b01) => get_store_instruction(StoreDataSize::HalfWord, common_arguments),
        (false, 0b10) => get_load_instruction(LoadDataSize::DoubleWord, common_arguments),
        (false, 0b11) => get_store_instruction(StoreDataSize::DoubleWord, common_arguments),
        (true, 0b01) => get_load_instruction(LoadDataSize::UnsignedHalfWord, common_arguments),
        (true, 0b10) => get_load_instruction(LoadDataSize::SignedByte, common_arguments),
        (true, 0b11) => get_load_instruction(LoadDataSize::SignedHalfWord, common_arguments),
        _ => Err(String::from("Unrecognized data size")),
    };

    fn get_load_instruction(data_size: LoadDataSize, args: LoadStoreArguments) -> Result<InstructionData, String> {
        Ok(InstructionData::Load(LoadArguments {
            data_size,
            common_arguments: args
        }))
    }

    fn get_store_instruction(data_size: StoreDataSize, args: LoadStoreArguments) -> Result<InstructionData, String> {
        Ok(InstructionData::Store(StoreArguments {
            data_size,
            common_arguments: args
        }))
    }
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
    // rotate is encoded as rotate / 2, so this is >> 8, << 1
    let rotate = ((encoded_instruction & 0x00000f00) >> 7) as u8;
    let immediate = (encoded_instruction & 0x000000ff) as u32;

    let immediate = immediate.rotate_right(rotate as u32);
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
    let immediate_shift = encoded_instruction & SHIFT_IMMEDIATE_BIT == 0;

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
        _ => panic!("Unknown shift type {:0>2X} (instruction: {:0>8X})", shift_type, encoded_instruction),
    };

    (operand_register, shift_type, shift_operand)
}

fn decode_large_immediate_arguments(encoded_instruction: u32) -> LargeImmediateArguments {
    let register: Register = u4::new(((encoded_instruction & 0x0000f000) >> 12) as u8);
    let immediate_high = ((encoded_instruction & 0x000f0000) >> 4) as u16;
    let immediate_low = (encoded_instruction & 0x00000fff) as u16;

    LargeImmediateArguments {
        register,
        immediate: immediate_low | immediate_high,
    }
}

fn decode_branch_exchange_arguments(encoded_instruction: u32) -> Register {
    u4::new((encoded_instruction & 0x0000000f) as u8)
}

const EQUAL_CONDITION: u8 = 0x0;
const NOT_EQUAL_CONDITION: u8 = 0x1;
const CARRY_SET_CONDITION: u8 = 0x2;
const CARRY_CLEAR_CONDITION: u8 = 0x3;
const NEGATIVE_CONDITION: u8 = 0x4;
const POSITIVE_CONDITION: u8 = 0x5;
const OVERFLOW_CONDITION: u8 = 0x6;
const NO_OVERFLOW_CONDITION: u8 = 0x7;
const UNSIGNED_HIGHER_CONDITION: u8 = 0x8;
const UNSIGNED_LOWER_OR_SAME_CONDITION: u8 = 0x9;
const GREATER_THAN_OR_EQUAL_CONDITION: u8 = 0xa;
const LESS_THAN_CONDITION: u8 = 0xb;
const GREATER_THAN_CONDITION: u8 = 0xc;
const LESS_THAN_OR_EQUAL_CONDITION: u8 = 0xd;
const ALWAYS_CONDITION: u8 = 0xe;
const CONDITION_MASK: u32 = 0xf0000000;
const INSTRUCTION_CLASS_MASK: u32 = 0x0e000000;
const BRANCH_INSTRUCTION_CLASS: u32 = 0x0a000000;
const DATA_PROCESSING_REGISTER_INSTRUCTION_CLASS: u32 = 0x00000000;
const DATA_PROCESSING_IMMEDIATE_INSTRUCTION_CLASS: u32 = 0x02000000;
const LOAD_STORE_IMMEDIATE_INSTRUCTION_CLASS: u32 = 0x04000000;
const LOAD_STORE_REGISTER_INSTRUCTION_CLASS: u32 = 0x05000000;
const SUPERVISOR_CALL_INSTRUCTION_CLASS: u32 = 0x0e000000;
const EXTRA_LOAD_STORES_FLAG: u32 = 0x00000090;
const DATA_PROCESSING_IMMEDIATE_EXTRA_INSTRUCTIONS_MASK: u32 = 0x01900000;
const DATA_PROCESSING_IMMEDIATE_EXTRA_INSTRUCTIONS_VALUE: u32 = 0x01000000;
const UPDATE_STATUS_BIT: u32 = 0x00100000;
const IMMEDIATE_MODE_BIT: u32 = 0x02000000;
const OPCODE_MASK: u32 = 0x01e00000;

const ADD_OPCODE: u8 = 0x4;
const ADD_WITH_CARRY_OPCODE: u8 = 0x5;
const AND_OPCODE: u8 = 0x0;
const BRANCH_EXCHANGE_OPCODE: u8 = 0x9;
const COMPARE_OPCODE: u8 = 0xa;
const MOVE_OPCODE: u8 = 0xd;
const MOVE_HALFWORD_OPCODE: u8 = 0x8;
const MOVE_HALFWORD_TOP_OPCODE: u8 = 0xa;
const MOVE_NOT_OPCODE: u8 = 0xf;
const OR_OPCODE: u8 = 0xc;
const SUBTRACT_OPCODE: u8 = 0x2;

const SHIFT_TYPE_LOGICAL_SHIFT_LEFT: u8 =       0b0000000;
const SHIFT_TYPE_LOGICAL_SHIFT_RIGHT: u8 =      0b0100000;
const SHIFT_TYPE_ARITHMETIC_SHIFT_RIGHT: u8 =   0b1000000;
const SHIFT_TYPE_ROTATE_RIGHT: u8 =             0b1100000;
const SHIFT_TYPE_MASK: u32 = 0x00000060;
const SHIFT_IMMEDIATE_BIT: u32 = 0x00000010;