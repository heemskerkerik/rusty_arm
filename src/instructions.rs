use ux::{u12, u5, u4, u24};

pub type Register = u4;

#[derive(Debug)]
pub enum Condition {
    Equal,                  // EQ
    NotEqual,               // NE
    CarrySet,               // CS
    CarryClear,             // CC
    Negative,               // MI
    Positive,               // PL
    Overflow,               // VS
    NoOverflow,             // VC
    UnsignedHigher,         // HI
    UnsignedLowerOrSame,    // LS
    GreaterThanOrEqual,     // GE
    LessThan,               // LT
    GreaterThan,            // GT
    LessThanOrEqual,        // LE
    Always,                 // AL
}

#[derive(Debug)]
pub enum ShiftOperand {
    Immediate(u5),
    Register(Register),
}

#[derive(Debug)]
pub enum ShiftType {
    LogicalShiftLeft,
    LogicalShiftRight,
    ArithmeticShiftRight,
    RotateRight,
}

#[derive(Debug)]
pub struct ReadWriteImmediateDataArguments {
    pub source_register: Register,
    pub destination_register: Register,
    pub immediate: u32,
    pub carry: bool,
    pub rotate: u8,
}

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

#[derive(Debug)]
pub struct ImmediateDataArguments {
    pub register: Register,
    pub immediate: u32,
    pub carry: bool,
    pub rotate: u8,
}

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
pub struct LargeImmediateArguments {
    pub register: Register,
    pub immediate: u16,
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
pub struct LoadStoreRegisterOffset {
    pub register: Register,
    pub shift_type: ShiftType,
    pub shift_operand: ux::u5,
}

#[derive(Debug)]
pub enum LoadStoreOffset {
    Immediate(u12),
    Register(LoadStoreRegisterOffset),
}

#[derive(Debug)]
pub enum LoadDataSize {
    Word,
    Byte,
    DoubleWord,
    UnsignedHalfWord,
    SignedByte,
    SignedHalfWord,
}

#[derive(Debug)]
pub enum StoreDataSize {
    Word,
    Byte,
    DoubleWord,
    HalfWord,
}

#[derive(Debug)]
pub enum LoadStoreIndexingType {
    PreIndexed,
    PostIndexed,
}

#[derive(Debug)]
pub enum LoadStoreWriteBackFlag {
    WriteBack,
    DoNotWriteBack,
}

#[derive(Debug)]
pub enum LoadStoreOffsetDirection {
    Positive,
    Negative,
}

#[derive(Debug)]
pub struct LoadArguments {
    pub data_size: LoadDataSize,
    pub common_arguments: LoadStoreArguments,
}
#[derive(Debug)]
pub struct StoreArguments {
    pub data_size: StoreDataSize,
    pub common_arguments: LoadStoreArguments,
}

#[derive(Debug)]
pub struct LoadStoreArguments {
    pub indexing_type: LoadStoreIndexingType,
    pub write_back: LoadStoreWriteBackFlag,
    pub offset_direction: LoadStoreOffsetDirection,
    pub value_register: Register,
    pub address_register: Register,
    pub offset: LoadStoreOffset,
}

#[derive(Debug)]
pub enum InstructionData {
    Add(ReadWriteDataArguments, UpdateStatusFlags),                 // ADD<c>[S]
    AddWithCarry(ReadWriteDataArguments, UpdateStatusFlags),        // ADC<c>[S]
    //And(ReadWriteDataArguments, UpdateStatusFlags),
    Branch(i32, BranchLinkFlag),                                    // B[L]<c>
    BranchExchange(Register),                                       // BX<c>
    Compare(DataArguments),                                         // CMP<c>
    Load(LoadArguments),                                            // LDR[B]<c>, LDRH<c>, LDRSH<c>, LDRD<c>, LDRSB<c>, POP<c>
    Move(DataArguments, UpdateStatusFlags),                         // MOV<c>[S]
    MoveHalfWord(LargeImmediateArguments),                          // MOVW<c>
    MoveNot(DataArguments, UpdateStatusFlags),                      // MVN<c>[S]
    MoveTop(LargeImmediateArguments),                               // MOVT<c>
    Or(ReadWriteDataArguments, UpdateStatusFlags),                  // ORR<c>[S]
    SupervisorCall(u24),                                            // SVC
    Store(StoreArguments),                                          // STR[B]<c>, STRH<c>, STRSH<c>, STRD<c>, STRSB<c>, PUSH<c>
    Subtract(ReadWriteDataArguments, UpdateStatusFlags),            // SUB<c>[S]
}

pub type Instruction = (Condition, InstructionData);