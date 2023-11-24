ELF 头：
  Magic：   7f 45 4c 46 02 01 01 00 00 00 00 00 00 00 00 00 
  类别:                              ELF64
  数据:                              2 补码，小端序 (little endian)
  Version:                           1 (current)
  OS/ABI:                            UNIX - System V
  ABI 版本:                          0
  类型:                              EXEC (可执行文件)
  系统架构:                          RISC-V
  版本:                              0x1
  入口点地址：               0x10140
  程序头起点：          64 (bytes into file)
  Start of section headers:          109968 (bytes into file)
  标志：             0x0
  Size of this header:               64 (bytes)
  Size of program headers:           56 (bytes)
  Number of program headers:         4
  Size of section headers:           64 (bytes)
  Number of section headers:         23
  Section header string table index: 22

节头：
  [号] 名称              类型             地址              偏移量
       大小              全体大小          旗标   链接   信息   对齐
  [ 0]                   NULL             0000000000000000  00000000
       0000000000000000  0000000000000000           0     0     0
  [ 1] .text             PROGBITS         0000000000010120  00000120
       0000000000005ad8  0000000000000000  AX       0     0     4
  [ 2] .rodata           PROGBITS         0000000000015c00  00005c00
       0000000000000d20  0000000000000000   A       0     0     16
  [ 3] .eh_frame         PROGBITS         0000000000016920  00006920
       0000000000000004  0000000000000000   A       0     0     4
  [ 4] .init_array       INIT_ARRAY       0000000000017ff0  00006ff0
       0000000000000008  0000000000000008  WA       0     0     8
  [ 5] .fini_array       FINI_ARRAY       0000000000017ff8  00006ff8
       0000000000000008  0000000000000008  WA       0     0     8
  [ 6] .data             PROGBITS         0000000000018000  00007000
       00000000000000e8  0000000000000000  WA       0     0     8
  [ 7] .got              PROGBITS         00000000000180e8  000070e8
       0000000000000020  0000000000000008  WA       0     0     8
  [ 8] .sdata            PROGBITS         0000000000018108  00007108
       0000000000000038  0000000000000000  WA       0     0     8
  [ 9] .sbss             NOBITS           0000000000018140  00007140
       0000000000000044  0000000000000000  WA       0     0     8
  [10] .bss              NOBITS           0000000000018188  00007140
       0000000000000648  0000000000000000  WA       0     0     8
  [11] .comment          PROGBITS         0000000000000000  00007140
       000000000000002c  0000000000000001  MS       0     0     1
  [12] .debug_aranges    PROGBITS         0000000000000000  0000716c
       0000000000000200  0000000000000000           0     0     1
  [13] .debug_info       PROGBITS         0000000000000000  0000736c
       00000000000027c7  0000000000000000           0     0     1
  [14] .debug_abbrev     PROGBITS         0000000000000000  00009b33
       0000000000000e1d  0000000000000000           0     0     1
  [15] .debug_line       PROGBITS         0000000000000000  0000a950
       0000000000004c70  0000000000000000           0     0     1
  [16] .debug_frame      PROGBITS         0000000000000000  0000f5c0
       0000000000000260  0000000000000000           0     0     8
  [17] .debug_str        PROGBITS         0000000000000000  0000f820
       0000000000000835  0000000000000001  MS       0     0     1
  [18] .debug_loc        PROGBITS         0000000000000000  00010055
       0000000000007a92  0000000000000000           0     0     1
  [19] .debug_ranges     PROGBITS         0000000000000000  00017ae7
       0000000000001580  0000000000000000           0     0     1
  [20] .symtab           SYMTAB           0000000000000000  00019068
       0000000000001440  0000000000000018          21   115     8
  [21] .strtab           STRTAB           0000000000000000  0001a4a8
       000000000000080b  0000000000000000           0     0     1
  [22] .shstrtab         STRTAB           0000000000000000  0001acb3
       00000000000000d7  0000000000000000           0     0     1
