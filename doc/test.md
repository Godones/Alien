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
- [ ] getpwnam_r_crash    [socket+connect]
- [ ] getpwnam_r_errno    [socket+connect]
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
- [ ] socket
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







## libctest-dyn

1. argv
2. basename
3. clocale_mbfuncs
4. clock_gettime
5. crypt
6. daemon_failure
7. dirname
8. dn_expand_empty
9. dn_expand_ptr_0
10. env
11. fdopen
12. fflush_exit
13. fgets_eof
14. fgetwc_buffering
15. fnmatch
16. fpclassify_invalid_ld80
17. fscanf
18. ftello_unflushed_append
19. fwscanf
20. getpwnam_r_crash    [socket+connect]
21. getpwnam_r_errno    [socket+connect]
22. iconv_open
23. iconv_roundtrips
24. inet_ntop_v4mapped
25. inet_pton
26. inet_pton_empty_last_field
27. iswspace_null
28. lrand48_signextend
29. lseek_large
30. malloc_0
31. mbc
32. mbsrtowcs_overflow
33. memmem_oob
34. memmem_oob_read
35. memstream
36. mkdtemp_failure
37. mkstemp_failure
38. pleval
39. printf_1e9_oob
40. printf_fmt_g_round
41. printf_fmt_g_zeros
42. printf_fmt_n
43. pthread_cancel
44. pthread_cancel_points
45. pthread_cancel_sem_wait
46. pthread_cond
47. pthread_condattr_setclock
48. pthread_cond_smasher
49. pthread_exit_cancel
50. pthread_once_deadlock
51. pthread_robust_detach
52. pthread_rwlock_ebusy
53. pthread_tsd
54. putenv_doublefree
55. qsort
56. random
57. regex_backref_0
58. regex_bracket_icase
59. regexec_nosub
60. regex_ere_backref
61. regex_escaped_high_byte
62. regex_negated_range
63. rewind_clear_error
64. rlimit_open_files
65. scanf_bytes_consumed
66. scanf_match_literal_eof
67. scanf_nullbyte_char
68. search_hsearch
69. search_insque
70. search_lsearch
71. search_tsearch
72. setjmp
73. setvbuf_unget
74. sigprocmask_internal
75. snprintf
76. socket
77. sscanf
78. sscanf_eof
79. sscanf_long
80. stat
81. statvfs
82. strftime
83. string
84. string_memcpy
85. string_memmem
86. string_memset
87. string_strchr
88. string_strcspn
89. string_strstr
90. strptime
91. strtod
92. strtod_simple
93. strtof
94. strtol
95. strtold
96. strverscmp
97. swprintf
98. syscall_sign_extend
99. tgmath
100. time
101. udiv
102. ungetc
103. uselocale_0
104. utime
105. wcsncpy_read_overflow
106. wcsstr
107. wcsstr_false_negative
108. wcstol



## 工具配置

[Rust cross compilation](https://danielmangum.com/posts/risc-v-bytes-rust-cross-compilation/)

[opensibi](https://tinylab.org/riscv-opensbi-quickstart/)

[Rust - OSDev Wiki](https://wiki.osdev.org/Rust)

[file system function](https://www.cnblogs.com/XNQC1314/p/9251197.html)



