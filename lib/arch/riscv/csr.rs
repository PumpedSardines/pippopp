use core::arch::asm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sstatus {
    /// Supervisor Previous Interrupt Enable
    pub SPIE: bool,
    /// Supervisor Interrupt Enable
    pub SIE: bool,
    /// Supervisor Previous Supervisor Mode
    pub SPP: bool,
    /// Supervisor Supervisor memory access
    pub SUM: bool,
}

impl Sstatus {
    pub fn load() -> Self {
        let sstatus: usize;
        unsafe {
            asm!("csrr {}, sstatus", out(reg) sstatus);
        }
        Sstatus {
            SPIE: (sstatus & (1 << 5)) != 0,
            SIE: (sstatus & (1 << 1)) != 0,
            SPP: (sstatus & (1 << 8)) != 0,
            SUM: (sstatus & (1 << 18)) != 0,
        }
    }

    pub fn store(&self) {
        let mut sstatus: usize = 0;
        if self.SPIE {
            sstatus |= 1 << 5;
        }
        if self.SIE {
            sstatus |= 1 << 1;
        }
        if self.SPP {
            sstatus |= 1 << 8;
        }
        if self.SUM {
            sstatus |= 1 << 18;
        }
        unsafe {
            asm!("csrw sstatus, {}", in(reg) sstatus);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sscratch(u32);

impl Sscratch {
    pub fn new(value: u32) -> Self {
        Sscratch(value)
    }

    pub fn load() -> Self {
        let sscratch: u32;
        unsafe {
            asm!("csrr {}, sscratch", out(reg) sscratch);
        }
        Sscratch(sscratch)
    }

    pub fn store(&self) {
        unsafe {
            asm!("csrw sscratch, {}", in(reg) self.0);
        }
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

/// Stores the return address from an exception or interrupt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sepc(u32);

impl Sepc {
    pub fn new(pc: u32) -> Self {
        Sepc(pc as u32)
    }

    pub fn load() -> Self {
        let sepc: u32;
        unsafe {
            asm!("csrr {}, sepc", out(reg) sepc);
        }
        Sepc(sepc)
    }

    pub fn store(&self) {
        unsafe {
            asm!("csrw sepc, {}", in(reg) self.0);
        }
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scause {
    Ecall,
    SegFault,
    Interrupt(u32),
    Unknown(u32),
}

impl Scause {
    pub fn load() -> Self {
        let scause: u32;
        unsafe {
            asm!("csrr {}, scause", out(reg) scause);
        }
        if scause & (1 << 31) != 0 {
            let interrupt_code = scause & !(1 << 31);
            Scause::Interrupt(interrupt_code)
        } else {
            // Exception
            match scause {
                0x8 => Scause::Ecall,
                0xd => Scause::SegFault,
                _ => Scause::Unknown(scause),
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stvec(u32);

impl Stvec {
    pub fn new(addr: u32) -> Self {
        Stvec(addr)
    }

    pub fn load() -> Self {
        let stvec: u32;
        unsafe {
            asm!("csrr {}, stvec", out(reg) stvec);
        }
        Stvec(stvec)
    }

    pub fn store(&self) {
        unsafe {
            asm!("csrw stvec, {}", in(reg) self.0);
        }
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Satp(u32);

impl Satp {
    pub fn new(satp: usize) -> Self {
        Self(satp as u32)
    }

    pub fn load() -> Self {
        let satp: u32;
        unsafe {
            asm!("csrr {}, satp", out(reg) satp);
        }
        Self(satp)
    }

    pub fn store(&self) {
        unsafe {
            asm!("csrw satp, {}", in(reg) self.0);
        }
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}
