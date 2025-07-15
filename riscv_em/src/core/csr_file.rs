pub struct CSR_file {
    csr_file: [u32; 4096],

}

impl Default for CSR_file {
    fn default() -> Self {
        CSR_file { csr_file: [0; 4096] }
    }
}

impl CSR_file {
    pub fn read(&self, csr: Csr, _mode: u32) -> u32 {
        let addr = csr_addr(csr);
        self.csr_file[addr]
    }

    pub fn write(&mut self, csr: Csr, data: u32, _mode: u32) {
        let addr = csr_addr(csr);
        self.csr_file[addr] = data;
    }

    pub fn read_addr(&self, addr: u32, _mode: u32) -> u32 {
        self.csr_file[addr as usize]
    }

    pub fn write_addr(&mut self, addr: u32, data: u32, _mode: u32) {
        self.csr_file[addr as usize] = data;
    }

    pub fn read_mcycle(&self) -> u64 {
        let mcycle = self.csr_file[csr_addr(Csr::mcycle)] as u64;
        let mcycleh = self.csr_file[csr_addr(Csr::mcycleh)] as u64;
        (mcycleh << 32) + mcycle
    }

    pub fn write_mcycle(&mut self, data: u64) {
        let mcycleh = (data >> 32) as u32;
        let mcycle = data as u32;
        self.csr_file[csr_addr(Csr::mcycle)] = mcycle;
        self.csr_file[csr_addr(Csr::mcycleh)] = mcycleh;
    }


}



#[derive(Debug)]
#[allow(non_camel_case_types)]
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

