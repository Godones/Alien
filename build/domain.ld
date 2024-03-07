OUTPUT_ARCH(riscv)
ENTRY(main)
BASE_ADDRESS = 0;

SECTIONS
{
    . = BASE_ADDRESS;
    .text :{
        stext = .;
        *(.text .text.*);
        . = ALIGN(4K);
        etext = .;
    }

    .rodata : ALIGN(4096){
        srodata = .;
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
        *(.eh_frame)
        . = ALIGN(4K);
        erodata = .;
    }

    .data : ALIGN(4096){
        sdata = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
        . = ALIGN(4K);
        edata = .;
    }

    .rela.dyn : ALIGN(4096) {
        __rela_dyn_start = .;
        *(.rela.dyn .rela.dyn*)
        __rela_dyn_end = .;
    }

    .bss :  ALIGN(4096){
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        . = ALIGN(4K);
        ebss = .;
    }

    /* If we're on a newer compiler */
    /DISCARD/ :
    {
        *(.interp)
        *(.dynstr)
        *(.hash)
        *(.gnu.hash)
        *(.header)
    } : phdr

/*    .eh_frame_hdr : ALIGN(4K){*/
/*        kernel_eh_frame_hdr = .;*/
/*        *(.eh_frame_hdr .eh_frame_hdr.*)*/
/*        . = ALIGN(4K);*/
/*        kernel_eh_frame_hdr_end = .;*/
/*    }*/

    PROVIDE(end = .);
}