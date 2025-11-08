#[allow(non_camel_case_types, dead_code)]
#[derive(Debug)]
pub enum Exception {
    Clear,
    Instruction_address_misaligned,
    Instruction_access_fault,
    Illegal_instruction,
    Breakpoint,
    Load_address_misaligned,
    Load_access_fault,
    StoreAMO_address_misaligned,
    StoreAMO_access_fault,
    Environment_call_from_Umode,
    Environment_call_from_Smode,
    Environment_call_from_Mmode,
    Instruction_page_fault,
    Load_page_fault,
    StoreAMO_page_fault,
    Hardware_error,
}

pub fn exception_number(exc: & Exception) -> u32 {
    match exc {
        Exception::Clear => u32::MAX,
        Exception::Instruction_address_misaligned => 0,
        Exception::Instruction_access_fault => 1,
        Exception::Illegal_instruction => 2,
        Exception::Breakpoint => 3,
        Exception::Load_address_misaligned => 4,
        Exception::Load_access_fault => 5,
        Exception::StoreAMO_address_misaligned => 6,
        Exception::StoreAMO_access_fault => 7,
        Exception::Environment_call_from_Umode => 8,
        Exception::Environment_call_from_Smode => 9,
        Exception::Environment_call_from_Mmode => 11,
        Exception::Instruction_page_fault => 12,
        Exception::Load_page_fault => 13,
        Exception::StoreAMO_page_fault => 15,
        Exception::Hardware_error => 19,
    }
}