Key to Flags:
  W (write), A (alloc), X (execute), M (merge), S (strings), I (info),
  L (link order), O (extra OS processing required), G (group), T (TLS),
  C (compressed), x (unknown), o (OS specific), E (exclude),
  D (mbind), p (processor specific)

There are no section groups in this file.

程序头：
  Type           Offset             VirtAddr           PhysAddr
                 FileSiz            MemSiz              Flags  Align
  LOAD           0x0000000000000000 0x0000000000010000 0x0000000000010000
                 0x0000000000006924 0x0000000000006924  R E    0x1000
  LOAD           0x0000000000006ff0 0x0000000000017ff0 0x0000000000017ff0
                 0x0000000000000150 0x00000000000007e0  RW     0x1000
  GNU_STACK      0x0000000000000000 0x0000000000000000 0x0000000000000000
                 0x0000000000000000 0x0000000000000000  RW     0x10
  GNU_RELRO      0x0000000000006ff0 0x0000000000017ff0 0x0000000000017ff0
                 0x0000000000000010 0x0000000000000010  R      0x1

 Section to Segment mapping:
  段节...
   00     .text .rodata .eh_frame 
   01     .init_array .fini_array .data .got .sdata .sbss .bss 
   02     
   03     .init_array .fini_array 

There is no dynamic section in this file.

该文件中没有重定位信息。

The decoding of unwind sections for machine type RISC-V is not currently supported.

