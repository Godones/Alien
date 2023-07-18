# 测试

## simple

- [x] wait
- [x] waitpid
- [x] write
- [x] yield
- [x] times
- [x] sleep
- [x] read
- [x] uname
- [x] openat
- [x] open
- [x] mkdir
- [x] gettimeofday
- [x] getpid
- [x] getppid
- [x] getcwd
- [x] fstat
- [x] fork
- [x] exit
- [x] execve
- [x] close
- [x] chdir
- [x] pipe
- [x] dup
- [x] dup2
- [x] unlink
- [x] brk
- [x] mount
- [x] getdents
- [x] munmap
- [x] clone

## libctest-static

- [x] argv
- [x] basename
- [x] clocale_mbfuncs
- [x] clock_gettime
- [x] crypt
- [x] daemon_failure
- [x] dirname
- [x] dn_expand_empty
- [x] dn_expand_ptr_0
- [x] env
- [x] fdopen
- [x] fflush_exit
- [x] fgets_eof
- [x] fgetwc_buffering
- [x] fnmatch
- [x] fpclassify_invalid_ld80
- [x] fscanf
- [x] ftello_unflushed_append
- [x] fwscanf
- [x] getpwnam_r_crash    [socket+connect]
- [x] getpwnam_r_errno    [socket+connect]
- [x] iconv_open
- [x] iconv_roundtrips
- [x] inet_ntop_v4mapped
- [x] inet_pton
- [x] inet_pton_empty_last_field
- [x] iswspace_null
- [x] lrand48_signextend
- [x] lseek_large
- [x] malloc_0
- [x] mbc
- [x] mbsrtowcs_overflow
- [x] memmem_oob
- [x] memmem_oob_read
- [x] memstream
- [x] mkdtemp_failure
- [x] mkstemp_failure
- [x] pleval
- [x] printf_1e9_oob
- [x] printf_fmt_g_round
- [x] printf_fmt_g_zeros
- [x] printf_fmt_n
- [x] pthread_cancel
- [x] pthread_cancel_points
- [x] pthread_cancel_sem_wait
- [x] pthread_cond
- [x] pthread_condattr_setclock
- [x] pthread_cond_smasher
- [x] pthread_exit_cancel
- [x] pthread_once_deadlock
- [x] pthread_robust_detach
- [x] pthread_rwlock_ebusy
- [x] pthread_tsd
- [x] putenv_doublefree
- [x] qsort
- [x] random
- [x] regex_backref_0
- [x] regex_bracket_icase
- [x] regexec_nosub
- [x] regex_ere_backref
- [x] regex_escaped_high_byte
- [x] regex_negated_range
- [x] rewind_clear_error
- [x] rlimit_open_files
- [x] scanf_bytes_consumed
- [x] scanf_match_literal_eof
- [x] scanf_nullbyte_char
- [x] search_hsearch
- [x] search_insque
- [x] search_lsearch
- [x] search_tsearch
- [x] setjmp
- [x] setvbuf_unget
- [x] sigprocmask_internal
- [x] snprintf
- [x] socket
- [x] sscanf
- [x] sscanf_eof
- [x] sscanf_long
- [x] stat
- [x] statvfs
- [x] strftime
- [x] string
- [x] string_memcpy
- [x] string_memmem
- [x] string_memset
- [x] string_strchr
- [x] string_strcspn
- [x] string_strstr
- [x] strptime
- [x] strtod
- [x] strtod_simple
- [x] strtof
- [x] strtol
- [x] strtold
- [x] strverscmp
- [x] swprintf
- [x] syscall_sign_extend
- [x] tgmath
- [x] time
- [x] udiv
- [x] ungetc
- [x] uselocale_0
- [x] utime
- [x] wcsncpy_read_overflow
- [x] wcsstr
- [x] wcsstr_false_negative
- [x] wcstol

```
./runtest.exe -w entry-static.exe pthread_cancel
./runtest.exe -w entry-static.exe pthread_cancel_points
./runtest.exe -w entry-static.exe pthread_cond
./runtest.exe -w entry-static.exe pthread_tsd
./runtest.exe -w entry-static.exe pthread_robust_detach
./runtest.exe -w entry-static.exe pthread_cancel_sem_wait
./runtest.exe -w entry-static.exe pthread_cond_smasher
./runtest.exe -w entry-static.exe pthread_condattr_setclock
./runtest.exe -w entry-static.exe pthread_exit_cancel
./runtest.exe -w entry-static.exe pthread_once_deadlock
./runtest.exe -w entry-static.exe pthread_rwlock_ebusy
```





## libctest-dyn



## other

- [x] busybox
- [x] lua
- [x] time-test
- [x] bash
- [x] iozone
  - [x] ./iozone -a -r 1k -s 4m
  - [x] ./iozone -t 4 -i 0 -i 1 -r 1k -s 1m
  - [x] ./iozone -t 4 -i 0 -i 2 -r 1k -s 1m
  - [x] ./iozone -t 4 -i 0 -i 3 -r 1k -s 1m
  - [x] ./iozone -t 4 -i 0 -i 5 -r 1k -s 1m
  - [x] ./iozone -t 4 -i 6 -i 7 -r 1k -s 1m
  - [x] ./iozone -t 4 -i 9 -i 10 -r 1k -s 1m
  - [x] ./iozone -t 4 -i 11 -i 12 -r 1k -s 1m
- [ ] unixbench
- [ ] netperf
  - [x] ./busybox sh netperf_testcode2.sh
  - [ ] 

- [ ] iperf





## 工具配置

[Rust cross compilation](https://danielmangum.com/posts/risc-v-bytes-rust-cross-compilation/)

[opensibi](https://tinylab.org/riscv-opensbi-quickstart/)

[Rust - OSDev Wiki](https://wiki.osdev.org/Rust)

[file system function](https://www.cnblogs.com/XNQC1314/p/9251197.html)



