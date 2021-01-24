use core::panic;

use ux::{u24, u4};

use crate::{context::*, instructions::*, syscall};

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
        InstructionData::AddWithCarry(ref args, ref update_status) => execute_add_with_carry(context, &args, &update_status),
        InstructionData::And(ref args, ref update_status) => execute_and(context, &args, &update_status),
        InstructionData::Branch(ref address, ref link) => execute_branch(context, &address, &link),
        InstructionData::BranchExchange(ref register) => execute_branch_exchange(context, &register),
        InstructionData::Compare(ref args) => execute_compare(context, &args),
        InstructionData::Load(ref args) => execute_load(context, &args),
        InstructionData::Move(ref args, ref update_status) => execute_move(context, &args, &update_status),
        InstructionData::MoveHalfWord(ref args) => execute_move_half_word(context, &args),
        InstructionData::MoveHalfWordTop(ref args) => execute_move_half_word_top(context, &args),
        InstructionData::MoveNot(ref args, ref update_status) => execute_move_not(context, &args, &update_status),
        InstructionData::Or(ref args, ref update_status) => execute_or(context, &args, &update_status),
        InstructionData::SupervisorCall(ref arg) => execute_supervisor_call(context, &arg),
        InstructionData::Store(ref args) => execute_store(context, &args),
        InstructionData::Subtract(ref args, ref update_status) => execute_subtract(context, &args, &update_status),
        _ => panic!("Instruction {:?} at {:0>8X} not yet implemented", instr.1, program_counter),
    }
}

fn is_condition_met(context: &CpuContext, cond: &Condition) -> bool {
    let status = context.get_status();

    match cond {
        Condition::Equal => status.zero,
        Condition::NotEqual => !status.zero,
        Condition::CarrySet => status.carry,
        Condition::CarryClear => !status.carry,
        Condition::Negative => status.negative,
        Condition::Positive => !status.negative,
        Condition::Overflow => status.overflow,
        Condition::NoOverflow => !status.overflow,
        Condition::UnsignedHigher => status.carry && !status.zero,
        Condition::UnsignedLowerOrSame => !status.carry || status.zero,
        Condition::GreaterThanOrEqual => status.negative == status.overflow,
        Condition::LessThan => status.negative != status.overflow,
        Condition::GreaterThan => !status.zero && status.negative == status.overflow,
        Condition::LessThanOrEqual => status.zero || status.negative != status.overflow,
        Condition::Always => true,
    }
}

