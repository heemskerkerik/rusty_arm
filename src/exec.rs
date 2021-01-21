use core::panic;

use ux::u4;

use crate::{context::*, instructions::*};

const INSTRUCTION_SIZE: u32 = 4;

pub fn execute(context: &mut CpuContext, instr: Instruction) {
    let program_counter = context.get_program_counter();
    context.set_program_counter(program_counter + INSTRUCTION_SIZE);

    if !is_condition_met(context, &instr.0) {
        return;
    }

    if cfg!(feature = "print_instructions") {
        println!("{:0>8X} {:0>8X} {:?}", program_counter, context.read_word(program_counter), instr);
    }

    match instr.1 {
        InstructionData::Add(ref args, ref update_status) => execute_add(context, &args, &update_status),
        InstructionData::Branch(ref address, ref link) => execute_branch(context, &address, &link),
        InstructionData::Compare(ref args) => execute_compare(context, &args),
        InstructionData::Load(ref args) => execute_load(context, &args),
        InstructionData::Move(ref args, ref update_status) => execute_move(context, &args, &update_status),
        InstructionData::MoveHalfWord(ref args) => execute_move_half_word(context, &args),
        InstructionData::Store(ref args) => execute_store(context, &args),
        _ => panic!("Instruction {:?} not yet implemented", instr.1),
    }
}

fn is_condition_met(context: &CpuContext, cond: &Condition) -> bool {
    let status = context.get_status();

    match cond {
        Condition::Always => true,
        Condition::NotEqual => !status.zero,
        Condition::Equal => status.zero,
        _ => panic!("Condition {:?} not yet implemented.", cond)
    }
}

fn execute_move(context: &mut CpuContext, args: &DataArguments, update_status: &UpdateStatusFlags) {
    let (register, value, carry) = match args {
        DataArguments::Immediate(args) => {
            (args.register, args.immediate, if args.rotate == 0 { context.get_status().carry } else { args.carry })
        },
        DataArguments::Register(args) => {
            let (operand, carry) = apply_shift_operand(context, &args.operand_register, &args.shift_type, &args.shift_operand);
            (args.register, operand, carry)
        }
    };

    context.set_register(register.into(), value);

    if let UpdateStatusFlags::UpdateStatusFlags = *update_status {
        context.set_status(
            Some(get_sign(value)), 
            Some(value == 0), 
            Some(carry), 
            None
        );
    }
}

fn execute_add(context: &mut CpuContext, args: &ReadWriteDataArguments, update_status: &UpdateStatusFlags) {
    let (destination_register, original, result, operand) = match args {
        ReadWriteDataArguments::Immediate(args) => {
            let original = context.get_register(args.source_register.into());
            let result = original.wrapping_add(args.immediate);

            (args.destination_register, original, result, args.immediate)
        },
        ReadWriteDataArguments::Register(args) => {
            let original = context.get_register(args.source_register.into());
            let (operand, _) = apply_shift_operand(context, &args.operand_register, &args.shift_type, &args.shift_operand);
            let result = original.wrapping_add(operand);

            (args.destination_register, original, result, operand)
        }
    };

    context.set_register(destination_register.into(), result);

    if let UpdateStatusFlags::UpdateStatusFlags = *update_status {
        context.set_status(
            Some(get_sign(result)), 
            Some(result == 0), 
            Some((original as u64) + (operand as u64) > (u32::MAX as u64)), 
            Some(
                get_sign(original) != get_sign(operand)
                && get_sign(original) != get_sign(result)
            )
        );
    }
}

fn execute_compare(context: &mut CpuContext, args: &DataArguments) {
    let (register, operand) = match args {
        DataArguments::Immediate(args) => {
            (args.register, args.immediate)
        },
        DataArguments::Register(args) => {
            let (operand, _) = apply_shift_operand(context, &args.operand_register, &args.shift_type, &args.shift_operand);
            (args.register, operand)
        }
    };

    let original = context.get_register(register.into());
    let result = original.wrapping_sub(operand);

    context.set_status(
        Some(get_sign(result)), 
        Some(result == 0), 
        Some(result < original), 
        Some(
            get_sign(original) != get_sign(operand)
         && get_sign(original) != get_sign(result)
        )
    );
}

fn execute_branch(context: &mut CpuContext, address: &i32, link: &BranchLinkFlag) {
    // execute has already incremented PC by INSTRUCTION_SIZE
    let original_program_counter = context.get_program_counter() - INSTRUCTION_SIZE;

    if let BranchLinkFlag::LinkReturnAddress = *link {
        panic!("Branch with link not yet implemented!");
    }

    let destination = original_program_counter.wrapping_add(*address as u32);

    if destination == original_program_counter {
        context.halt()
    } else {
        context.set_program_counter(destination);
    }
}

fn execute_load(context: &mut CpuContext, args: &LoadStoreArguments) {
    execute_load_store(context, args, get_load_data, load_data);
}

