use super::{exceptions::Exception, Core};
use crate::memory;

static LEGAL_ADRESSES: [u32; 61] = [0x340, 0x140, 0xC00, 0xC80, 0xC01, 0xC81, 0xC02, 0xC82, 0xB00, 0xB80, 0xB02, 0xB82, 0x320, 0x306, 0x106, 0x344, 0x144, 0x304, 0x104, 0x305, 0x105, 0x341, 0x141, 0x342, 0x142, 0x343, 0x143, 0x302, 0x312, 0x303, 0x300, 0x310, 0x100, 0x180, 0x301, 0xF14, 0x3A0, 0x3A1, 0x3A2, 0x3A3, 0x3B0, 0x3B1, 0x3B2, 0x3B3, 0x3B4, 0x3B5, 0x3B6, 0x3B7, 0x3B8, 0x3B9, 0x3BA, 0x3BB, 0x3BC, 0x3BD, 0x3BE, 0x3BF, 0xf11, 0xf12, 0xf13, 0xf14, 0x0];

pub fn read(csr: Csr, core: &Core) -> u32 {
    let addr = csr_addr(csr);
    core.csr_file[addr]
}

pub fn write(csr: Csr, data: u32, core: &mut Core) {
    let addr = csr_addr(csr);
    core.csr_file[addr] = data;
    mirror(core);
}

#[allow(non_snake_case)]
pub fn read_pmpXcfg(n: u32, core: &Core) -> u8 {
    let addr = match n/4 {
        0 => csr_addr(Csr::pmpcfg0),
        1 => csr_addr(Csr::pmpcfg1),
        2 => csr_addr(Csr::pmpcfg2),
        _ => csr_addr(Csr::pmpcfg3),
    };
    let csr = core.csr_file[addr];
    let val = csr >> (n%4)*8;
    val as u8
}

#[allow(non_snake_case)]
pub fn write_pmpXcfg(n: u32, data: u8, core: &mut Core) {
    let addr = match n/4 {
        0 => csr_addr(Csr::pmpcfg0),
        1 => csr_addr(Csr::pmpcfg1),
        2 => csr_addr(Csr::pmpcfg2),
        _ => csr_addr(Csr::pmpcfg3),
    };
    let mut csr = core.csr_file[addr];
    csr &= !(0b11111111 << (n%4)*8);
    let data = u32::from(data) << (n%4)*8;
    core.csr_file[addr] = csr & data;
    mirror(core);
}

pub fn read_addr(addr: u32, core: &Core) -> Result<u32, Exception> {
    // println!("csr read: 0x{:x} 0x{:x}", addr, core.csr_file[addr as usize]);
    let perm = permissions(addr);
    if perm.mode > core.mode {
        println!("Error csr read: 0x{:x}; No permisions {:?}", addr, perm);
        return Err(Exception::Illegal_instruction);
    }
    for laddr in LEGAL_ADRESSES {
        if laddr == addr {
            return Ok(core.csr_file[addr as usize]);
        }
    }
    println!("Error csr read: 0x{:x}; Illegal address", addr);
    Err(Exception::Illegal_instruction)
}

pub fn write_addr(addr: u32, data: u32, core: &mut Core) -> Result<(), Exception> {
    // println!("csr write: 0x{:x} 0x{:x}", addr, core.csr_file[addr as usize]);
    let perm = permissions(addr);
    if perm.mode > core.mode || !perm.w {
        println!("Error csr write: 0x{:x} 0x{:x}; No permisions: {:?}", addr, data, perm);
        return Err(Exception::Illegal_instruction)
    }
    for laddr in LEGAL_ADRESSES {
        if laddr == addr {
            core.csr_file[addr as usize] = data;
            mirror(core);
            return Ok(())
        }
    }
    println!("Error csr write: 0x{:x}; Illegal address", addr);
    Err(Exception::Illegal_instruction)
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
    let time = core.mtime;
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
    let mie = core.csr_file[csr_addr(Csr::mie)];
    let mip = core.csr_file[csr_addr(Csr::mip)];
    let mideleg = core.csr_file[csr_addr(Csr::mideleg)];
    core.csr_file[csr_addr(Csr::sie)] = mie & mideleg;
    core.csr_file[csr_addr(Csr::sip)] = mip & mideleg;
}

#[derive(Debug)]
struct Csr_permissions {
    mode: u32,
    w: bool,
}

fn permissions(addr: u32) -> Csr_permissions {
    let mode = (addr & (0b11 << 8)) >> 8;
    let rw = (addr & (0b11 << 10)) >> 10;
    let w = rw < 0b11;
    Csr_permissions { mode, w }
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
    mvendorid,
    marchid,
    mimpid,

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

    pmpcfg0,
    pmpcfg1,
    pmpcfg2,
    pmpcfg3,

    pmpaddr0,
    pmpaddr1,
    pmpaddr2,
    pmpaddr3,
    pmpaddr4,
    pmpaddr5,
    pmpaddr6,
    pmpaddr7,
    pmpaddr8,
    pmpaddr9,
    pmpaddr10,
    pmpaddr11,
    pmpaddr12,
    pmpaddr13,
    pmpaddr14,
    pmpaddr15,

}


pub fn csr_addr(csrname: Csr) -> usize {
    match csrname {
        Csr::mvendorid => 0xf11,
        Csr::marchid => 0xf12,
        Csr::mimpid => 0xf13,

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

        Csr::pmpcfg0 => 0x3A0,
        Csr::pmpcfg1 => 0x3A1,
        Csr::pmpcfg2 => 0x3A2,
        Csr::pmpcfg3 => 0x3A3,

        Csr::pmpaddr0 => 0x3B0,
        Csr::pmpaddr1 => 0x3B1,
        Csr::pmpaddr2 => 0x3B2,
        Csr::pmpaddr3 => 0x3B3,
        Csr::pmpaddr4 => 0x3B4,
        Csr::pmpaddr5 => 0x3B5,
        Csr::pmpaddr6 => 0x3B6,
        Csr::pmpaddr7 => 0x3B7,
        Csr::pmpaddr8 => 0x3B8,
        Csr::pmpaddr9 => 0x3B9,
        Csr::pmpaddr10 => 0x3BA,
        Csr::pmpaddr11 => 0x3BB,
        Csr::pmpaddr12 => 0x3BC,
        Csr::pmpaddr13 => 0x3BD,
        Csr::pmpaddr14 => 0x3BE,
        Csr::pmpaddr15 => 0x3BF,
    }
}
        