fn execute_move(context: &mut CpuContext, args: &DataArguments, update_status: &UpdateStatusFlags) {
    let (register, value, carry) = get_data_arguments(context, args);

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

fn execute_move_not(context: &mut CpuContext, args: &DataArguments, update_status: &UpdateStatusFlags) {
    let (register, value, carry) = get_data_arguments(context, args);
    let value = !value;

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
    execute_add_core(context, args, update_status, |_, _, v| v)
}

fn execute_add_with_carry(context: &mut CpuContext, args: &ReadWriteDataArguments, update_status: &UpdateStatusFlags) {
    execute_add_core(
        context, 
        args, 
        update_status, 
        |c, _, v| v.wrapping_add(if c.get_status().carry { 1 } else { 0 })
    )
}

fn execute_add_core(
    context: &mut CpuContext, 
    args: &ReadWriteDataArguments, 
    update_status: &UpdateStatusFlags,
    modify_value: fn(&CpuContext, &ReadWriteDataArguments, u32) -> u32
) {
    let (destination_register, original, operand, _) = get_read_write_data_arguments(context, args);

    let result = original.wrapping_add(operand);
    let result = modify_value(context, args, result);

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

fn execute_subtract(context: &mut CpuContext, args: &ReadWriteDataArguments, update_status: &UpdateStatusFlags) {
    let (destination_register, original, operand, _) = get_read_write_data_arguments(context, args);

    let result = original.wrapping_sub(operand);

    context.set_register(destination_register.into(), result);

    if let UpdateStatusFlags::UpdateStatusFlags = *update_status {
        context.set_status(
            Some(get_sign(result)),
            Some(result == 0),
            Some(result <= original),
            Some(
                get_sign(original) != get_sign(operand)
             && get_sign(original) != get_sign(result)
            )
        );
    }
}

fn execute_or(context: &mut CpuContext, args: &ReadWriteDataArguments, update_status: &UpdateStatusFlags) {
    let (destination_register, original, operand, carry) = get_read_write_data_arguments(context, args);

    let result = original | operand;
    context.set_register(destination_register.into(), result);

    if let UpdateStatusFlags::UpdateStatusFlags = *update_status {
        context.set_status(
            Some(get_sign(result)),
            Some(result == 0),
            Some(carry),
            None
        );
    }
}

fn execute_and(context: &mut CpuContext, args: &ReadWriteDataArguments, update_status: &UpdateStatusFlags) {
    let (destination_register, original, operand, carry) = get_read_write_data_arguments(context, args);

    let result = original & operand;
    context.set_register(destination_register.into(), result);

    if let UpdateStatusFlags::UpdateStatusFlags = *update_status {
        context.set_status(
            Some(get_sign(result)),
            Some(result == 0),
            Some(carry),
            None
        );
    }
}

fn execute_compare(context: &mut CpuContext, args: &DataArguments) {
    let (register, operand, _) = get_data_arguments(context, args);

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
    // PC has already been advanced by execute
    let original_program_counter = context.get_program_counter();

    if let BranchLinkFlag::LinkReturnAddress = *link {
        context.set_register(CpuContext::get_link_return_register(), original_program_counter);
    }

    let destination = (original_program_counter - INSTRUCTION_SIZE).wrapping_add(*address as u32);

    if destination == original_program_counter {
        context.halt()
    } else {
        context.set_program_counter(destination);
    }
}

fn execute_branch_exchange(context: &mut CpuContext, register: &Register) {
    let destination_address = context.get_register((*register).into());
    context.set_program_counter(destination_address);
}

fn execute_load(context: &mut CpuContext, args: &LoadArguments) {
    execute_load_store(context, args, &args.common_arguments, get_load_data, load_data);
}

fn execute_store(context: &mut CpuContext, args: &StoreArguments) {
    execute_load_store(context, args, &args.common_arguments, get_store_data, store_data);
}

fn execute_load_store<A>(
    context: &mut CpuContext,
    full_args: &A,
    args: &LoadStoreArguments,
    get_data: fn(&CpuContext, u32, &A) -> u32,
    action: fn(&mut CpuContext, u32, u32, &A)
) {
    let address = context.get_register(args.address_register.into());
    let offset: u32 = get_load_store_offset(context, &args.offset);
    let address = match args.indexing_type {
        LoadStoreIndexingType::PreIndexed => apply_offset(address, offset, &args.offset_direction),
        LoadStoreIndexingType::PostIndexed => address,
    };

    let data = get_data(context, address, full_args);

    if let LoadStoreWriteBackFlag::WriteBack = args.write_back {
        let address = match args.indexing_type {
            LoadStoreIndexingType::PreIndexed => address,
            LoadStoreIndexingType::PostIndexed => apply_offset(address, offset, &args.offset_direction),
        };
        context.set_register(args.address_register.into(), address);
    }

    action(context, address, data, full_args);
}

fn get_load_data(context: &CpuContext, address: u32, args: &LoadArguments) -> u32 {
    match args.data_size {
        LoadDataSize::Word => context.read_word(address),
        LoadDataSize::Byte => context.read_byte(address) as u32,
        LoadDataSize::UnsignedHalfWord => context.read_half_word(address) as u32,
        _ => panic!("Data type {:?} not supported", args.data_size)
    }
}

fn load_data(context: &mut CpuContext, _: u32, data: u32, args: &LoadArguments) {
    context.set_register(args.common_arguments.value_register.into(), data);
}

fn get_store_data(context: &CpuContext, _: u32, args: &StoreArguments) -> u32 {
    let data = context.get_register(args.common_arguments.value_register.into());
    data
}

fn store_data(context: &mut CpuContext, address: u32, data: u32, args: &StoreArguments) {
    match args.data_size {
        StoreDataSize::Word => context.write_word(address, data),
        StoreDataSize::Byte => context.write_byte(address, (data & 0x000000ff) as u8),
        StoreDataSize::HalfWord => context.write_half_word(address, (data & 0x0000ffff) as u16),
        _ => panic!("Data type {:?} not supported", args.data_size),
    }
}

fn execute_move_half_word(context: &mut CpuContext, args: &LargeImmediateArguments) {
    context.set_register(args.register.into(), args.immediate as u32);
}

fn execute_move_half_word_top(context: &mut CpuContext, args: &LargeImmediateArguments) {
    let original = context.get_register(args.register.into());
    let shifted_immediate = (args.immediate as u32) << 16;

    let value = original & 0x0000ffff | shifted_immediate;

    context.set_register(args.register.into(), value);
}

fn execute_supervisor_call(context: &mut CpuContext, arg: &u24) {
    const SYSTEM_CALL: u32 = 0;

    if *arg != u24::new(SYSTEM_CALL) {
        panic!("Unsupported supervisor call {:0>6X}", *arg);
    }

    syscall::execute_system_call(context);
}

fn get_sign(value: u32) -> bool {
    value & 0x80000000 != 0
}

fn get_data_arguments(context: &CpuContext, args: &DataArguments) -> (Register, u32, bool) {
    match args {
        DataArguments::Immediate(args) => {
            (args.register, args.immediate, if args.rotate == 0 { context.get_status().carry } else { args.carry })
        },
        DataArguments::Register(args) => {
            let (operand, carry) = apply_shift_operand(context, &args.operand_register, &args.shift_type, &args.shift_operand);
            (args.register, operand, carry)
        }
    }
}

fn get_read_write_data_arguments(context: &CpuContext, args: &ReadWriteDataArguments) -> (Register, u32, u32, bool) {
    match args {
        ReadWriteDataArguments::Immediate(args) => {
            let original = context.get_register(args.source_register.into());
            (args.destination_register, original, args.immediate, args.carry)
        },
        ReadWriteDataArguments::Register(args) => {
            let original = context.get_register(args.source_register.into());
            let (operand, carry) = apply_shift_operand(context, &args.operand_register, &args.shift_type, &args.shift_operand);

            (args.destination_register, original, operand, carry)
        }
    }
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
    let result = value.wrapping_shl(bits as u32);

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
    let result = value.wrapping_shr(bits as u32);

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