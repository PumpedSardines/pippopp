.section .text._start
.global _start

_start:
  la sp, _stack_begin
  la gp, __global_pointer
  call _set_up_irq

  // li t0, 0xdeadbee0
  // csrw stvec, t0

  j kernel_main

loop: j loop

_set_up_irq:
  la t0, _irq_request_riscv32im
  csrw stvec, t0
  // bit 5 is timer interrupt. Must be set in sie for interrupts to work
  li t0, 1 << 5
  csrw sie, t0
  ret

.align 4
_irq_request_riscv32im:
  // Load the kernel pointer and store the old stack pointer
  csrrw sp, sscratch, sp

  // Start by saving some registers on the scratch buffer in the kernel
  sw t0, 0*4(sp)
  sw t1, 1*4(sp)
  sw t2, 2*4(sp)

  // Get if previous request was user or supervisor
  csrr t0, sstatus
  srli t0, t0, 8
  andi t0, t0, 1

  // In the case where the previous request was supervisor, we want to use the same stack
  // In the case where the previous request was user, we want to reset the stack at the beginning

  // t0 = x5
  // t1 = x6
  // t2 = x7

  mv t1, sp

  // Restore the kernel pointer and get the old stack pointer
  csrrw sp, sscratch, sp
  // Store the stack pointer on the scratch
  sw sp, 3*4(t1)

  lw t2, 9*4(t1) // Load if we are waiting
  beqz t2, 1f
  // If we are waiting, we only need to restore the stack, no other info is relevant
  lw sp, 8*4(t1) // Load the stack pointer from the kernel
  // Load the kernel pointer into first argument
  mv a0, t1
  call waiting_trap_entry
1:
  
  bnez t0, 1f // Jump if supervisor mode
  // Load the stack pointer from the kernel
  // If we are in supervisor mode we want to reuse the same stack so we skip to set it here
  lw sp, 8*4(t1)
1:
  // Increment the stack pointer
  addi sp, sp, -4*33
2:
  // The outcommented lines contain junk, so dont restore them just yet
  sw x1, 4*0(sp)
  // sw x2, 4*1(sp)
  sw x3, 4*2(sp)
  sw x4, 4*3(sp)
  // sw x5, 4*4(sp)
  // sw x6, 4*5(sp)
  // sw x7, 4*6(sp)
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
  // Invert the user bit since the Frame object expectes the opposite
  xori t0, t0, 1
  sw t0, 4*32(sp)
  // Store the 
  csrr t0, sepc
  sw t0, 4*31(sp)

  // Load the old t0 from the kernel scracth
  lw t0, 0*4(t1)
  sw t0, 4*4(sp)
  // Load the old t1 from the kernel scracth
  lw t0, 1*4(t1)
  sw t0, 5*4(sp)
  // Load the old t2 from the kernel scracth
  lw t0, 2*4(t1)
  sw t0, 6*4(sp)
  // Load the old sp from the kernel scracth
  lw t0, 3*4(t1)
  sw t0, 1*4(sp)

  // The trap_entry takes in a pointer to the stack frame and a pointer to the kernel object
  mv a0, sp
  mv a1, t1

  call trap_entry
1: j 1b
