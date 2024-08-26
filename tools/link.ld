OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    .text : ALIGN(4096){
        stext = .;
        *(.text.entry)
        . = ALIGN(4K);
        strampoline = . ;
        *(.text.trampoline)
        . = ALIGN(4K);
        etrampoline = . ;
        *(.text.startup)
        *(.text .text.*)
        . = ALIGN(4K);
        etext = .;
    }

    .rodata : ALIGN(4096){
        srodata = .;
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
        . = ALIGN(4K);
        erodata = .;
    }

    .data : ALIGN(4096){
        sdata = .;
        sinit = .;
        *(.init_array .init_array.*)
        einit = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
        . = ALIGN(4K);
        edata = .;
    }

    .bss :  ALIGN(4096){
        *(.bss.stack)
        . = ALIGN(4K);
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        . = ALIGN(4K);
        ebss = .;
    }

    .gcc_except_table : ALIGN(4K){
        kernel_gcc_except_table = .;
        *(.gcc_except_table .gcc_except_table.*)
        kernel_gcc_except_table_end = .;
    }

    .eh_frame_hdr : ALIGN(4K){
        kernel_eh_frame_hdr = .;
        *(.eh_frame_hdr .eh_frame_hdr.*)
        . = ALIGN(4K);
        kernel_eh_frame_hdr_end = .;
    }
    .eh_frame : ALIGN(4K){
        kernel_eh_frame = .;
        KEEP(*(.eh_frame .eh_frame.*))
        kernel_eh_frame_end = .;
    }
    . = ALIGN(4K);
    sheap = .;
    PROVIDE(end = .);
}