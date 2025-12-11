use crate::kernel::environment::Frame;

#[derive(Debug, Clone)]
#[allow(dead_code)]
#[repr(C)]
pub struct FrameRiscv32im {
    pub ra: u32,
    pub sp: u32,
    pub gp: u32,
    pub tp: u32,
    pub t0: u32,
    pub t1: u32,
    pub t2: u32,
    pub s0: u32,
    pub s1: u32,
    pub a0: u32,
    pub a1: u32,
    pub a2: u32,
    pub a3: u32,
    pub a4: u32,
    pub a5: u32,
    pub a6: u32,
    pub a7: u32,
    pub s2: u32,
    pub s3: u32,
    pub s4: u32,
    pub s5: u32,
    pub s6: u32,
    pub s7: u32,
    pub s8: u32,
    pub s9: u32,
    pub s10: u32,
    pub s11: u32,
    pub t3: u32,
    pub t4: u32,
    pub t5: u32,
    pub t6: u32,
    pub pc: u32,
    pub user_mode: u32,
}

impl Frame for FrameRiscv32im {
    fn empty(pc: usize) -> Self {
        let mut f: Self = unsafe { core::mem::zeroed() };
        f.pc = pc as u32;
        f.user_mode = 1;
        return f;
    }

    fn set_pc(&mut self, pc: usize) {
        self.pc = pc as u32;
    }

    fn set_success(&mut self, args: (Option<usize>, Option<usize>, Option<usize>)) {
        self.a0 = args.0.map_or(self.a0, |v| v as u32);
        self.a1 = args.1.map_or(self.a1, |v| v as u32);
        self.a2 = args.2.map_or(self.a2, |v| v as u32); 
        self.a7 = 0;
    }

    fn set_error(&mut self, args: (Option<usize>, Option<usize>, Option<usize>)) {
        self.a0 = args.0.map_or(self.a0, |v| v as u32);
        self.a1 = args.1.map_or(self.a1, |v| v as u32);
        self.a2 = args.2.map_or(self.a2, |v| v as u32); 
        self.a7 = 1;
    }

    fn set_is_user_mode(&mut self, is_user_mode: bool) {
        self.user_mode = if is_user_mode { 1 } else { 0 };
    }

    fn is_user_mode(&self) -> bool {
        self.user_mode != 0
    }
}
