ENTRY(_start)

SECTIONS
{
   __stack_size = DEFINED(__stack_size) ? __stack_size : 0x100000;
   PROVIDE(__stack_size = __stack_size);
   __heap_size = DEFINED(__heap_size) ? __heap_size : 0x800;

   . = 0x80400000;
   PROVIDE(__kernel_begin = .);
   .text : {
    *(.text._start)
    *(.text*);
   }

   .rodata : {
    *(.rodata*);
   }

   .data : { *(.data*)
             PROVIDE( __global_pointer = . + 0x800 );
             *(.sdata*)}


  .bss : { *(.bss*) }
  .rodata : { *(.rodata*) }
  .comment : { *(.comment*) }
  .eh_frame : { *(.eh_frame*) }
  .stack :  {
  PROVIDE(_stack_end = .);
  . = ALIGN(4);
  . += __stack_size;
  PROVIDE(_stack_begin = .);
  }

  .kernel_heap : {
    . = ALIGN(4096);
    PROVIDE(__kernel_heap_begin = .);
    . += 1024 * 4096;
    PROVIDE(__kernel_heap_end = .);
  }


  PROVIDE(__kernel_end = .);

  .heap : {
    . = ALIGN(4096);
    PROVIDE(__heap_begin = .);
    . += 64 * 1024 * 1024; /* 64MB */
    PROVIDE(__heap_end = .);
  }

}
