use super::Core;

pub fn read(csr: Csr, core: &Core) -> u32 {
    let addr = csr_addr(csr);
    core.csr_file[addr]
}

pub fn write(csr: Csr, data: u32, core: &mut Core) {
    let addr = csr_addr(csr);
    core.csr_file[addr] = data;
    mirror(core);
}

pub fn read_addr(addr: u32, core: &Core) -> u32 {
    println!("csr read: 0x{:x}", addr);
    core.csr_file[addr as usize]
}

pub fn write_addr(addr: u32, data: u32, core: &mut Core) {
    println!("csr write: 0x{:x} {:x}", addr, data);
    core.csr_file[addr as usize] = data;
    mirror(core);
}

pub fn read_64(csr: Csr64, core: &Core) -> u64 {
    let low: u64;
    let high: u64;
    match csr {
        Csr64::time => {
            low = core.csr_file[csr_addr(Csr::time)] as u64;
            high = core.csr_file[csr_addr(Csr::timeh)] as u64;
        }
        Csr64::cycle => {
            low = core.csr_file[csr_addr(Csr::cycle)] as u64;
            high = core.csr_file[csr_addr(Csr::cycleh)] as u64;
        }
        Csr64::mcycle => {
            low = core.csr_file[csr_addr(Csr::mcycle)] as u64;
            high = core.csr_file[csr_addr(Csr::mcycleh)] as u64;
        }
        Csr64::instret => {
            low = core.csr_file[csr_addr(Csr::instret)] as u64;
            high = core.csr_file[csr_addr(Csr::instreth)] as u64;
        }
        Csr64::minstret => {
            low = core.csr_file[csr_addr(Csr::minstret)] as u64;
            high = core.csr_file[csr_addr(Csr::minstreth)] as u64;
        }
        Csr64::medeleg => {
            low = core.csr_file[csr_addr(Csr::medeleg)] as u64;
            high = core.csr_file[csr_addr(Csr::medelegh)] as u64;
        }
        Csr64::mstatus => {
            low = core.csr_file[csr_addr(Csr::mstatus)] as u64;
            high = core.csr_file[csr_addr(Csr::mstatush)] as u64;
        }
    };
    (high << 32) + low
}

pub fn write_64(csr: Csr64, data: u64, core: &mut Core) {
    match csr {
        Csr64::time => {
            core.csr_file[csr_addr(Csr::time)] = data as u32;
            core.csr_file[csr_addr(Csr::timeh)] = (data >> 32) as u32;
        }
        Csr64::cycle => {
            core.csr_file[csr_addr(Csr::cycle)] = data as u32;
            core.csr_file[csr_addr(Csr::cycleh)] = (data >> 32) as u32;
        }
        Csr64::mcycle => {
            core.csr_file[csr_addr(Csr::mcycle)] = data as u32;
            core.csr_file[csr_addr(Csr::mcycleh)] = (data >> 32) as u32;
        }
        Csr64::instret => {
            core.csr_file[csr_addr(Csr::instret)] = data as u32;
            core.csr_file[csr_addr(Csr::instreth)] = (data >> 32) as u32;
        }
        Csr64::minstret => {
            core.csr_file[csr_addr(Csr::minstret)] = data as u32;
            core.csr_file[csr_addr(Csr::minstreth)] = (data >> 32) as u32;
        }
        Csr64::medeleg => {
            core.csr_file[csr_addr(Csr::medeleg)] = data as u32;
            core.csr_file[csr_addr(Csr::medelegh)] = (data >> 32) as u32;
        }
        Csr64::mstatus => {
            core.csr_file[csr_addr(Csr::mstatus)] = data as u32;
            core.csr_file[csr_addr(Csr::mstatush)] = (data >> 32) as u32;
        }
    };
    mirror(core);
}

fn mirror(core: &mut Core) {
    // sstatus
    let mstatus = core.csr_file[csr_addr(Csr::mstatus)];
    let mask = 0b10000001100011111110011111100011;
    core.csr_file[csr_addr(Csr::sstatus)] = mstatus & mask;

    // timers
    let mcycle = core.csr_file[csr_addr(Csr::mcycle)];
    let mcycleh = core.csr_file[csr_addr(Csr::mcycleh)];
    let time = core.memory.csr_read(crate::memory::RTC::Mtime);
    let minstret = core.csr_file[csr_addr(Csr::minstret)];
    let minstreth = core.csr_file[csr_addr(Csr::minstreth)];
    core.csr_file[csr_addr(Csr::cycle)] = mcycle;
    core.csr_file[csr_addr(Csr::cycleh)] = mcycleh;
    core.csr_file[csr_addr(Csr::time)] = time as u32;
    core.csr_file[csr_addr(Csr::timeh)] = (time >> 32) as u32;
    core.csr_file[csr_addr(Csr::instret)] = minstret;
    core.csr_file[csr_addr(Csr::instreth)] = minstreth;

    //hmpcounters?

    // sip, sie
    let mie = read(Csr::mie, core);
    let mip = read(Csr::mip, core);
    let mideleg = read(Csr::mideleg, core);
    core.csr_file[csr_addr(Csr::sie)] = mie & mideleg;
    core.csr_file[csr_addr(Csr::sip)] = mip & mideleg;
}

#[derive(Debug)]
#[allow(non_camel_case_types, dead_code)]
pub enum Csr64 {
    cycle,
    time,
    instret,

    mcycle,
    minstret,

    medeleg,

    mstatus,
}

#[derive(Debug)]
#[allow(non_camel_case_types, dead_code)]
pub enum Csr {
    mscratch,
    sscratch,

    cycle,
    cycleh,
    time,
    timeh,
    instret,
    instreth,

    mcycle,
    mcycleh,
    minstret,
    minstreth,

    minhibiten,
    mcounteren,
    scounteren,

    mip,
    sip,
    mie,
    sie,
    mtvec,
    stvec,

    mepc,
    sepc,
    mcause,
    scause,
    mtval,
    stval,

    medeleg,
    medelegh,
    mideleg,

    mstatus,
    mstatush,
    sstatus,

    satp,

    misa,
    mhartid,
}

fn csr_addr(csrname: Csr) -> usize {
    match csrname {
        Csr::mscratch => 0x340,
        Csr::sscratch => 0x140,

        Csr::cycle => 0xC00,
        Csr::cycleh => 0xC80,
        Csr::time => 0xC01,
        Csr::timeh => 0xC81,
        Csr::instret => 0xC02,
        Csr::instreth => 0xC82,

        Csr::mcycle => 0xB00,
        Csr::mcycleh => 0xB80,
        Csr::minstret => 0xB02,
        Csr::minstreth => 0xB82,

        Csr::minhibiten => 0x320,
        Csr::mcounteren => 0x306,
        Csr::scounteren => 0x106,

        Csr::mip => 0x344,
        Csr::sip => 0x144,
        Csr::mie => 0x304,
        Csr::sie => 0x104,
        Csr::mtvec => 0x305,
        Csr::stvec => 0x105,

        Csr::mepc => 0x341,
        Csr::sepc => 0x141,
        Csr::mcause => 0x342,
        Csr::scause => 0x142,
        Csr::mtval => 0x343,
        Csr::stval => 0x143,

        Csr::medeleg => 0x302,
        Csr::medelegh => 0x312,
        Csr::mideleg => 0x303,

        Csr::mstatus => 0x300,
        Csr::mstatush => 0x310,
        Csr::sstatus => 0x100,

        Csr::satp => 0x180,

        Csr::misa => 0x301,
        Csr::mhartid => 0xF14,
    }
}