fn execute_store(context: &mut CpuContext, args: &LoadStoreArguments) {
    execute_load_store(context, args, get_store_data, store_data);
}

fn execute_load_store(
    context: &mut CpuContext,
    args: &LoadStoreArguments, 
    get_data: fn(&CpuContext, u32, &LoadStoreArguments) -> u32,
    action: fn(&mut CpuContext, u32, u32, &LoadStoreArguments)) {
    let address = context.get_register(args.address_register.into());
    let offset: u32 = get_load_store_offset(context, &args.offset);
    let address = match args.indexing_type {
        LoadStoreIndexingType::PreIndexed => apply_offset(address, offset, &args.offset_direction),
        LoadStoreIndexingType::PostIndexed => address,
    };

    let data = get_data(context, address, args);

    if let LoadStoreWriteBackFlag::WriteBack = args.write_back {
        let address = match args.indexing_type {
            LoadStoreIndexingType::PreIndexed => address,
            LoadStoreIndexingType::PostIndexed => apply_offset(address, offset, &args.offset_direction),
        };
        context.set_register(args.address_register.into(), address);
    }

    action(context, address, data, args);
}

fn get_load_data(context: &CpuContext, address: u32, args: &LoadStoreArguments) -> u32 {
    let data = context.read_word(address);
    match args.data_size {
        LoadStoreRegularDataSize::Word => data,
        LoadStoreRegularDataSize::Byte => data & 0x000000ff,
    }
}

fn load_data(context: &mut CpuContext, _: u32, data: u32, args: &LoadStoreArguments) {
    context.set_register(args.value_register.into(), data);
}

fn get_store_data(context: &CpuContext, address: u32, args: &LoadStoreArguments) -> u32 {
    let data = context.get_register(args.value_register.into());
    match args.data_size {
        LoadStoreRegularDataSize::Word => data,
        LoadStoreRegularDataSize::Byte => {
            let current_word = context.read_word(address);
            (current_word & 0xffffff00) | (data & 0x000000ff)
        },
    }
}

fn store_data(context: &mut CpuContext, address: u32, data: u32, _: &LoadStoreArguments) {
    context.write_word(address, data);
}

fn execute_move_half_word(context: &mut CpuContext, args: &LargeImmediateArguments) {
    context.set_register(args.register.into(), args.immediate as u32);
}

fn get_sign(value: u32) -> bool {
    value & 0x80000000 != 0
}

fn apply_shift_operand(context: &CpuContext, register: &u4, shift_type: &ShiftType, shift_operand: &ShiftOperand) -> (u32, bool) {
    let raw = context.get_register((*register).into());

    let shift_operand = get_shift_operand(context, shift_operand);

    return match *shift_type {
        ShiftType::LogicalShiftLeft => logical_shift_left(context, raw, shift_operand),
        ShiftType::LogicalShiftRight => logical_shift_right(context, raw, shift_operand),
        _ => panic!("Unsupported shift type {:?}", *shift_type),
    };

    fn get_shift_operand(context: &CpuContext, operand: &ShiftOperand) -> u8 {
        match *operand {
            ShiftOperand::Immediate(immediate) => immediate.into(),
            ShiftOperand::Register(register) => context.get_register(register.into()) as u8,
        }
    }

}

fn logical_shift_left(context: &CpuContext, value: u32, bits: u8) -> (u32, bool) {
    let result = value << bits;

    if bits == 0 {
        (result, context.get_status().carry)
    } else if bits < 32 {
        (result, (value & (1 << (32 - bits))) != 0)
    } else if bits > 32 {
        (result, false)
    } else { // bits == 32 
        (result, (value & 1) != 0)
    }
}

fn logical_shift_right(context: &CpuContext, value: u32, bits: u8) -> (u32, bool) {
    let result = value >> bits;

    if bits == 0 {
        (result, context.get_status().carry)
    } else if bits < 32 {
        (result, (value & (1 << (bits - 1))) != 0)
    } else if bits > 32 {
        (result, false)
    } else { // bits == 32 
        (result, get_sign(value))
    }
}

fn apply_offset(address: u32, offset: u32, direction: &LoadStoreOffsetDirection) -> u32 {
    match *direction {
        LoadStoreOffsetDirection::Positive => address + offset,
        LoadStoreOffsetDirection::Negative => address - offset,
    }
}

fn get_load_store_offset(context: &CpuContext, offset: &LoadStoreOffset) -> u32 {
    match *offset {
        LoadStoreOffset::Immediate(v) => v.into(),
        LoadStoreOffset::Register(ref args) => {
            let offset = context.get_register(args.register.into());

            let shift_operand: u32 = args.shift_operand.into();
            match args.shift_type {
                ShiftType::LogicalShiftLeft => offset << shift_operand,
                ShiftType::LogicalShiftRight => offset >> shift_operand,
                ShiftType::RotateRight => offset.rotate_right(shift_operand),
                _ => panic!("Shift type {:?} not yet implemented", args.shift_type),
            }
        }
    }
}