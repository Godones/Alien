OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x40200000;

SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    .text : ALIGN(4096){
        __executable_start = .;
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
        __etext = .;
    }

    .rodata : ALIGN(4096){
        srodata = .;
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
        *(.gcc_except_table .gcc_except_table.*)
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

    eh_frame = .;
	.eh_frame : ALIGN(4096){
		__eh_frame_hdr_start = .;
        __GNU_EH_FRAME_HDR = .;
		*(.eh_frame_hdr)
		__eh_frame_hdr_end = .;
		__eh_frame_start = .;
		*(.eh_frame)
		*(.rela.eh_frame)
		__eh_frame_end = .;
        . = ALIGN(4K);
	}
    
    sheap = .;
/*    ekernel = .;*/
    PROVIDE(end = .);
}