OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x40200000;

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
        . = ALIGN(4K);
        *(.bss.stack)
        . = ALIGN(4K);
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        . = ALIGN(4K);
        ebss = .;
    }


    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
    . = ALIGN(4K);

    PROVIDE(end = .);
}