Symbol table '.symtab' contains 216 entries:
   Num:    Value          Size Type    Bind   Vis      Ndx Name
     0: 0000000000000000     0 NOTYPE  LOCAL  DEFAULT  UND 
     1: 0000000000010120     0 SECTION LOCAL  DEFAULT    1 .text
     2: 0000000000015c00     0 SECTION LOCAL  DEFAULT    2 .rodata
     3: 0000000000016920     0 SECTION LOCAL  DEFAULT    3 .eh_frame
     4: 0000000000017ff0     0 SECTION LOCAL  DEFAULT    4 .init_array
     5: 0000000000017ff8     0 SECTION LOCAL  DEFAULT    5 .fini_array
     6: 0000000000018000     0 SECTION LOCAL  DEFAULT    6 .data
     7: 00000000000180e8     0 SECTION LOCAL  DEFAULT    7 .got
     8: 0000000000018108     0 SECTION LOCAL  DEFAULT    8 .sdata
     9: 0000000000018140     0 SECTION LOCAL  DEFAULT    9 .sbss
    10: 0000000000018188     0 SECTION LOCAL  DEFAULT   10 .bss
    11: 0000000000000000     0 SECTION LOCAL  DEFAULT   11 .comment
    12: 0000000000000000     0 SECTION LOCAL  DEFAULT   12 .debug_aranges
    13: 0000000000000000     0 SECTION LOCAL  DEFAULT   13 .debug_info
    14: 0000000000000000     0 SECTION LOCAL  DEFAULT   14 .debug_abbrev
    15: 0000000000000000     0 SECTION LOCAL  DEFAULT   15 .debug_line
    16: 0000000000000000     0 SECTION LOCAL  DEFAULT   16 .debug_frame
    17: 0000000000000000     0 SECTION LOCAL  DEFAULT   17 .debug_str
    18: 0000000000000000     0 SECTION LOCAL  DEFAULT   18 .debug_loc
    19: 0000000000000000     0 SECTION LOCAL  DEFAULT   19 .debug_ranges
    20: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS exit.c
    21: 000000000001067c     4 FUNC    LOCAL  DEFAULT    1 dummy
    22: 0000000000010680    72 FUNC    LOCAL  DEFAULT    1 libc_exit_fini
    23: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS crt1.c
    24: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS crtstuff.c
    25: 0000000000016920     0 OBJECT  LOCAL  DEFAULT    3 
    26: 0000000000010180     0 FUNC    LOCAL  DEFAULT    1 deregister_tm_clones
    27: 00000000000101a0     0 FUNC    LOCAL  DEFAULT    1 register_tm_clones
    28: 00000000000101d0     0 FUNC    LOCAL  DEFAULT    1 __do_global_dtors_aux
    29: 0000000000018188     1 OBJECT  LOCAL  DEFAULT   10 completed.1
    30: 0000000000017ff8     0 OBJECT  LOCAL  DEFAULT    5 __do_global_dtor[...]
    31: 0000000000010220     0 FUNC    LOCAL  DEFAULT    1 frame_dummy
    32: 0000000000018190    48 OBJECT  LOCAL  DEFAULT   10 object.0
    33: 0000000000017ff0     0 OBJECT  LOCAL  DEFAULT    4 __frame_dummy_in[...]
    34: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS time-test.c
    35: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS __libc_start_main.c
    36: 00000000000103d0     4 FUNC    LOCAL  DEFAULT    1 dummy
    37: 00000000000103d4     4 FUNC    LOCAL  DEFAULT    1 dummy1
    38: 0000000000010594    76 FUNC    LOCAL  DEFAULT    1 libc_start_init
    39: 00000000000105e0    68 FUNC    LOCAL  DEFAULT    1 libc_start_main_[...]
    40: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS defsysinfo.c
    41: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS libc.c
    42: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS printf.c
    43: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS stdout.c
    44: 0000000000018230  1032 OBJECT  LOCAL  DEFAULT   10 buf
    45: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS vfprintf.c
    46: 0000000000010710   260 FUNC    LOCAL  DEFAULT    1 pop_arg
    47: 0000000000010814    44 FUNC    LOCAL  DEFAULT    1 fmt_u
    48: 0000000000010840   104 FUNC    LOCAL  DEFAULT    1 getint
    49: 00000000000108a8    36 FUNC    LOCAL  DEFAULT    1 out
    50: 00000000000108cc   168 FUNC    LOCAL  DEFAULT    1 pad
    51: 0000000000010974  4252 FUNC    LOCAL  DEFAULT    1 fmt_fp
    52: 0000000000011a10  2344 FUNC    LOCAL  DEFAULT    1 printf_core
    53: 0000000000015e70   464 OBJECT  LOCAL  DEFAULT    2 states
    54: 0000000000016040    16 OBJECT  LOCAL  DEFAULT    2 xdigits
    55: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS memset.c
    56: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS strnlen.c
    57: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS clock_gettime.c
    58: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS __environ.c
    59: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS __init_tls.c
    60: 00000000000127b8   412 FUNC    LOCAL  DEFAULT    1 static_init_tls
    61: 0000000000018638   360 OBJECT  LOCAL  DEFAULT   10 builtin_tls
    62: 00000000000187a0    48 OBJECT  LOCAL  DEFAULT   10 main_tls
    63: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS __errno_location.c
    64: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS strerror.c
    65: 0000000000016050    89 OBJECT  LOCAL  DEFAULT    2 errid
    66: 00000000000160b0  1823 OBJECT  LOCAL  DEFAULT    2 errmsg
    67: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS _Exit.c
    68: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS syscall_ret.c
    69: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS __lctrans.c
    70: 0000000000012a08     4 FUNC    LOCAL  DEFAULT    1 dummy
    71: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS __fpclassifyl.c
    72: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS __signbitl.c
    73: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS frexpl.c
    74: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS wctomb.c
    75: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS __lockfile.c
    76: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS __stdio_close.c
    77: 0000000000012ca8     4 FUNC    LOCAL  DEFAULT    1 dummy
    78: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS __stdio_seek.c
    79: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS __stdout_write.c
    80: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS __towrite.c
    81: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS fwrite.c
    82: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS memchr.c
    83: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS memcpy.c
    84: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS default_attr.c
    85: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS lseek.c
    86: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS wcrtomb.c
    87: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS __stdio_exit.c
    88: 0000000000013578   132 FUNC    LOCAL  DEFAULT    1 close_file
    89: 0000000000018170     8 OBJECT  LOCAL  DEFAULT    9 dummy_file
    90: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS __stdio_write.c
    91: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS ofl.c
    92: 0000000000018178     8 OBJECT  LOCAL  DEFAULT    9 ofl_head
    93: 0000000000018180     4 OBJECT  LOCAL  DEFAULT    9 ofl_lock
    94: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS __lock.c
    95: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS addtf3.c
    96: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS eqtf2.c
    97: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS multf3.c
    98: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS subtf3.c
    99: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS fixtfsi.c
   100: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS fixunstfsi.c
   101: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS floatsitf.c
   102: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS floatunsitf.c
   103: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS extenddftf2.c
   104: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS libgcc2.c
   105: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS libgcc2.c
   106: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS crtstuff.c
   107: 0000000000016920     0 OBJECT  LOCAL  DEFAULT    3 __FRAME_END__
   108: 0000000000000000     0 FILE    LOCAL  DEFAULT  ABS 
   109: 0000000000018000     0 NOTYPE  LOCAL  DEFAULT    5 __fini_array_end
   110: 0000000000017ff8     0 NOTYPE  LOCAL  DEFAULT    5 __fini_array_start
   111: 0000000000017ff8     0 NOTYPE  LOCAL  DEFAULT    4 __init_array_end
   112: 0000000000016820   256 OBJECT  LOCAL  DEFAULT    2 __clz_tab
   113: 00000000000180f8     0 OBJECT  LOCAL  DEFAULT    7 _GLOBAL_OFFSET_TABLE_
   114: 0000000000017ff0     0 NOTYPE  LOCAL  DEFAULT    4 __init_array_start
   115: 0000000000015a3c   100 FUNC    GLOBAL HIDDEN     1 __floatunsitf
   116: 00000000000125ec   108 FUNC    WEAK   DEFAULT    1 clock_gettime
   117: 0000000000018168     4 OBJECT  GLOBAL HIDDEN     9 __thread_list_lock
   118: 0000000000018130     8 OBJECT  GLOBAL HIDDEN     8 __stdout_used
   119: 00000000000143e4   208 FUNC    GLOBAL HIDDEN     1 __eqtf2
   120: 00000000000106c8    72 FUNC    GLOBAL DEFAULT    1 printf
   121: 0000000000018800     0 NOTYPE  GLOBAL HIDDEN   ABS __global_pointer$
   122: 0000000000018120     8 OBJECT  GLOBAL DEFAULT    8 stdout
   123: 00000000000138ec  2808 FUNC    GLOBAL HIDDEN     1 __addtf3
   124: 0000000000012a10    16 FUNC    GLOBAL HIDDEN     1 __lctrans_cur
   125: 00000000000103d8   444 FUNC    GLOBAL HIDDEN     1 __init_libc
   126: 00000000000129d0    56 FUNC    GLOBAL HIDDEN     1 __syscall_ret
   127: 00000000000159b4   136 FUNC    GLOBAL HIDDEN     1 __floatsitf
   128: 00000000000129ac    12 FUNC    GLOBAL DEFAULT    1 strerror
   129: 0000000000012ce0   108 FUNC    GLOBAL HIDDEN     1 __stdout_write
   130: 0000000000013778     8 FUNC    GLOBAL HIDDEN     1 __ofl_unlock
   131: 0000000000012c48    96 FUNC    GLOBAL HIDDEN     1 __unlockfile
   132: 0000000000018148     8 OBJECT  GLOBAL HIDDEN     9 __hwcap
   133: 0000000000018108     0 NOTYPE  GLOBAL DEFAULT    8 __SDATA_BEGIN__
   134: 0000000000012a0c     4 FUNC    GLOBAL HIDDEN     1 __lctrans
   135: 0000000000013648   272 FUNC    GLOBAL HIDDEN     1 __stdio_write
   136: 0000000000012a20    68 FUNC    GLOBAL DEFAULT    1 __fpclassifyl
   137: 0000000000012d4c    84 FUNC    GLOBAL HIDDEN     1 __towrite
   138: 0000000000012fec  1092 FUNC    GLOBAL DEFAULT    1 memcpy
   139: 0000000000013758    32 FUNC    GLOBAL HIDDEN     1 __ofl_lock
   140: 00000000000180e8     0 OBJECT  GLOBAL HIDDEN     7 __TMC_END__
   141: 0000000000013878   116 FUNC    GLOBAL HIDDEN     1 __unlock
   142: 00000000000181c0   112 OBJECT  GLOBAL HIDDEN    10 __libc
   143: 0000000000018128     0 OBJECT  GLOBAL HIDDEN     8 __dso_handle
   144: 0000000000012b48    44 FUNC    GLOBAL DEFAULT    1 wctomb
   145: 0000000000013430     0 FUNC    GLOBAL HIDDEN     1 __set_thread_area
   146: 0000000000018170     8 OBJECT  WEAK   HIDDEN     9 __stdin_used
   147: 00000000000126d8   224 FUNC    GLOBAL HIDDEN     1 __copy_tls
   148: 0000000000012cd8     8 FUNC    GLOBAL HIDDEN     1 __stdio_seek
   149: 0000000000018160     8 OBJECT  WEAK   DEFAULT    9 _environ
   150: 0000000000012b74   212 FUNC    GLOBAL HIDDEN     1 __lockfile
   151: 0000000000012954    12 FUNC    WEAK   HIDDEN     1 ___errno_location
   152: 000000000001343c    12 FUNC    WEAK   DEFAULT    1 lseek
   153: 0000000000018160     8 OBJECT  GLOBAL DEFAULT    9 __environ
   154: 00000000000129b8    24 FUNC    GLOBAL DEFAULT    1 _Exit
   155: 0000000000012da0     4 FUNC    GLOBAL HIDDEN     1 __towrite_needs_[...]
   156: 00000000000127b8   412 FUNC    WEAK   HIDDEN     1 __init_tls
   157: 00000000000103d0     4 FUNC    WEAK   DEFAULT    1 _init
   158: 00000000000125a8    68 FUNC    GLOBAL DEFAULT    1 strnlen
   159: 0000000000012a08     4 FUNC    WEAK   HIDDEN     1 __lctrans_impl
   160: 000000000001067c     4 FUNC    WEAK   HIDDEN     1 __funcs_on_exit
   161: 00000000000102a8    84 FUNC    GLOBAL DEFAULT    1 iter
   162: 0000000000010250    88 FUNC    GLOBAL DEFAULT    1 now_ns
   163: 0000000000012960    76 FUNC    WEAK   DEFAULT    1 strerror_l
   164: 0000000000018160     8 OBJECT  WEAK   DEFAULT    9 environ
   165: 000000000001582c   212 FUNC    GLOBAL HIDDEN     1 __fixtfsi
   166: 0000000000012f58   148 FUNC    GLOBAL DEFAULT    1 memchr
   167: 0000000000015900   180 FUNC    GLOBAL HIDDEN     1 __fixunstfsi
   168: 0000000000018160     8 OBJECT  WEAK   DEFAULT    9 ___environ
   169: 0000000000018150     8 OBJECT  GLOBAL DEFAULT    9 __progname
   170: 0000000000010140     0 FUNC    GLOBAL DEFAULT    1 _start
   171: 0000000000010158    40 FUNC    GLOBAL DEFAULT    1 _start_c
   172: 00000000000143e4   208 FUNC    GLOBAL HIDDEN     1 __netf2
   173: 0000000000018000   232 OBJECT  GLOBAL HIDDEN     6 __stdout_FILE
   174: 0000000000018150     8 OBJECT  WEAK   DEFAULT    9 program_invocati[...]
   175: 0000000000010594    76 FUNC    WEAK   HIDDEN     1 __libc_start_init
   176: 0000000000015aa0   284 FUNC    GLOBAL HIDDEN     1 __extenddftf2
   177: 0000000000012658   128 FUNC    GLOBAL HIDDEN     1 __init_tp
   178: 00000000000103d4     4 FUNC    WEAK   HIDDEN     1 __init_ssp
   179: 00000000000187d0     0 NOTYPE  GLOBAL DEFAULT   10 __BSS_END__
   180: 0000000000012da4   268 FUNC    GLOBAL HIDDEN     1 __fwritex
   181: 0000000000018140     0 NOTYPE  GLOBAL DEFAULT    9 __bss_start
   182: 00000000000124b0   248 FUNC    GLOBAL DEFAULT    1 memset
   183: 00000000000102fc   212 FUNC    GLOBAL DEFAULT    1 main
   184: 00000000000135fc    76 FUNC    GLOBAL HIDDEN     1 __stdio_exit
   185: 0000000000013780   248 FUNC    GLOBAL HIDDEN     1 __lock
   186: 0000000000012a64     8 FUNC    GLOBAL DEFAULT    1 __signbitl
   187: 0000000000012ca8     4 FUNC    WEAK   DEFAULT    1 __aio_close
   188: 000000000001343c    12 FUNC    GLOBAL HIDDEN     1 __lseek
   189: 00000000000144b4  2168 FUNC    GLOBAL HIDDEN     1 __multf3
   190: 000000000001067c     4 FUNC    WEAK   DEFAULT    1 _fini
   191: 0000000000014d2c  2816 FUNC    GLOBAL HIDDEN     1 __subtf3
   192: 0000000000010680    72 FUNC    WEAK   HIDDEN     1 __libc_exit_fini
   193: 0000000000012eb0   168 FUNC    WEAK   DEFAULT    1 fwrite_unlocked
   194: 0000000000018000     0 NOTYPE  GLOBAL DEFAULT    6 __DATA_BEGIN__
   195: 0000000000012eb0   168 FUNC    GLOBAL DEFAULT    1 fwrite
   196: 0000000000018140     0 NOTYPE  GLOBAL DEFAULT    8 _edata
   197: 00000000000187d0     0 NOTYPE  GLOBAL DEFAULT   10 _end
   198: 00000000000125ec   108 FUNC    GLOBAL HIDDEN     1 __clock_gettime
   199: 0000000000012cac    44 FUNC    GLOBAL HIDDEN     1 __stdio_close
   200: 0000000000012954    12 FUNC    GLOBAL DEFAULT    1 __errno_location
   201: 0000000000012960    76 FUNC    GLOBAL DEFAULT    1 __strerror_l
   202: 0000000000010120    32 FUNC    GLOBAL DEFAULT    1 exit
   203: 0000000000018170     8 OBJECT  WEAK   HIDDEN     9 __stderr_used
   204: 00000000000135fc    76 FUNC    WEAK   HIDDEN     1 __stdio_exit_needed
   205: 0000000000012a6c   220 FUNC    GLOBAL DEFAULT    1 frexpl
   206: 0000000000010624    88 FUNC    GLOBAL DEFAULT    1 __libc_start_main
   207: 000000000001343c    12 FUNC    WEAK   DEFAULT    1 lseek64
   208: 0000000000018158     8 OBJECT  WEAK   DEFAULT    9 program_invocati[...]
   209: 000000000001813c     4 OBJECT  GLOBAL HIDDEN     8 __default_stacksize
   210: 0000000000018138     4 OBJECT  GLOBAL HIDDEN     8 __default_guardsize
   211: 0000000000015bbc    60 FUNC    GLOBAL HIDDEN     1 __clzdi2
   212: 0000000000013448   304 FUNC    GLOBAL DEFAULT    1 wcrtomb
   213: 0000000000018140     8 OBJECT  GLOBAL HIDDEN     9 __sysinfo
   214: 0000000000012338   376 FUNC    GLOBAL DEFAULT    1 vfprintf
   215: 0000000000018158     8 OBJECT  GLOBAL DEFAULT    9 __progname_full

No version information found in this file.
