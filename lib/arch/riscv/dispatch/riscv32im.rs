use core::arch::{asm, global_asm};

use crate::{
    arch::riscv::{
        csr::{Sepc, Sstatus},
        frame::riscv32im::FrameRiscv32im,
    },
    kernel::{
        environment::{Dispatch, DispatchLevel, Frame},
        scheduler::Stack,
    },
};

pub struct DispatcherRiscv32im {}

impl DispatcherRiscv32im {
    fn leave_control(frame: &FrameRiscv32im) -> ! {
        Self::deactivate_irq();
        let mut sstatus = Sstatus::load();
        sstatus.SPP = !frame.is_user_mode();
        sstatus.store();

        unsafe {
            asm!("
                mv a0, {frame}
                // Get the pc pointer
                lw t0, 124(a0)
                csrw sepc, t0
                lw x1, 0(a0)
                lw x2, 4(a0)
                lw x3, 8(a0)
                lw x4, 12(a0)
                lw x5, 16(a0)
                lw x6, 20(a0)
                lw x7, 24(a0)
                lw x8, 28(a0)
                lw x9, 32(a0)
                // Skip x10 since it's a0
                lw x11, 40(a0)
                lw x12, 44(a0)
                lw x13, 48(a0)
                lw x14, 52(a0)
                lw x15, 56(a0)
                lw x16, 60(a0)
                lw x17, 64(a0)
                lw x18, 68(a0)
                lw x19, 72(a0)
                lw x20, 76(a0)
                lw x21, 80(a0)
                lw x22, 84(a0)
                lw x23, 88(a0)
                lw x24, 92(a0)
                lw x25, 96(a0)
                lw x26, 100(a0)
                lw x27, 104(a0)
                lw x28, 108(a0)
                lw x29, 112(a0)
                lw x30, 116(a0)
                lw x31, 120(a0)
                lw x10, 36(a0) // restore a0 too
                sret
            ", 
                frame = in(reg) (frame as *const FrameRiscv32im),
                options(noreturn)
            );
        }
    }
}

impl Dispatch<FrameRiscv32im> for DispatcherRiscv32im {
    unsafe fn dispatch(frame: &FrameRiscv32im) -> ! {
        Self::leave_control(frame)
    }

    fn activate_irq() {
        let mut sstatus = Sstatus::load();
        sstatus.SIE = true;
        sstatus.store();
    }

    fn deactivate_irq() {
        let mut sstatus = Sstatus::load();
        sstatus.SIE = false;
        sstatus.store();
    }

    fn irq_lock<T, F: FnOnce() -> T>(f: F) -> T {
        DispatcherRiscv32im::deactivate_irq();
        let temp = f();
        DispatcherRiscv32im::activate_irq();
        temp
    }

    fn kernel_yield() {
        DispatcherRiscv32im::deactivate_irq();
        unsafe {
            riscv32imac_ecall_yield();
        }
        DispatcherRiscv32im::activate_irq();
    }
}

extern "C" {
    fn riscv32imac_ecall_yield();
}

global_asm!(
    "
.global riscv32imac_ecall_yield
riscv32imac_ecall_yield:
  addi sp, sp, -4*33
  sw x1, 4*0(sp)
  sw x3, 4*2(sp)
  sw x4, 4*3(sp)
  sw x5, 4*4(sp)
  sw x6, 4*5(sp)
  sw x7, 4*6(sp)
  sw x8, 4*7(sp)
  sw x9, 4*8(sp)
  sw x10, 4*9(sp)
  sw x11, 4*10(sp)
  sw x12, 4*11(sp)
  sw x13, 4*12(sp)
  sw x14, 4*13(sp)
  sw x15, 4*14(sp)
  sw x16, 4*15(sp)
  sw x17, 4*16(sp)
  sw x18, 4*17(sp)
  sw x19, 4*18(sp)
  sw x20, 4*19(sp)
  sw x21, 4*20(sp)
  sw x22, 4*21(sp)
  sw x23, 4*22(sp)
  sw x24, 4*23(sp)
  sw x25, 4*24(sp)
  sw x26, 4*25(sp)
  sw x27, 4*26(sp)
  sw x28, 4*27(sp)
  sw x29, 4*28(sp)
  sw x30, 4*29(sp)
  sw x31, 4*30(sp)
  la t0, 1f
  sw t0, 4*31(sp)
  addi t0, sp, 4*33
  sw t0, 4*1(sp)
  li t0, 0
  sw t0, 4*32(sp)
  mv a0, sp
  csrr a1, sscratch
  call supervisor_ecall
1:
  nop
  ret
"
);
