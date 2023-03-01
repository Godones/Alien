
target/riscv64gc-unknown-none-elf/release/hello：     文件格式 elf64-littleriscv


Disassembly of section .text:

0000000000010000 <main>:
   10000:	7139                	addi	sp,sp,-64
   10002:	fc06                	sd	ra,56(sp)

0000000000010004 <.LBB0_1>:
   10004:	00002517          	auipc	a0,0x2
   10008:	00c50513          	addi	a0,a0,12 # 12010 <.Lanon.fad58de7366495db4650cfefac2fcd61.2>
   1000c:	ec2a                	sd	a0,24(sp)
   1000e:	4505                	li	a0,1
   10010:	f02a                	sd	a0,32(sp)
   10012:	e402                	sd	zero,8(sp)

0000000000010014 <.LBB0_2>:
   10014:	00002517          	auipc	a0,0x2
   10018:	fec50513          	addi	a0,a0,-20 # 12000 <.Lanon.fad58de7366495db4650cfefac2fcd61.1>
   1001c:	f42a                	sd	a0,40(sp)
   1001e:	f802                	sd	zero,48(sp)
   10020:	0028                	addi	a0,sp,8
   10022:	00000097          	auipc	ra,0x0
   10026:	290080e7          	jalr	656(ra) # 102b2 <_ZN7userlib2io7__print17hcc553115631cb938E>
   1002a:	70e2                	ld	ra,56(sp)
   1002c:	6121                	addi	sp,sp,64
   1002e:	8082                	ret

0000000000010030 <init_heap>:
   10030:	711d                	addi	sp,sp,-96
   10032:	ec86                	sd	ra,88(sp)
   10034:	e8a2                	sd	s0,80(sp)
   10036:	e4a6                	sd	s1,72(sp)
   10038:	e0ca                	sd	s2,64(sp)
   1003a:	fc4e                	sd	s3,56(sp)
   1003c:	f852                	sd	s4,48(sp)
   1003e:	f456                	sd	s5,40(sp)
   10040:	f05a                	sd	s6,32(sp)
   10042:	ec5e                	sd	s7,24(sp)
   10044:	e862                	sd	s8,16(sp)
   10046:	e466                	sd	s9,8(sp)
   10048:	e06a                	sd	s10,0(sp)

000000000001004a <.LBB0_14>:
   1004a:	00007997          	auipc	s3,0x7
   1004e:	2ae98993          	addi	s3,s3,686 # 172f8 <_ZN7userlib5alloc4HEAP17h573c842b00e3a432E.llvm.6931228215858126918>
   10052:	4505                	li	a0,1
   10054:	00a9b92f          	amoadd.d	s2,a0,(s3)
   10058:	0089b503          	ld	a0,8(s3)
   1005c:	0230000f          	fence	r,rw
   10060:	01250a63          	beq	a0,s2,10074 <.LBB0_15>
   10064:	0100000f          	fence	w,unknown
   10068:	0089b503          	ld	a0,8(s3)
   1006c:	0230000f          	fence	r,rw
   10070:	ff251ae3          	bne	a0,s2,10064 <.LBB0_14+0x1a>

0000000000010074 <.LBB0_15>:
   10074:	00003517          	auipc	a0,0x3
   10078:	28450513          	addi	a0,a0,644 # 132f8 <_ZN7userlib5alloc10HEAP_SPACE17h114e27e1461dc493E.llvm.6931228215858126918>
   1007c:	6591                	lui	a1,0x4
   1007e:	95aa                	add	a1,a1,a0
   10080:	99e1                	andi	a1,a1,-8
   10082:	00750613          	addi	a2,a0,7
   10086:	ff867413          	andi	s0,a2,-8
   1008a:	1085e363          	bltu	a1,s0,10190 <.LBB0_21>
   1008e:	4d01                	li	s10,0
   10090:	00840613          	addi	a2,s0,8
   10094:	0ac5ea63          	bltu	a1,a2,10148 <.LBB0_19+0x8a>
   10098:	6591                	lui	a1,0x4

000000000001009a <.LBB0_16>:
   1009a:	00003617          	auipc	a2,0x3
   1009e:	f6660613          	addi	a2,a2,-154 # 13000 <.LCPI0_0>
   100a2:	00063a03          	ld	s4,0(a2)

00000000000100a6 <.LBB0_17>:
   100a6:	00003617          	auipc	a2,0x3
   100aa:	f6260613          	addi	a2,a2,-158 # 13008 <.LCPI0_1>
   100ae:	00063a83          	ld	s5,0(a2)

00000000000100b2 <.LBB0_18>:
   100b2:	00003617          	auipc	a2,0x3
   100b6:	f5e60613          	addi	a2,a2,-162 # 13010 <.LCPI0_2>
   100ba:	00063b03          	ld	s6,0(a2)

00000000000100be <.LBB0_19>:
   100be:	00003617          	auipc	a2,0x3
   100c2:	f5a60613          	addi	a2,a2,-166 # 13018 <.LCPI0_3>
   100c6:	00063b83          	ld	s7,0(a2)
   100ca:	952e                	add	a0,a0,a1
   100cc:	ff857c13          	andi	s8,a0,-8
   100d0:	4cfd                	li	s9,31
   100d2:	40800533          	neg	a0,s0
   100d6:	00a474b3          	and	s1,s0,a0
   100da:	408c0533          	sub	a0,s8,s0
   100de:	00000097          	auipc	ra,0x0
   100e2:	3ba080e7          	jalr	954(ra) # 10498 <_ZN22buddy_system_allocator17prev_power_of_two17he8186359febea13fE>
   100e6:	00a4e363          	bltu	s1,a0,100ec <.LBB0_19+0x2e>
   100ea:	84aa                	mv	s1,a0
   100ec:	cc85                	beqz	s1,10124 <.LBB0_19+0x66>
   100ee:	fff48513          	addi	a0,s1,-1
   100f2:	fff4c593          	not	a1,s1
   100f6:	8d6d                	and	a0,a0,a1
   100f8:	00155593          	srli	a1,a0,0x1
   100fc:	0145f5b3          	and	a1,a1,s4
   10100:	8d0d                	sub	a0,a0,a1
   10102:	015575b3          	and	a1,a0,s5
   10106:	8109                	srli	a0,a0,0x2
   10108:	01557533          	and	a0,a0,s5
   1010c:	952e                	add	a0,a0,a1
   1010e:	00455593          	srli	a1,a0,0x4
   10112:	952e                	add	a0,a0,a1
   10114:	01657533          	and	a0,a0,s6
   10118:	03750533          	mul	a0,a0,s7
   1011c:	9161                	srli	a0,a0,0x38
   1011e:	00acf763          	bgeu	s9,a0,1012c <.LBB0_19+0x6e>
   10122:	a8a1                	j	1017a <.LBB0_20>
   10124:	04000513          	li	a0,64
   10128:	04ace963          	bltu	s9,a0,1017a <.LBB0_20>
   1012c:	9d26                	add	s10,s10,s1
   1012e:	050e                	slli	a0,a0,0x3
   10130:	954e                	add	a0,a0,s3
   10132:	0541                	addi	a0,a0,16
   10134:	85a2                	mv	a1,s0
   10136:	00000097          	auipc	ra,0x0
   1013a:	3ea080e7          	jalr	1002(ra) # 10520 <_ZN22buddy_system_allocator11linked_list10LinkedList4push17h6a533afaa3e8dc6bE>
   1013e:	9426                	add	s0,s0,s1
   10140:	00840513          	addi	a0,s0,8
   10144:	f8ac77e3          	bgeu	s8,a0,100d2 <.LBB0_19+0x14>
   10148:	1209b503          	ld	a0,288(s3)
   1014c:	956a                	add	a0,a0,s10
   1014e:	12a9b023          	sd	a0,288(s3)
   10152:	00190513          	addi	a0,s2,1
   10156:	0310000f          	fence	rw,w
   1015a:	00a9b423          	sd	a0,8(s3)
   1015e:	60e6                	ld	ra,88(sp)
   10160:	6446                	ld	s0,80(sp)
   10162:	64a6                	ld	s1,72(sp)
   10164:	6906                	ld	s2,64(sp)
   10166:	79e2                	ld	s3,56(sp)
   10168:	7a42                	ld	s4,48(sp)
   1016a:	7aa2                	ld	s5,40(sp)
   1016c:	7b02                	ld	s6,32(sp)
   1016e:	6be2                	ld	s7,24(sp)
   10170:	6c42                	ld	s8,16(sp)
   10172:	6ca2                	ld	s9,8(sp)
   10174:	6d02                	ld	s10,0(sp)
   10176:	6125                	addi	sp,sp,96
   10178:	8082                	ret

000000000001017a <.LBB0_20>:
   1017a:	00002617          	auipc	a2,0x2
   1017e:	f4e60613          	addi	a2,a2,-178 # 120c8 <anon.1c93528cad3da575e6989ee989895b5f.3.llvm.13359080111618654021>
   10182:	02000593          	li	a1,32
   10186:	00000097          	auipc	ra,0x0
   1018a:	416080e7          	jalr	1046(ra) # 1059c <_ZN4core9panicking18panic_bounds_check17h7918fc3ccbae3e2fE>
	...

0000000000010190 <.LBB0_21>:
   10190:	00002517          	auipc	a0,0x2
   10194:	e9050513          	addi	a0,a0,-368 # 12020 <anon.1c93528cad3da575e6989ee989895b5f.0.llvm.13359080111618654021>

0000000000010198 <.LBB0_22>:
   10198:	00002617          	auipc	a2,0x2
   1019c:	f1860613          	addi	a2,a2,-232 # 120b0 <anon.1c93528cad3da575e6989ee989895b5f.2.llvm.13359080111618654021>
   101a0:	45f9                	li	a1,30
   101a2:	00000097          	auipc	ra,0x0
   101a6:	3ce080e7          	jalr	974(ra) # 10570 <_ZN4core9panicking5panic17h597b4f53f5061709E>
	...

00000000000101ac <_ZN4core3ptr37drop_in_place$LT$core..fmt..Error$GT$17h5835390175285eccE.llvm.4825624523723557213>:
   101ac:	8082                	ret

00000000000101ae <_ZN50_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write$GT$10write_char17h85f3c2e72e5958b3E.llvm.4825624523723557213>:
   101ae:	1141                	addi	sp,sp,-16
   101b0:	0005851b          	sext.w	a0,a1
   101b4:	08000613          	li	a2,128
   101b8:	c602                	sw	zero,12(sp)
   101ba:	00c57663          	bgeu	a0,a2,101c6 <_ZN50_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write$GT$10write_char17h85f3c2e72e5958b3E.llvm.4825624523723557213+0x18>
   101be:	00b10623          	sb	a1,12(sp)
   101c2:	4605                	li	a2,1
   101c4:	a851                	j	10258 <_ZN50_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write$GT$10write_char17h85f3c2e72e5958b3E.llvm.4825624523723557213+0xaa>
   101c6:	00b5d51b          	srliw	a0,a1,0xb
   101ca:	ed19                	bnez	a0,101e8 <_ZN50_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write$GT$10write_char17h85f3c2e72e5958b3E.llvm.4825624523723557213+0x3a>
   101cc:	0065d513          	srli	a0,a1,0x6
   101d0:	0c056513          	ori	a0,a0,192
   101d4:	00a10623          	sb	a0,12(sp)
   101d8:	03f5f513          	andi	a0,a1,63
   101dc:	08056513          	ori	a0,a0,128
   101e0:	00a106a3          	sb	a0,13(sp)
   101e4:	4609                	li	a2,2
   101e6:	a88d                	j	10258 <_ZN50_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write$GT$10write_char17h85f3c2e72e5958b3E.llvm.4825624523723557213+0xaa>
   101e8:	0105d51b          	srliw	a0,a1,0x10
   101ec:	e905                	bnez	a0,1021c <_ZN50_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write$GT$10write_char17h85f3c2e72e5958b3E.llvm.4825624523723557213+0x6e>
   101ee:	02059513          	slli	a0,a1,0x20
   101f2:	9101                	srli	a0,a0,0x20
   101f4:	00c5d61b          	srliw	a2,a1,0xc
   101f8:	0e066613          	ori	a2,a2,224
   101fc:	00c10623          	sb	a2,12(sp)
   10200:	1552                	slli	a0,a0,0x34
   10202:	9169                	srli	a0,a0,0x3a
   10204:	08056513          	ori	a0,a0,128
   10208:	00a106a3          	sb	a0,13(sp)
   1020c:	03f5f513          	andi	a0,a1,63
   10210:	08056513          	ori	a0,a0,128
   10214:	00a10723          	sb	a0,14(sp)
   10218:	460d                	li	a2,3
   1021a:	a83d                	j	10258 <_ZN50_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write$GT$10write_char17h85f3c2e72e5958b3E.llvm.4825624523723557213+0xaa>
   1021c:	02059513          	slli	a0,a1,0x20
   10220:	9101                	srli	a0,a0,0x20
   10222:	02b51613          	slli	a2,a0,0x2b
   10226:	9275                	srli	a2,a2,0x3d
   10228:	0f066613          	ori	a2,a2,240
   1022c:	00c10623          	sb	a2,12(sp)
   10230:	02e51613          	slli	a2,a0,0x2e
   10234:	9269                	srli	a2,a2,0x3a
   10236:	08066613          	ori	a2,a2,128
   1023a:	00c106a3          	sb	a2,13(sp)
   1023e:	1552                	slli	a0,a0,0x34
   10240:	9169                	srli	a0,a0,0x3a
   10242:	08056513          	ori	a0,a0,128
   10246:	00a10723          	sb	a0,14(sp)
   1024a:	03f5f513          	andi	a0,a1,63
   1024e:	08056513          	ori	a0,a0,128
   10252:	00a107a3          	sb	a0,15(sp)
   10256:	4611                	li	a2,4
   10258:	4505                	li	a0,1
   1025a:	006c                	addi	a1,sp,12
   1025c:	04000893          	li	a7,64
   10260:	00000073          	ecall
   10264:	4501                	li	a0,0
   10266:	0141                	addi	sp,sp,16
   10268:	8082                	ret

000000000001026a <_ZN50_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write$GT$9write_fmt17h0c85e093eeb846e5E.llvm.4825624523723557213>:
   1026a:	7139                	addi	sp,sp,-64
   1026c:	fc06                	sd	ra,56(sp)
   1026e:	6108                	ld	a0,0(a0)
   10270:	7590                	ld	a2,40(a1)
   10272:	7194                	ld	a3,32(a1)
   10274:	e02a                	sd	a0,0(sp)
   10276:	f832                	sd	a2,48(sp)
   10278:	f436                	sd	a3,40(sp)
   1027a:	6d88                	ld	a0,24(a1)
   1027c:	6990                	ld	a2,16(a1)
   1027e:	6594                	ld	a3,8(a1)
   10280:	618c                	ld	a1,0(a1)
   10282:	f02a                	sd	a0,32(sp)
   10284:	ec32                	sd	a2,24(sp)
   10286:	e836                	sd	a3,16(sp)
   10288:	e42e                	sd	a1,8(sp)

000000000001028a <.LBB2_1>:
   1028a:	00002597          	auipc	a1,0x2
   1028e:	e5658593          	addi	a1,a1,-426 # 120e0 <anon.f66e08f6b2d961e480fb3b12cb7622cb.0.llvm.4825624523723557213>
   10292:	850a                	mv	a0,sp
   10294:	0030                	addi	a2,sp,8
   10296:	00000097          	auipc	ra,0x0
   1029a:	3c4080e7          	jalr	964(ra) # 1065a <_ZN4core3fmt5write17he707b088ca7ea77bE>
   1029e:	70e2                	ld	ra,56(sp)
   102a0:	6121                	addi	sp,sp,64
   102a2:	8082                	ret

00000000000102a4 <_ZN50_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write$GT$9write_str17h56308d6d2b6fb9c6E.llvm.4825624523723557213>:
   102a4:	4505                	li	a0,1
   102a6:	04000893          	li	a7,64
   102aa:	00000073          	ecall
   102ae:	4501                	li	a0,0
   102b0:	8082                	ret

00000000000102b2 <_ZN7userlib2io7__print17hcc553115631cb938E>:
   102b2:	715d                	addi	sp,sp,-80
   102b4:	e486                	sd	ra,72(sp)
   102b6:	750c                	ld	a1,40(a0)
   102b8:	7110                	ld	a2,32(a0)
   102ba:	0094                	addi	a3,sp,64
   102bc:	e436                	sd	a3,8(sp)
   102be:	fc2e                	sd	a1,56(sp)
   102c0:	f832                	sd	a2,48(sp)
   102c2:	6d0c                	ld	a1,24(a0)
   102c4:	6910                	ld	a2,16(a0)
   102c6:	6514                	ld	a3,8(a0)
   102c8:	6108                	ld	a0,0(a0)
   102ca:	f42e                	sd	a1,40(sp)
   102cc:	f032                	sd	a2,32(sp)
   102ce:	ec36                	sd	a3,24(sp)
   102d0:	e82a                	sd	a0,16(sp)

00000000000102d2 <.LBB5_3>:
   102d2:	00002597          	auipc	a1,0x2
   102d6:	e0e58593          	addi	a1,a1,-498 # 120e0 <anon.f66e08f6b2d961e480fb3b12cb7622cb.0.llvm.4825624523723557213>
   102da:	0028                	addi	a0,sp,8
   102dc:	0810                	addi	a2,sp,16
   102de:	00000097          	auipc	ra,0x0
   102e2:	37c080e7          	jalr	892(ra) # 1065a <_ZN4core3fmt5write17he707b088ca7ea77bE>
   102e6:	e501                	bnez	a0,102ee <.LBB5_4>
   102e8:	60a6                	ld	ra,72(sp)
   102ea:	6161                	addi	sp,sp,80
   102ec:	8082                	ret

00000000000102ee <.LBB5_4>:
   102ee:	00002517          	auipc	a0,0x2
   102f2:	e2250513          	addi	a0,a0,-478 # 12110 <anon.f66e08f6b2d961e480fb3b12cb7622cb.1.llvm.4825624523723557213>

00000000000102f6 <.LBB5_5>:
   102f6:	00002697          	auipc	a3,0x2
   102fa:	e4a68693          	addi	a3,a3,-438 # 12140 <anon.f66e08f6b2d961e480fb3b12cb7622cb.2.llvm.4825624523723557213>

00000000000102fe <.LBB5_6>:
   102fe:	00002717          	auipc	a4,0x2
   10302:	e9a70713          	addi	a4,a4,-358 # 12198 <anon.f66e08f6b2d961e480fb3b12cb7622cb.4.llvm.4825624523723557213>
   10306:	02b00593          	li	a1,43
   1030a:	0090                	addi	a2,sp,64
   1030c:	00000097          	auipc	ra,0x0
   10310:	2d0080e7          	jalr	720(ra) # 105dc <_ZN4core6result13unwrap_failed17h5e58f0c34337c8fcE>
	...

0000000000010316 <_start>:
   10316:	1141                	addi	sp,sp,-16
   10318:	e406                	sd	ra,8(sp)
   1031a:	00000097          	auipc	ra,0x0
   1031e:	d16080e7          	jalr	-746(ra) # 10030 <init_heap>
   10322:	00000097          	auipc	ra,0x0
   10326:	cde080e7          	jalr	-802(ra) # 10000 <main>
   1032a:	2501                	sext.w	a0,a0
   1032c:	05d00893          	li	a7,93
   10330:	4581                	li	a1,0
   10332:	4601                	li	a2,0
   10334:	00000073          	ecall
   10338:	a001                	j	10338 <_start+0x22>

000000000001033a <rust_begin_unwind>:
   1033a:	7135                	addi	sp,sp,-160
   1033c:	ed06                	sd	ra,152(sp)
   1033e:	e922                	sd	s0,144(sp)
   10340:	842a                	mv	s0,a0
   10342:	00000097          	auipc	ra,0x0
   10346:	1f8080e7          	jalr	504(ra) # 1053a <_ZN4core5panic10panic_info9PanicInfo7message17h5b5f79d040178b14E>
   1034a:	10050963          	beqz	a0,1045c <.LBB0_23>
   1034e:	e02a                	sd	a0,0(sp)

0000000000010350 <.LBB0_11>:
   10350:	00007517          	auipc	a0,0x7
   10354:	0d050513          	addi	a0,a0,208 # 17420 <_ZN7userlib5panic5ERROR17hde6e59aa01650af9E>
   10358:	4585                	li	a1,1
   1035a:	00b5352f          	amoadd.d	a0,a1,(a0)
   1035e:	c909                	beqz	a0,10370 <.LBB0_11+0x20>
   10360:	0d200893          	li	a7,210
   10364:	4501                	li	a0,0
   10366:	4581                	li	a1,0
   10368:	4601                	li	a2,0
   1036a:	00000073          	ecall
   1036e:	a001                	j	1036e <.LBB0_11+0x1e>
   10370:	8522                	mv	a0,s0
   10372:	00000097          	auipc	ra,0x0
   10376:	1cc080e7          	jalr	460(ra) # 1053e <_ZN4core5panic10panic_info9PanicInfo8location17ha93c43ca80ac2856E>
   1037a:	cd2d                	beqz	a0,103f4 <.LBB0_16+0x26>
   1037c:	610c                	ld	a1,0(a0)
   1037e:	6510                	ld	a2,8(a0)
   10380:	fc2e                	sd	a1,56(sp)
   10382:	e0b2                	sd	a2,64(sp)
   10384:	4908                	lw	a0,16(a0)
   10386:	c6aa                	sw	a0,76(sp)
   10388:	1828                	addi	a0,sp,56
   1038a:	e42a                	sd	a0,8(sp)

000000000001038c <.LBB0_12>:
   1038c:	00000517          	auipc	a0,0x0
   10390:	0ee50513          	addi	a0,a0,238 # 1047a <_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17hc153841de3b4330bE>
   10394:	e82a                	sd	a0,16(sp)
   10396:	00e8                	addi	a0,sp,76
   10398:	ec2a                	sd	a0,24(sp)

000000000001039a <.LBB0_13>:
   1039a:	00001517          	auipc	a0,0x1
   1039e:	c3050513          	addi	a0,a0,-976 # 10fca <_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17hef9fe5e8b139d194E>
   103a2:	f02a                	sd	a0,32(sp)
   103a4:	850a                	mv	a0,sp
   103a6:	f42a                	sd	a0,40(sp)

00000000000103a8 <.LBB0_14>:
   103a8:	00000517          	auipc	a0,0x0
   103ac:	0e650513          	addi	a0,a0,230 # 1048e <_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17hee02ba83243e6d55E>
   103b0:	f82a                	sd	a0,48(sp)
   103b2:	0128                	addi	a0,sp,136
   103b4:	e8aa                	sd	a0,80(sp)
   103b6:	ec82                	sd	zero,88(sp)

00000000000103b8 <.LBB0_15>:
   103b8:	00002517          	auipc	a0,0x2
   103bc:	e3850513          	addi	a0,a0,-456 # 121f0 <.Lanon.86a3613c128665d32fc75176e6ae67c2.11>
   103c0:	f4aa                	sd	a0,104(sp)
   103c2:	4511                	li	a0,4
   103c4:	f8aa                	sd	a0,112(sp)
   103c6:	0028                	addi	a0,sp,8
   103c8:	fcaa                	sd	a0,120(sp)
   103ca:	450d                	li	a0,3
   103cc:	e12a                	sd	a0,128(sp)

00000000000103ce <.LBB0_16>:
   103ce:	00002597          	auipc	a1,0x2
   103d2:	d1258593          	addi	a1,a1,-750 # 120e0 <anon.f66e08f6b2d961e480fb3b12cb7622cb.0.llvm.4825624523723557213>
   103d6:	0888                	addi	a0,sp,80
   103d8:	08b0                	addi	a2,sp,88
   103da:	00000097          	auipc	ra,0x0
   103de:	280080e7          	jalr	640(ra) # 1065a <_ZN4core3fmt5write17he707b088ca7ea77bE>
   103e2:	e929                	bnez	a0,10434 <.LBB0_20>
   103e4:	0d200893          	li	a7,210
   103e8:	4501                	li	a0,0
   103ea:	4581                	li	a1,0
   103ec:	4601                	li	a2,0
   103ee:	00000073          	ecall
   103f2:	a001                	j	103f2 <.LBB0_16+0x24>
   103f4:	850a                	mv	a0,sp
   103f6:	e42a                	sd	a0,8(sp)

00000000000103f8 <.LBB0_17>:
   103f8:	00000517          	auipc	a0,0x0
   103fc:	09650513          	addi	a0,a0,150 # 1048e <_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17hee02ba83243e6d55E>
   10400:	e82a                	sd	a0,16(sp)
   10402:	0128                	addi	a0,sp,136
   10404:	fc2a                	sd	a0,56(sp)
   10406:	ec82                	sd	zero,88(sp)

0000000000010408 <.LBB0_18>:
   10408:	00002517          	auipc	a0,0x2
   1040c:	db850513          	addi	a0,a0,-584 # 121c0 <.Lanon.86a3613c128665d32fc75176e6ae67c2.7>
   10410:	f4aa                	sd	a0,104(sp)
   10412:	4509                	li	a0,2
   10414:	f8aa                	sd	a0,112(sp)
   10416:	0028                	addi	a0,sp,8
   10418:	fcaa                	sd	a0,120(sp)
   1041a:	4505                	li	a0,1
   1041c:	e12a                	sd	a0,128(sp)

000000000001041e <.LBB0_19>:
   1041e:	00002597          	auipc	a1,0x2
   10422:	cc258593          	addi	a1,a1,-830 # 120e0 <anon.f66e08f6b2d961e480fb3b12cb7622cb.0.llvm.4825624523723557213>
   10426:	1828                	addi	a0,sp,56
   10428:	08b0                	addi	a2,sp,88
   1042a:	00000097          	auipc	ra,0x0
   1042e:	230080e7          	jalr	560(ra) # 1065a <_ZN4core3fmt5write17he707b088ca7ea77bE>
   10432:	d94d                	beqz	a0,103e4 <.LBB0_16+0x16>

0000000000010434 <.LBB0_20>:
   10434:	00002517          	auipc	a0,0x2
   10438:	cdc50513          	addi	a0,a0,-804 # 12110 <anon.f66e08f6b2d961e480fb3b12cb7622cb.1.llvm.4825624523723557213>

000000000001043c <.LBB0_21>:
   1043c:	00002697          	auipc	a3,0x2
   10440:	d0468693          	addi	a3,a3,-764 # 12140 <anon.f66e08f6b2d961e480fb3b12cb7622cb.2.llvm.4825624523723557213>

0000000000010444 <.LBB0_22>:
   10444:	00002717          	auipc	a4,0x2
   10448:	d5470713          	addi	a4,a4,-684 # 12198 <anon.f66e08f6b2d961e480fb3b12cb7622cb.4.llvm.4825624523723557213>
   1044c:	02b00593          	li	a1,43
   10450:	0130                	addi	a2,sp,136
   10452:	00000097          	auipc	ra,0x0
   10456:	18a080e7          	jalr	394(ra) # 105dc <_ZN4core6result13unwrap_failed17h5e58f0c34337c8fcE>
	...

000000000001045c <.LBB0_23>:
   1045c:	00002517          	auipc	a0,0x2
   10460:	dd450513          	addi	a0,a0,-556 # 12230 <.Lanon.86a3613c128665d32fc75176e6ae67c2.12>

0000000000010464 <.LBB0_24>:
   10464:	00002617          	auipc	a2,0x2
   10468:	e3460613          	addi	a2,a2,-460 # 12298 <.Lanon.86a3613c128665d32fc75176e6ae67c2.14>
   1046c:	02b00593          	li	a1,43
   10470:	00000097          	auipc	ra,0x0
   10474:	100080e7          	jalr	256(ra) # 10570 <_ZN4core9panicking5panic17h597b4f53f5061709E>
	...

000000000001047a <_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17hc153841de3b4330bE>:
   1047a:	6110                	ld	a2,0(a0)
   1047c:	6514                	ld	a3,8(a0)
   1047e:	872e                	mv	a4,a1
   10480:	8532                	mv	a0,a2
   10482:	85b6                	mv	a1,a3
   10484:	863a                	mv	a2,a4
   10486:	00001317          	auipc	t1,0x1
   1048a:	82830067          	jr	-2008(t1) # 10cae <_ZN42_$LT$str$u20$as$u20$core..fmt..Display$GT$3fmt17h5eb5edd471b47f2bE>

000000000001048e <_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17hee02ba83243e6d55E>:
   1048e:	6108                	ld	a0,0(a0)
   10490:	00000317          	auipc	t1,0x0
   10494:	19830067          	jr	408(t1) # 10628 <_ZN59_$LT$core..fmt..Arguments$u20$as$u20$core..fmt..Display$GT$3fmt17hab38ea65330d2494E>

0000000000010498 <_ZN22buddy_system_allocator17prev_power_of_two17he8186359febea13fE>:
   10498:	c935                	beqz	a0,1050c <.LBB0_7+0x1a>
   1049a:	00155593          	srli	a1,a0,0x1
   1049e:	8d4d                	or	a0,a0,a1
   104a0:	00255593          	srli	a1,a0,0x2
   104a4:	8d4d                	or	a0,a0,a1
   104a6:	00455593          	srli	a1,a0,0x4
   104aa:	8d4d                	or	a0,a0,a1
   104ac:	00855593          	srli	a1,a0,0x8
   104b0:	8d4d                	or	a0,a0,a1
   104b2:	01055593          	srli	a1,a0,0x10
   104b6:	8d4d                	or	a0,a0,a1
   104b8:	02055593          	srli	a1,a0,0x20
   104bc:	8d4d                	or	a0,a0,a1
   104be:	fff54513          	not	a0,a0

00000000000104c2 <.LBB0_4>:
   104c2:	00003597          	auipc	a1,0x3
   104c6:	b5e58593          	addi	a1,a1,-1186 # 13020 <.LCPI0_0>
   104ca:	618c                	ld	a1,0(a1)

00000000000104cc <.LBB0_5>:
   104cc:	00003617          	auipc	a2,0x3
   104d0:	b5c60613          	addi	a2,a2,-1188 # 13028 <.LCPI0_1>
   104d4:	6210                	ld	a2,0(a2)
   104d6:	00155693          	srli	a3,a0,0x1
   104da:	8df5                	and	a1,a1,a3
   104dc:	8d0d                	sub	a0,a0,a1
   104de:	00c575b3          	and	a1,a0,a2
   104e2:	8109                	srli	a0,a0,0x2
   104e4:	8d71                	and	a0,a0,a2
   104e6:	952e                	add	a0,a0,a1

00000000000104e8 <.LBB0_6>:
   104e8:	00003597          	auipc	a1,0x3
   104ec:	b4858593          	addi	a1,a1,-1208 # 13030 <.LCPI0_2>
   104f0:	618c                	ld	a1,0(a1)

00000000000104f2 <.LBB0_7>:
   104f2:	00003617          	auipc	a2,0x3
   104f6:	b4660613          	addi	a2,a2,-1210 # 13038 <.LCPI0_3>
   104fa:	6210                	ld	a2,0(a2)
   104fc:	00455693          	srli	a3,a0,0x4
   10500:	9536                	add	a0,a0,a3
   10502:	8d6d                	and	a0,a0,a1
   10504:	02c50533          	mul	a0,a0,a2
   10508:	9161                	srli	a0,a0,0x38
   1050a:	a019                	j	10510 <.LBB0_7+0x1e>
   1050c:	04000513          	li	a0,64
   10510:	03f00593          	li	a1,63
   10514:	40a58533          	sub	a0,a1,a0
   10518:	4585                	li	a1,1
   1051a:	00a59533          	sll	a0,a1,a0
   1051e:	8082                	ret

0000000000010520 <_ZN22buddy_system_allocator11linked_list10LinkedList4push17h6a533afaa3e8dc6bE>:
   10520:	6110                	ld	a2,0(a0)
   10522:	e190                	sd	a2,0(a1)
   10524:	e10c                	sd	a1,0(a0)
   10526:	8082                	ret

0000000000010528 <_ZN4core3ops8function6FnOnce9call_once17h7d5538df98a02550E>:
   10528:	6108                	ld	a0,0(a0)
   1052a:	a001                	j	1052a <_ZN4core3ops8function6FnOnce9call_once17h7d5538df98a02550E+0x2>

000000000001052c <_ZN4core3ptr102drop_in_place$LT$$RF$core..iter..adapters..copied..Copied$LT$core..slice..iter..Iter$LT$u8$GT$$GT$$GT$17h7b29e87dce2f01cdE>:
   1052c:	8082                	ret

000000000001052e <_ZN36_$LT$T$u20$as$u20$core..any..Any$GT$7type_id17h8c18475f5d6753f4E>:
   1052e:	00003517          	auipc	a0,0x3
   10532:	bfa50513          	addi	a0,a0,-1030 # 13128 <.LCPI97_0>
   10536:	6108                	ld	a0,0(a0)
   10538:	8082                	ret

000000000001053a <_ZN4core5panic10panic_info9PanicInfo7message17h5b5f79d040178b14E>:
   1053a:	6908                	ld	a0,16(a0)
   1053c:	8082                	ret

000000000001053e <_ZN4core5panic10panic_info9PanicInfo8location17ha93c43ca80ac2856E>:
   1053e:	6d08                	ld	a0,24(a0)
   10540:	8082                	ret

0000000000010542 <_ZN4core9panicking9panic_fmt17h60c7ff3c5bb8029dE>:
   10542:	7179                	addi	sp,sp,-48
   10544:	f406                	sd	ra,40(sp)

0000000000010546 <.LBB169_1>:
   10546:	00002617          	auipc	a2,0x2
   1054a:	d6a60613          	addi	a2,a2,-662 # 122b0 <.Lanon.821f7da411c20e9d3ff3aab887ae7593.2>
   1054e:	e032                	sd	a2,0(sp)

0000000000010550 <.LBB169_2>:
   10550:	00002617          	auipc	a2,0x2
   10554:	db860613          	addi	a2,a2,-584 # 12308 <.Lanon.821f7da411c20e9d3ff3aab887ae7593.231>
   10558:	e432                	sd	a2,8(sp)
   1055a:	e82a                	sd	a0,16(sp)
   1055c:	ec2e                	sd	a1,24(sp)
   1055e:	4505                	li	a0,1
   10560:	02a10023          	sb	a0,32(sp)
   10564:	850a                	mv	a0,sp
   10566:	00000097          	auipc	ra,0x0
   1056a:	dd4080e7          	jalr	-556(ra) # 1033a <rust_begin_unwind>
	...

0000000000010570 <_ZN4core9panicking5panic17h597b4f53f5061709E>:
   10570:	715d                	addi	sp,sp,-80
   10572:	e486                	sd	ra,72(sp)
   10574:	fc2a                	sd	a0,56(sp)
   10576:	e0ae                	sd	a1,64(sp)
   10578:	1828                	addi	a0,sp,56
   1057a:	ec2a                	sd	a0,24(sp)
   1057c:	4505                	li	a0,1
   1057e:	f02a                	sd	a0,32(sp)
   10580:	e402                	sd	zero,8(sp)

0000000000010582 <.LBB171_1>:
   10582:	00002517          	auipc	a0,0x2
   10586:	d2e50513          	addi	a0,a0,-722 # 122b0 <.Lanon.821f7da411c20e9d3ff3aab887ae7593.2>
   1058a:	f42a                	sd	a0,40(sp)
   1058c:	f802                	sd	zero,48(sp)
   1058e:	0028                	addi	a0,sp,8
   10590:	85b2                	mv	a1,a2
   10592:	00000097          	auipc	ra,0x0
   10596:	fb0080e7          	jalr	-80(ra) # 10542 <_ZN4core9panicking9panic_fmt17h60c7ff3c5bb8029dE>
	...

000000000001059c <_ZN4core9panicking18panic_bounds_check17h7918fc3ccbae3e2fE>:
   1059c:	7159                	addi	sp,sp,-112
   1059e:	f486                	sd	ra,104(sp)
   105a0:	e42a                	sd	a0,8(sp)
   105a2:	e82e                	sd	a1,16(sp)
   105a4:	0808                	addi	a0,sp,16
   105a6:	e4aa                	sd	a0,72(sp)

00000000000105a8 <.LBB175_1>:
   105a8:	00001517          	auipc	a0,0x1
   105ac:	a3250513          	addi	a0,a0,-1486 # 10fda <_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u64$GT$3fmt17h6fab6fe087fa630eE>
   105b0:	e8aa                	sd	a0,80(sp)
   105b2:	002c                	addi	a1,sp,8
   105b4:	ecae                	sd	a1,88(sp)
   105b6:	f0aa                	sd	a0,96(sp)

00000000000105b8 <.LBB175_2>:
   105b8:	00002517          	auipc	a0,0x2
   105bc:	d3050513          	addi	a0,a0,-720 # 122e8 <.Lanon.821f7da411c20e9d3ff3aab887ae7593.206>
   105c0:	f42a                	sd	a0,40(sp)
   105c2:	4509                	li	a0,2
   105c4:	f82a                	sd	a0,48(sp)
   105c6:	ec02                	sd	zero,24(sp)
   105c8:	00ac                	addi	a1,sp,72
   105ca:	fc2e                	sd	a1,56(sp)
   105cc:	e0aa                	sd	a0,64(sp)
   105ce:	0828                	addi	a0,sp,24
   105d0:	85b2                	mv	a1,a2
   105d2:	00000097          	auipc	ra,0x0
   105d6:	f70080e7          	jalr	-144(ra) # 10542 <_ZN4core9panicking9panic_fmt17h60c7ff3c5bb8029dE>
	...

00000000000105dc <_ZN4core6result13unwrap_failed17h5e58f0c34337c8fcE>:
   105dc:	7119                	addi	sp,sp,-128
   105de:	fc86                	sd	ra,120(sp)
   105e0:	e42a                	sd	a0,8(sp)
   105e2:	e82e                	sd	a1,16(sp)
   105e4:	ec32                	sd	a2,24(sp)
   105e6:	f036                	sd	a3,32(sp)
   105e8:	0028                	addi	a0,sp,8
   105ea:	ecaa                	sd	a0,88(sp)

00000000000105ec <.LBB182_1>:
   105ec:	00001517          	auipc	a0,0x1
   105f0:	a1650513          	addi	a0,a0,-1514 # 11002 <_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17h4575078792fa2417E>
   105f4:	f0aa                	sd	a0,96(sp)
   105f6:	0828                	addi	a0,sp,24
   105f8:	f4aa                	sd	a0,104(sp)

00000000000105fa <.LBB182_2>:
   105fa:	00001517          	auipc	a0,0x1
   105fe:	a0050513          	addi	a0,a0,-1536 # 10ffa <_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h45ca0030bea6599bE>
   10602:	f8aa                	sd	a0,112(sp)

0000000000010604 <.LBB182_3>:
   10604:	00002517          	auipc	a0,0x2
   10608:	d2c50513          	addi	a0,a0,-724 # 12330 <.Lanon.821f7da411c20e9d3ff3aab887ae7593.250>
   1060c:	fc2a                	sd	a0,56(sp)
   1060e:	4509                	li	a0,2
   10610:	e0aa                	sd	a0,64(sp)
   10612:	f402                	sd	zero,40(sp)
   10614:	08ac                	addi	a1,sp,88
   10616:	e4ae                	sd	a1,72(sp)
   10618:	e8aa                	sd	a0,80(sp)
   1061a:	1028                	addi	a0,sp,40
   1061c:	85ba                	mv	a1,a4
   1061e:	00000097          	auipc	ra,0x0
   10622:	f24080e7          	jalr	-220(ra) # 10542 <_ZN4core9panicking9panic_fmt17h60c7ff3c5bb8029dE>
	...

0000000000010628 <_ZN59_$LT$core..fmt..Arguments$u20$as$u20$core..fmt..Display$GT$3fmt17hab38ea65330d2494E>:
   10628:	7139                	addi	sp,sp,-64
   1062a:	fc06                	sd	ra,56(sp)
   1062c:	7510                	ld	a2,40(a0)
   1062e:	7118                	ld	a4,32(a0)
   10630:	6d1c                	ld	a5,24(a0)
   10632:	f832                	sd	a2,48(sp)
   10634:	6194                	ld	a3,0(a1)
   10636:	f43a                	sd	a4,40(sp)
   10638:	f03e                	sd	a5,32(sp)
   1063a:	6910                	ld	a2,16(a0)
   1063c:	6518                	ld	a4,8(a0)
   1063e:	6108                	ld	a0,0(a0)
   10640:	658c                	ld	a1,8(a1)
   10642:	ec32                	sd	a2,24(sp)
   10644:	e83a                	sd	a4,16(sp)
   10646:	e42a                	sd	a0,8(sp)
   10648:	0030                	addi	a2,sp,8
   1064a:	8536                	mv	a0,a3
   1064c:	00000097          	auipc	ra,0x0
   10650:	00e080e7          	jalr	14(ra) # 1065a <_ZN4core3fmt5write17he707b088ca7ea77bE>
   10654:	70e2                	ld	ra,56(sp)
   10656:	6121                	addi	sp,sp,64
   10658:	8082                	ret

000000000001065a <_ZN4core3fmt5write17he707b088ca7ea77bE>:
   1065a:	7119                	addi	sp,sp,-128
   1065c:	fc86                	sd	ra,120(sp)
   1065e:	f8a2                	sd	s0,112(sp)
   10660:	f4a6                	sd	s1,104(sp)
   10662:	f0ca                	sd	s2,96(sp)
   10664:	ecce                	sd	s3,88(sp)
   10666:	e8d2                	sd	s4,80(sp)
   10668:	e4d6                	sd	s5,72(sp)
   1066a:	e0da                	sd	s6,64(sp)
   1066c:	89b2                	mv	s3,a2
   1066e:	4605                	li	a2,1
   10670:	1616                	slli	a2,a2,0x25
   10672:	f832                	sd	a2,48(sp)
   10674:	460d                	li	a2,3
   10676:	02c10c23          	sb	a2,56(sp)
   1067a:	0009b603          	ld	a2,0(s3)
   1067e:	e802                	sd	zero,16(sp)
   10680:	f002                	sd	zero,32(sp)
   10682:	e02a                	sd	a0,0(sp)
   10684:	e42e                	sd	a1,8(sp)
   10686:	c669                	beqz	a2,10750 <.LBB219_31+0x9e>
   10688:	0089b503          	ld	a0,8(s3)
   1068c:	10050e63          	beqz	a0,107a8 <.LBB219_31+0xf6>
   10690:	0109b583          	ld	a1,16(s3)
   10694:	fff50693          	addi	a3,a0,-1
   10698:	068e                	slli	a3,a3,0x3
   1069a:	828d                	srli	a3,a3,0x3
   1069c:	00168913          	addi	s2,a3,1
   106a0:	00858493          	addi	s1,a1,8
   106a4:	03800593          	li	a1,56
   106a8:	02b50a33          	mul	s4,a0,a1
   106ac:	01860413          	addi	s0,a2,24
   106b0:	4a85                	li	s5,1

00000000000106b2 <.LBB219_31>:
   106b2:	00000b17          	auipc	s6,0x0
   106b6:	e76b0b13          	addi	s6,s6,-394 # 10528 <_ZN4core3ops8function6FnOnce9call_once17h7d5538df98a02550E>
   106ba:	6090                	ld	a2,0(s1)
   106bc:	ca09                	beqz	a2,106ce <.LBB219_31+0x1c>
   106be:	66a2                	ld	a3,8(sp)
   106c0:	6502                	ld	a0,0(sp)
   106c2:	ff84b583          	ld	a1,-8(s1)
   106c6:	6e94                	ld	a3,24(a3)
   106c8:	9682                	jalr	a3
   106ca:	10051163          	bnez	a0,107cc <.LBB219_31+0x11a>
   106ce:	4448                	lw	a0,12(s0)
   106d0:	da2a                	sw	a0,52(sp)
   106d2:	01040503          	lb	a0,16(s0)
   106d6:	02a10c23          	sb	a0,56(sp)
   106da:	440c                	lw	a1,8(s0)
   106dc:	0209b503          	ld	a0,32(s3)
   106e0:	d82e                	sw	a1,48(sp)
   106e2:	ff843683          	ld	a3,-8(s0)
   106e6:	600c                	ld	a1,0(s0)
   106e8:	ce89                	beqz	a3,10702 <.LBB219_31+0x50>
   106ea:	4601                	li	a2,0
   106ec:	01569c63          	bne	a3,s5,10704 <.LBB219_31+0x52>
   106f0:	0592                	slli	a1,a1,0x4
   106f2:	95aa                	add	a1,a1,a0
   106f4:	6590                	ld	a2,8(a1)
   106f6:	01660463          	beq	a2,s6,106fe <.LBB219_31+0x4c>
   106fa:	4601                	li	a2,0
   106fc:	a021                	j	10704 <.LBB219_31+0x52>
   106fe:	618c                	ld	a1,0(a1)
   10700:	618c                	ld	a1,0(a1)
   10702:	4605                	li	a2,1
   10704:	e832                	sd	a2,16(sp)
   10706:	ec2e                	sd	a1,24(sp)
   10708:	fe843683          	ld	a3,-24(s0)
   1070c:	ff043583          	ld	a1,-16(s0)
   10710:	ce89                	beqz	a3,1072a <.LBB219_31+0x78>
   10712:	4601                	li	a2,0
   10714:	01569c63          	bne	a3,s5,1072c <.LBB219_31+0x7a>
   10718:	0592                	slli	a1,a1,0x4
   1071a:	95aa                	add	a1,a1,a0
   1071c:	6590                	ld	a2,8(a1)
   1071e:	01660463          	beq	a2,s6,10726 <.LBB219_31+0x74>
   10722:	4601                	li	a2,0
   10724:	a021                	j	1072c <.LBB219_31+0x7a>
   10726:	618c                	ld	a1,0(a1)
   10728:	618c                	ld	a1,0(a1)
   1072a:	4605                	li	a2,1
   1072c:	f032                	sd	a2,32(sp)
   1072e:	f42e                	sd	a1,40(sp)
   10730:	6c0c                	ld	a1,24(s0)
   10732:	0592                	slli	a1,a1,0x4
   10734:	952e                	add	a0,a0,a1
   10736:	6510                	ld	a2,8(a0)
   10738:	6108                	ld	a0,0(a0)
   1073a:	858a                	mv	a1,sp
   1073c:	9602                	jalr	a2
   1073e:	e559                	bnez	a0,107cc <.LBB219_31+0x11a>
   10740:	04c1                	addi	s1,s1,16
   10742:	fc8a0a13          	addi	s4,s4,-56
   10746:	03840413          	addi	s0,s0,56
   1074a:	f60a18e3          	bnez	s4,106ba <.LBB219_31+0x8>
   1074e:	a881                	j	1079e <.LBB219_31+0xec>
   10750:	0289b503          	ld	a0,40(s3)
   10754:	c931                	beqz	a0,107a8 <.LBB219_31+0xf6>
   10756:	0209b583          	ld	a1,32(s3)
   1075a:	0109b603          	ld	a2,16(s3)
   1075e:	fff50693          	addi	a3,a0,-1
   10762:	0692                	slli	a3,a3,0x4
   10764:	8291                	srli	a3,a3,0x4
   10766:	00168913          	addi	s2,a3,1
   1076a:	00860413          	addi	s0,a2,8
   1076e:	00451a13          	slli	s4,a0,0x4
   10772:	00858493          	addi	s1,a1,8
   10776:	6010                	ld	a2,0(s0)
   10778:	ca01                	beqz	a2,10788 <.LBB219_31+0xd6>
   1077a:	66a2                	ld	a3,8(sp)
   1077c:	6502                	ld	a0,0(sp)
   1077e:	ff843583          	ld	a1,-8(s0)
   10782:	6e94                	ld	a3,24(a3)
   10784:	9682                	jalr	a3
   10786:	e139                	bnez	a0,107cc <.LBB219_31+0x11a>
   10788:	6090                	ld	a2,0(s1)
   1078a:	ff84b503          	ld	a0,-8(s1)
   1078e:	858a                	mv	a1,sp
   10790:	9602                	jalr	a2
   10792:	ed0d                	bnez	a0,107cc <.LBB219_31+0x11a>
   10794:	0441                	addi	s0,s0,16
   10796:	1a41                	addi	s4,s4,-16
   10798:	04c1                	addi	s1,s1,16
   1079a:	fc0a1ee3          	bnez	s4,10776 <.LBB219_31+0xc4>
   1079e:	0189b503          	ld	a0,24(s3)
   107a2:	00a96863          	bltu	s2,a0,107b2 <.LBB219_31+0x100>
   107a6:	a02d                	j	107d0 <.LBB219_31+0x11e>
   107a8:	4901                	li	s2,0
   107aa:	0189b503          	ld	a0,24(s3)
   107ae:	02a97163          	bgeu	s2,a0,107d0 <.LBB219_31+0x11e>
   107b2:	0109b503          	ld	a0,16(s3)
   107b6:	00491593          	slli	a1,s2,0x4
   107ba:	00b50633          	add	a2,a0,a1
   107be:	66a2                	ld	a3,8(sp)
   107c0:	6502                	ld	a0,0(sp)
   107c2:	620c                	ld	a1,0(a2)
   107c4:	6610                	ld	a2,8(a2)
   107c6:	6e94                	ld	a3,24(a3)
   107c8:	9682                	jalr	a3
   107ca:	c119                	beqz	a0,107d0 <.LBB219_31+0x11e>
   107cc:	4505                	li	a0,1
   107ce:	a011                	j	107d2 <.LBB219_31+0x120>
   107d0:	4501                	li	a0,0
   107d2:	70e6                	ld	ra,120(sp)
   107d4:	7446                	ld	s0,112(sp)
   107d6:	74a6                	ld	s1,104(sp)
   107d8:	7906                	ld	s2,96(sp)
   107da:	69e6                	ld	s3,88(sp)
   107dc:	6a46                	ld	s4,80(sp)
   107de:	6aa6                	ld	s5,72(sp)
   107e0:	6b06                	ld	s6,64(sp)
   107e2:	6109                	addi	sp,sp,128
   107e4:	8082                	ret

00000000000107e6 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E>:
   107e6:	7159                	addi	sp,sp,-112
   107e8:	f486                	sd	ra,104(sp)
   107ea:	f0a2                	sd	s0,96(sp)
   107ec:	eca6                	sd	s1,88(sp)
   107ee:	e8ca                	sd	s2,80(sp)
   107f0:	e4ce                	sd	s3,72(sp)
   107f2:	e0d2                	sd	s4,64(sp)
   107f4:	fc56                	sd	s5,56(sp)
   107f6:	f85a                	sd	s6,48(sp)
   107f8:	f45e                	sd	s7,40(sp)
   107fa:	f062                	sd	s8,32(sp)
   107fc:	ec66                	sd	s9,24(sp)
   107fe:	e86a                	sd	s10,16(sp)
   10800:	e46e                	sd	s11,8(sp)
   10802:	89be                	mv	s3,a5
   10804:	893a                	mv	s2,a4
   10806:	8b36                	mv	s6,a3
   10808:	8a32                	mv	s4,a2
   1080a:	8c2a                	mv	s8,a0
   1080c:	c1b9                	beqz	a1,10852 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x6c>
   1080e:	030c6403          	lwu	s0,48(s8)
   10812:	00147513          	andi	a0,s0,1
   10816:	00110ab7          	lui	s5,0x110
   1081a:	c119                	beqz	a0,10820 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x3a>
   1081c:	02b00a93          	li	s5,43
   10820:	01350cb3          	add	s9,a0,s3
   10824:	00447513          	andi	a0,s0,4
   10828:	cd15                	beqz	a0,10864 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x7e>
   1082a:	02000513          	li	a0,32
   1082e:	04ab7063          	bgeu	s6,a0,1086e <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x88>
   10832:	4501                	li	a0,0
   10834:	040b0363          	beqz	s6,1087a <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x94>
   10838:	85da                	mv	a1,s6
   1083a:	8652                	mv	a2,s4
   1083c:	00060683          	lb	a3,0(a2)
   10840:	0605                	addi	a2,a2,1
   10842:	fc06a693          	slti	a3,a3,-64
   10846:	0016c693          	xori	a3,a3,1
   1084a:	15fd                	addi	a1,a1,-1
   1084c:	9536                	add	a0,a0,a3
   1084e:	f5fd                	bnez	a1,1083c <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x56>
   10850:	a02d                	j	1087a <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x94>
   10852:	030c2403          	lw	s0,48(s8)
   10856:	00198c93          	addi	s9,s3,1
   1085a:	02d00a93          	li	s5,45
   1085e:	00447513          	andi	a0,s0,4
   10862:	f561                	bnez	a0,1082a <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x44>
   10864:	4a01                	li	s4,0
   10866:	010c3503          	ld	a0,16(s8)
   1086a:	ed01                	bnez	a0,10882 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x9c>
   1086c:	a099                	j	108b2 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xcc>
   1086e:	8552                	mv	a0,s4
   10870:	85da                	mv	a1,s6
   10872:	00000097          	auipc	ra,0x0
   10876:	44c080e7          	jalr	1100(ra) # 10cbe <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E>
   1087a:	9caa                	add	s9,s9,a0
   1087c:	010c3503          	ld	a0,16(s8)
   10880:	c90d                	beqz	a0,108b2 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xcc>
   10882:	018c3d03          	ld	s10,24(s8)
   10886:	03acf663          	bgeu	s9,s10,108b2 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xcc>
   1088a:	00847513          	andi	a0,s0,8
   1088e:	e541                	bnez	a0,10916 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x130>
   10890:	038c4583          	lbu	a1,56(s8)
   10894:	460d                	li	a2,3
   10896:	4505                	li	a0,1
   10898:	00c58363          	beq	a1,a2,1089e <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xb8>
   1089c:	852e                	mv	a0,a1
   1089e:	00357593          	andi	a1,a0,3
   108a2:	419d0533          	sub	a0,s10,s9
   108a6:	c1e1                	beqz	a1,10966 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x180>
   108a8:	4605                	li	a2,1
   108aa:	0cc59163          	bne	a1,a2,1096c <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x186>
   108ae:	4d01                	li	s10,0
   108b0:	a0d9                	j	10976 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x190>
   108b2:	000c3403          	ld	s0,0(s8)
   108b6:	008c3483          	ld	s1,8(s8)
   108ba:	8522                	mv	a0,s0
   108bc:	85a6                	mv	a1,s1
   108be:	8656                	mv	a2,s5
   108c0:	86d2                	mv	a3,s4
   108c2:	875a                	mv	a4,s6
   108c4:	00000097          	auipc	ra,0x0
   108c8:	140080e7          	jalr	320(ra) # 10a04 <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h1d292530dab90c4eE>
   108cc:	4b85                	li	s7,1
   108ce:	c10d                	beqz	a0,108f0 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x10a>
   108d0:	855e                	mv	a0,s7
   108d2:	70a6                	ld	ra,104(sp)
   108d4:	7406                	ld	s0,96(sp)
   108d6:	64e6                	ld	s1,88(sp)
   108d8:	6946                	ld	s2,80(sp)
   108da:	69a6                	ld	s3,72(sp)
   108dc:	6a06                	ld	s4,64(sp)
   108de:	7ae2                	ld	s5,56(sp)
   108e0:	7b42                	ld	s6,48(sp)
   108e2:	7ba2                	ld	s7,40(sp)
   108e4:	7c02                	ld	s8,32(sp)
   108e6:	6ce2                	ld	s9,24(sp)
   108e8:	6d42                	ld	s10,16(sp)
   108ea:	6da2                	ld	s11,8(sp)
   108ec:	6165                	addi	sp,sp,112
   108ee:	8082                	ret
   108f0:	6c9c                	ld	a5,24(s1)
   108f2:	8522                	mv	a0,s0
   108f4:	85ca                	mv	a1,s2
   108f6:	864e                	mv	a2,s3
   108f8:	70a6                	ld	ra,104(sp)
   108fa:	7406                	ld	s0,96(sp)
   108fc:	64e6                	ld	s1,88(sp)
   108fe:	6946                	ld	s2,80(sp)
   10900:	69a6                	ld	s3,72(sp)
   10902:	6a06                	ld	s4,64(sp)
   10904:	7ae2                	ld	s5,56(sp)
   10906:	7b42                	ld	s6,48(sp)
   10908:	7ba2                	ld	s7,40(sp)
   1090a:	7c02                	ld	s8,32(sp)
   1090c:	6ce2                	ld	s9,24(sp)
   1090e:	6d42                	ld	s10,16(sp)
   10910:	6da2                	ld	s11,8(sp)
   10912:	6165                	addi	sp,sp,112
   10914:	8782                	jr	a5
   10916:	034c2403          	lw	s0,52(s8)
   1091a:	03000513          	li	a0,48
   1091e:	038c4583          	lbu	a1,56(s8)
   10922:	e02e                	sd	a1,0(sp)
   10924:	000c3d83          	ld	s11,0(s8)
   10928:	008c3483          	ld	s1,8(s8)
   1092c:	02ac2a23          	sw	a0,52(s8)
   10930:	4b85                	li	s7,1
   10932:	037c0c23          	sb	s7,56(s8)
   10936:	856e                	mv	a0,s11
   10938:	85a6                	mv	a1,s1
   1093a:	8656                	mv	a2,s5
   1093c:	86d2                	mv	a3,s4
   1093e:	875a                	mv	a4,s6
   10940:	00000097          	auipc	ra,0x0
   10944:	0c4080e7          	jalr	196(ra) # 10a04 <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h1d292530dab90c4eE>
   10948:	f541                	bnez	a0,108d0 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xea>
   1094a:	8a22                	mv	s4,s0
   1094c:	419d0533          	sub	a0,s10,s9
   10950:	00150413          	addi	s0,a0,1
   10954:	147d                	addi	s0,s0,-1
   10956:	c449                	beqz	s0,109e0 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x1fa>
   10958:	7090                	ld	a2,32(s1)
   1095a:	03000593          	li	a1,48
   1095e:	856e                	mv	a0,s11
   10960:	9602                	jalr	a2
   10962:	d96d                	beqz	a0,10954 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x16e>
   10964:	b7b5                	j	108d0 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xea>
   10966:	8d2a                	mv	s10,a0
   10968:	852e                	mv	a0,a1
   1096a:	a031                	j	10976 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x190>
   1096c:	00150593          	addi	a1,a0,1
   10970:	8105                	srli	a0,a0,0x1
   10972:	0015dd13          	srli	s10,a1,0x1
   10976:	000c3c83          	ld	s9,0(s8)
   1097a:	008c3d83          	ld	s11,8(s8)
   1097e:	034c2403          	lw	s0,52(s8)
   10982:	00150493          	addi	s1,a0,1
   10986:	14fd                	addi	s1,s1,-1
   10988:	c889                	beqz	s1,1099a <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x1b4>
   1098a:	020db603          	ld	a2,32(s11)
   1098e:	8566                	mv	a0,s9
   10990:	85a2                	mv	a1,s0
   10992:	9602                	jalr	a2
   10994:	d96d                	beqz	a0,10986 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x1a0>
   10996:	4b85                	li	s7,1
   10998:	bf25                	j	108d0 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xea>
   1099a:	00110537          	lui	a0,0x110
   1099e:	4b85                	li	s7,1
   109a0:	f2a408e3          	beq	s0,a0,108d0 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xea>
   109a4:	8566                	mv	a0,s9
   109a6:	85ee                	mv	a1,s11
   109a8:	8656                	mv	a2,s5
   109aa:	86d2                	mv	a3,s4
   109ac:	875a                	mv	a4,s6
   109ae:	00000097          	auipc	ra,0x0
   109b2:	056080e7          	jalr	86(ra) # 10a04 <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h1d292530dab90c4eE>
   109b6:	fd09                	bnez	a0,108d0 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xea>
   109b8:	018db683          	ld	a3,24(s11)
   109bc:	8566                	mv	a0,s9
   109be:	85ca                	mv	a1,s2
   109c0:	864e                	mv	a2,s3
   109c2:	9682                	jalr	a3
   109c4:	f511                	bnez	a0,108d0 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xea>
   109c6:	4481                	li	s1,0
   109c8:	029d0a63          	beq	s10,s1,109fc <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x216>
   109cc:	020db603          	ld	a2,32(s11)
   109d0:	0485                	addi	s1,s1,1
   109d2:	8566                	mv	a0,s9
   109d4:	85a2                	mv	a1,s0
   109d6:	9602                	jalr	a2
   109d8:	d965                	beqz	a0,109c8 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x1e2>
   109da:	fff48513          	addi	a0,s1,-1
   109de:	a005                	j	109fe <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x218>
   109e0:	6c94                	ld	a3,24(s1)
   109e2:	856e                	mv	a0,s11
   109e4:	85ca                	mv	a1,s2
   109e6:	864e                	mv	a2,s3
   109e8:	9682                	jalr	a3
   109ea:	ee0513e3          	bnez	a0,108d0 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xea>
   109ee:	4b81                	li	s7,0
   109f0:	034c2a23          	sw	s4,52(s8)
   109f4:	6502                	ld	a0,0(sp)
   109f6:	02ac0c23          	sb	a0,56(s8)
   109fa:	bdd9                	j	108d0 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xea>
   109fc:	856a                	mv	a0,s10
   109fe:	01a53bb3          	sltu	s7,a0,s10
   10a02:	b5f9                	j	108d0 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xea>

0000000000010a04 <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h1d292530dab90c4eE>:
   10a04:	7179                	addi	sp,sp,-48
   10a06:	f406                	sd	ra,40(sp)
   10a08:	f022                	sd	s0,32(sp)
   10a0a:	ec26                	sd	s1,24(sp)
   10a0c:	e84a                	sd	s2,16(sp)
   10a0e:	e44e                	sd	s3,8(sp)
   10a10:	0006079b          	sext.w	a5,a2
   10a14:	00110837          	lui	a6,0x110
   10a18:	893a                	mv	s2,a4
   10a1a:	84b6                	mv	s1,a3
   10a1c:	842e                	mv	s0,a1
   10a1e:	89aa                	mv	s3,a0
   10a20:	01078963          	beq	a5,a6,10a32 <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h1d292530dab90c4eE+0x2e>
   10a24:	7014                	ld	a3,32(s0)
   10a26:	854e                	mv	a0,s3
   10a28:	85b2                	mv	a1,a2
   10a2a:	9682                	jalr	a3
   10a2c:	85aa                	mv	a1,a0
   10a2e:	4505                	li	a0,1
   10a30:	ed91                	bnez	a1,10a4c <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h1d292530dab90c4eE+0x48>
   10a32:	cc81                	beqz	s1,10a4a <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h1d292530dab90c4eE+0x46>
   10a34:	6c1c                	ld	a5,24(s0)
   10a36:	854e                	mv	a0,s3
   10a38:	85a6                	mv	a1,s1
   10a3a:	864a                	mv	a2,s2
   10a3c:	70a2                	ld	ra,40(sp)
   10a3e:	7402                	ld	s0,32(sp)
   10a40:	64e2                	ld	s1,24(sp)
   10a42:	6942                	ld	s2,16(sp)
   10a44:	69a2                	ld	s3,8(sp)
   10a46:	6145                	addi	sp,sp,48
   10a48:	8782                	jr	a5
   10a4a:	4501                	li	a0,0
   10a4c:	70a2                	ld	ra,40(sp)
   10a4e:	7402                	ld	s0,32(sp)
   10a50:	64e2                	ld	s1,24(sp)
   10a52:	6942                	ld	s2,16(sp)
   10a54:	69a2                	ld	s3,8(sp)
   10a56:	6145                	addi	sp,sp,48
   10a58:	8082                	ret

0000000000010a5a <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E>:
   10a5a:	715d                	addi	sp,sp,-80
   10a5c:	e486                	sd	ra,72(sp)
   10a5e:	e0a2                	sd	s0,64(sp)
   10a60:	fc26                	sd	s1,56(sp)
   10a62:	f84a                	sd	s2,48(sp)
   10a64:	f44e                	sd	s3,40(sp)
   10a66:	f052                	sd	s4,32(sp)
   10a68:	ec56                	sd	s5,24(sp)
   10a6a:	e85a                	sd	s6,16(sp)
   10a6c:	e45e                	sd	s7,8(sp)
   10a6e:	8a2a                	mv	s4,a0
   10a70:	01053283          	ld	t0,16(a0) # 110010 <_ZN7userlib5panic5ERROR17hde6e59aa01650af9E+0xf8bf0>
   10a74:	7108                	ld	a0,32(a0)
   10a76:	fff28693          	addi	a3,t0,-1
   10a7a:	00d036b3          	snez	a3,a3
   10a7e:	fff50713          	addi	a4,a0,-1
   10a82:	00e03733          	snez	a4,a4
   10a86:	8ef9                	and	a3,a3,a4
   10a88:	89b2                	mv	s3,a2
   10a8a:	892e                	mv	s2,a1
   10a8c:	16069d63          	bnez	a3,10c06 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x1ac>
   10a90:	4585                	li	a1,1
   10a92:	10b51863          	bne	a0,a1,10ba2 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x148>
   10a96:	028a3503          	ld	a0,40(s4)
   10a9a:	4581                	li	a1,0
   10a9c:	013906b3          	add	a3,s2,s3
   10aa0:	00150713          	addi	a4,a0,1
   10aa4:	00110337          	lui	t1,0x110
   10aa8:	0df00893          	li	a7,223
   10aac:	0f000813          	li	a6,240
   10ab0:	864a                	mv	a2,s2
   10ab2:	a801                	j	10ac2 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x68>
   10ab4:	00160513          	addi	a0,a2,1
   10ab8:	8d91                	sub	a1,a1,a2
   10aba:	95aa                	add	a1,a1,a0
   10abc:	862a                	mv	a2,a0
   10abe:	0e640263          	beq	s0,t1,10ba2 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x148>
   10ac2:	177d                	addi	a4,a4,-1
   10ac4:	c725                	beqz	a4,10b2c <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0xd2>
   10ac6:	0cd60e63          	beq	a2,a3,10ba2 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x148>
   10aca:	00060503          	lb	a0,0(a2)
   10ace:	0ff57413          	andi	s0,a0,255
   10ad2:	fe0551e3          	bgez	a0,10ab4 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x5a>
   10ad6:	00164503          	lbu	a0,1(a2)
   10ada:	01f47793          	andi	a5,s0,31
   10ade:	03f57493          	andi	s1,a0,63
   10ae2:	0288f963          	bgeu	a7,s0,10b14 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0xba>
   10ae6:	00264503          	lbu	a0,2(a2)
   10aea:	049a                	slli	s1,s1,0x6
   10aec:	03f57513          	andi	a0,a0,63
   10af0:	8cc9                	or	s1,s1,a0
   10af2:	03046763          	bltu	s0,a6,10b20 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0xc6>
   10af6:	00364503          	lbu	a0,3(a2)
   10afa:	17f6                	slli	a5,a5,0x3d
   10afc:	93ad                	srli	a5,a5,0x2b
   10afe:	049a                	slli	s1,s1,0x6
   10b00:	03f57513          	andi	a0,a0,63
   10b04:	8d45                	or	a0,a0,s1
   10b06:	00f56433          	or	s0,a0,a5
   10b0a:	08640c63          	beq	s0,t1,10ba2 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x148>
   10b0e:	00460513          	addi	a0,a2,4
   10b12:	b75d                	j	10ab8 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x5e>
   10b14:	00260513          	addi	a0,a2,2
   10b18:	079a                	slli	a5,a5,0x6
   10b1a:	0097e433          	or	s0,a5,s1
   10b1e:	bf69                	j	10ab8 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x5e>
   10b20:	00360513          	addi	a0,a2,3
   10b24:	07b2                	slli	a5,a5,0xc
   10b26:	00f4e433          	or	s0,s1,a5
   10b2a:	b779                	j	10ab8 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x5e>
   10b2c:	06d60b63          	beq	a2,a3,10ba2 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x148>
   10b30:	00060503          	lb	a0,0(a2)
   10b34:	04055363          	bgez	a0,10b7a <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x120>
   10b38:	0ff57513          	andi	a0,a0,255
   10b3c:	0e000693          	li	a3,224
   10b40:	02d56d63          	bltu	a0,a3,10b7a <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x120>
   10b44:	0f000693          	li	a3,240
   10b48:	02d56963          	bltu	a0,a3,10b7a <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x120>
   10b4c:	00164683          	lbu	a3,1(a2)
   10b50:	00264703          	lbu	a4,2(a2)
   10b54:	03f6f693          	andi	a3,a3,63
   10b58:	03f77713          	andi	a4,a4,63
   10b5c:	00364603          	lbu	a2,3(a2)
   10b60:	1576                	slli	a0,a0,0x3d
   10b62:	912d                	srli	a0,a0,0x2b
   10b64:	06b2                	slli	a3,a3,0xc
   10b66:	071a                	slli	a4,a4,0x6
   10b68:	8ed9                	or	a3,a3,a4
   10b6a:	03f67613          	andi	a2,a2,63
   10b6e:	8e55                	or	a2,a2,a3
   10b70:	8d51                	or	a0,a0,a2
   10b72:	00110637          	lui	a2,0x110
   10b76:	02c50663          	beq	a0,a2,10ba2 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x148>
   10b7a:	c185                	beqz	a1,10b9a <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x140>
   10b7c:	0135fd63          	bgeu	a1,s3,10b96 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x13c>
   10b80:	00b90533          	add	a0,s2,a1
   10b84:	00050503          	lb	a0,0(a0)
   10b88:	fc000613          	li	a2,-64
   10b8c:	00c55763          	bge	a0,a2,10b9a <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x140>
   10b90:	4501                	li	a0,0
   10b92:	e511                	bnez	a0,10b9e <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x144>
   10b94:	a039                	j	10ba2 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x148>
   10b96:	ff359de3          	bne	a1,s3,10b90 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x136>
   10b9a:	854a                	mv	a0,s2
   10b9c:	c119                	beqz	a0,10ba2 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x148>
   10b9e:	89ae                	mv	s3,a1
   10ba0:	892a                	mv	s2,a0
   10ba2:	06028263          	beqz	t0,10c06 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x1ac>
   10ba6:	018a3403          	ld	s0,24(s4)
   10baa:	02000513          	li	a0,32
   10bae:	04a9f463          	bgeu	s3,a0,10bf6 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x19c>
   10bb2:	4501                	li	a0,0
   10bb4:	00098e63          	beqz	s3,10bd0 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x176>
   10bb8:	85ce                	mv	a1,s3
   10bba:	864a                	mv	a2,s2
   10bbc:	00060683          	lb	a3,0(a2) # 110000 <_ZN7userlib5panic5ERROR17hde6e59aa01650af9E+0xf8be0>
   10bc0:	0605                	addi	a2,a2,1
   10bc2:	fc06a693          	slti	a3,a3,-64
   10bc6:	0016c693          	xori	a3,a3,1
   10bca:	15fd                	addi	a1,a1,-1
   10bcc:	9536                	add	a0,a0,a3
   10bce:	f5fd                	bnez	a1,10bbc <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x162>
   10bd0:	02857b63          	bgeu	a0,s0,10c06 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x1ac>
   10bd4:	038a4583          	lbu	a1,56(s4)
   10bd8:	468d                	li	a3,3
   10bda:	4601                	li	a2,0
   10bdc:	00d58363          	beq	a1,a3,10be2 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x188>
   10be0:	862e                	mv	a2,a1
   10be2:	00367593          	andi	a1,a2,3
   10be6:	40a40533          	sub	a0,s0,a0
   10bea:	c1a1                	beqz	a1,10c2a <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x1d0>
   10bec:	4605                	li	a2,1
   10bee:	04c59163          	bne	a1,a2,10c30 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x1d6>
   10bf2:	4a81                	li	s5,0
   10bf4:	a099                	j	10c3a <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x1e0>
   10bf6:	854a                	mv	a0,s2
   10bf8:	85ce                	mv	a1,s3
   10bfa:	00000097          	auipc	ra,0x0
   10bfe:	0c4080e7          	jalr	196(ra) # 10cbe <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E>
   10c02:	fc8569e3          	bltu	a0,s0,10bd4 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x17a>
   10c06:	008a3583          	ld	a1,8(s4)
   10c0a:	000a3503          	ld	a0,0(s4)
   10c0e:	6d9c                	ld	a5,24(a1)
   10c10:	85ca                	mv	a1,s2
   10c12:	864e                	mv	a2,s3
   10c14:	60a6                	ld	ra,72(sp)
   10c16:	6406                	ld	s0,64(sp)
   10c18:	74e2                	ld	s1,56(sp)
   10c1a:	7942                	ld	s2,48(sp)
   10c1c:	79a2                	ld	s3,40(sp)
   10c1e:	7a02                	ld	s4,32(sp)
   10c20:	6ae2                	ld	s5,24(sp)
   10c22:	6b42                	ld	s6,16(sp)
   10c24:	6ba2                	ld	s7,8(sp)
   10c26:	6161                	addi	sp,sp,80
   10c28:	8782                	jr	a5
   10c2a:	8aaa                	mv	s5,a0
   10c2c:	852e                	mv	a0,a1
   10c2e:	a031                	j	10c3a <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x1e0>
   10c30:	00150593          	addi	a1,a0,1
   10c34:	8105                	srli	a0,a0,0x1
   10c36:	0015da93          	srli	s5,a1,0x1
   10c3a:	000a3b03          	ld	s6,0(s4)
   10c3e:	008a3b83          	ld	s7,8(s4)
   10c42:	034a2483          	lw	s1,52(s4)
   10c46:	00150413          	addi	s0,a0,1
   10c4a:	147d                	addi	s0,s0,-1
   10c4c:	c809                	beqz	s0,10c5e <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x204>
   10c4e:	020bb603          	ld	a2,32(s7)
   10c52:	855a                	mv	a0,s6
   10c54:	85a6                	mv	a1,s1
   10c56:	9602                	jalr	a2
   10c58:	d96d                	beqz	a0,10c4a <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x1f0>
   10c5a:	4a05                	li	s4,1
   10c5c:	a82d                	j	10c96 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x23c>
   10c5e:	00110537          	lui	a0,0x110
   10c62:	4a05                	li	s4,1
   10c64:	02a48963          	beq	s1,a0,10c96 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x23c>
   10c68:	018bb683          	ld	a3,24(s7)
   10c6c:	855a                	mv	a0,s6
   10c6e:	85ca                	mv	a1,s2
   10c70:	864e                	mv	a2,s3
   10c72:	9682                	jalr	a3
   10c74:	e10d                	bnez	a0,10c96 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x23c>
   10c76:	4401                	li	s0,0
   10c78:	008a8c63          	beq	s5,s0,10c90 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x236>
   10c7c:	020bb603          	ld	a2,32(s7)
   10c80:	0405                	addi	s0,s0,1
   10c82:	855a                	mv	a0,s6
   10c84:	85a6                	mv	a1,s1
   10c86:	9602                	jalr	a2
   10c88:	d965                	beqz	a0,10c78 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x21e>
   10c8a:	fff40513          	addi	a0,s0,-1
   10c8e:	a011                	j	10c92 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x238>
   10c90:	8556                	mv	a0,s5
   10c92:	01553a33          	sltu	s4,a0,s5
   10c96:	8552                	mv	a0,s4
   10c98:	60a6                	ld	ra,72(sp)
   10c9a:	6406                	ld	s0,64(sp)
   10c9c:	74e2                	ld	s1,56(sp)
   10c9e:	7942                	ld	s2,48(sp)
   10ca0:	79a2                	ld	s3,40(sp)
   10ca2:	7a02                	ld	s4,32(sp)
   10ca4:	6ae2                	ld	s5,24(sp)
   10ca6:	6b42                	ld	s6,16(sp)
   10ca8:	6ba2                	ld	s7,8(sp)
   10caa:	6161                	addi	sp,sp,80
   10cac:	8082                	ret

0000000000010cae <_ZN42_$LT$str$u20$as$u20$core..fmt..Display$GT$3fmt17h5eb5edd471b47f2bE>:
   10cae:	86ae                	mv	a3,a1
   10cb0:	85aa                	mv	a1,a0
   10cb2:	8532                	mv	a0,a2
   10cb4:	8636                	mv	a2,a3
   10cb6:	00000317          	auipc	t1,0x0
   10cba:	da430067          	jr	-604(t1) # 10a5a <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E>

0000000000010cbe <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E>:
   10cbe:	862a                	mv	a2,a0
   10cc0:	051d                	addi	a0,a0,7
   10cc2:	ff857713          	andi	a4,a0,-8
   10cc6:	40c708b3          	sub	a7,a4,a2
   10cca:	0115ec63          	bltu	a1,a7,10ce2 <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E+0x24>
   10cce:	41158833          	sub	a6,a1,a7
   10cd2:	00883513          	sltiu	a0,a6,8
   10cd6:	0098b793          	sltiu	a5,a7,9
   10cda:	0017c793          	xori	a5,a5,1
   10cde:	8d5d                	or	a0,a0,a5
   10ce0:	cd11                	beqz	a0,10cfc <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E+0x3e>
   10ce2:	4501                	li	a0,0
   10ce4:	c999                	beqz	a1,10cfa <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E+0x3c>
   10ce6:	00060683          	lb	a3,0(a2)
   10cea:	0605                	addi	a2,a2,1
   10cec:	fc06a693          	slti	a3,a3,-64
   10cf0:	0016c693          	xori	a3,a3,1
   10cf4:	15fd                	addi	a1,a1,-1
   10cf6:	9536                	add	a0,a0,a3
   10cf8:	f5fd                	bnez	a1,10ce6 <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E+0x28>
   10cfa:	8082                	ret
   10cfc:	00787593          	andi	a1,a6,7
   10d00:	4781                	li	a5,0
   10d02:	00c70f63          	beq	a4,a2,10d20 <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E+0x62>
   10d06:	40e60733          	sub	a4,a2,a4
   10d0a:	8532                	mv	a0,a2
   10d0c:	00050683          	lb	a3,0(a0) # 110000 <_ZN7userlib5panic5ERROR17hde6e59aa01650af9E+0xf8be0>
   10d10:	0505                	addi	a0,a0,1
   10d12:	fc06a693          	slti	a3,a3,-64
   10d16:	0016c693          	xori	a3,a3,1
   10d1a:	0705                	addi	a4,a4,1
   10d1c:	97b6                	add	a5,a5,a3
   10d1e:	f77d                	bnez	a4,10d0c <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E+0x4e>
   10d20:	011602b3          	add	t0,a2,a7
   10d24:	4601                	li	a2,0
   10d26:	cd99                	beqz	a1,10d44 <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E+0x86>
   10d28:	ff887513          	andi	a0,a6,-8
   10d2c:	00a286b3          	add	a3,t0,a0
   10d30:	00068503          	lb	a0,0(a3)
   10d34:	0685                	addi	a3,a3,1
   10d36:	fc052513          	slti	a0,a0,-64
   10d3a:	00154513          	xori	a0,a0,1
   10d3e:	15fd                	addi	a1,a1,-1
   10d40:	962a                	add	a2,a2,a0
   10d42:	f5fd                	bnez	a1,10d30 <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E+0x72>
   10d44:	00385713          	srli	a4,a6,0x3

0000000000010d48 <.LBB291_27>:
   10d48:	00002517          	auipc	a0,0x2
   10d4c:	4b850513          	addi	a0,a0,1208 # 13200 <.LCPI291_0>
   10d50:	00053f03          	ld	t5,0(a0)

0000000000010d54 <.LBB291_28>:
   10d54:	00002517          	auipc	a0,0x2
   10d58:	4b450513          	addi	a0,a0,1204 # 13208 <.LCPI291_1>
   10d5c:	00053883          	ld	a7,0(a0)
   10d60:	10001537          	lui	a0,0x10001
   10d64:	0512                	slli	a0,a0,0x4
   10d66:	0505                	addi	a0,a0,1
   10d68:	0542                	slli	a0,a0,0x10
   10d6a:	00150813          	addi	a6,a0,1 # 10001001 <_ZN7userlib5panic5ERROR17hde6e59aa01650af9E+0xffe9be1>
   10d6e:	00f60533          	add	a0,a2,a5
   10d72:	a025                	j	10d9a <.LBB291_28+0x46>
   10d74:	003e1613          	slli	a2,t3,0x3
   10d78:	00c302b3          	add	t0,t1,a2
   10d7c:	41c38733          	sub	a4,t2,t3
   10d80:	003e7613          	andi	a2,t3,3
   10d84:	0115f6b3          	and	a3,a1,a7
   10d88:	81a1                	srli	a1,a1,0x8
   10d8a:	0115f5b3          	and	a1,a1,a7
   10d8e:	95b6                	add	a1,a1,a3
   10d90:	030585b3          	mul	a1,a1,a6
   10d94:	91c1                	srli	a1,a1,0x30
   10d96:	952e                	add	a0,a0,a1
   10d98:	e241                	bnez	a2,10e18 <.LBB291_28+0xc4>
   10d9a:	d325                	beqz	a4,10cfa <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E+0x3c>
   10d9c:	83ba                	mv	t2,a4
   10d9e:	8316                	mv	t1,t0
   10da0:	0c000593          	li	a1,192
   10da4:	8e3a                	mv	t3,a4
   10da6:	00b76463          	bltu	a4,a1,10dae <.LBB291_28+0x5a>
   10daa:	0c000e13          	li	t3,192
   10dae:	0fce7593          	andi	a1,t3,252
   10db2:	00359613          	slli	a2,a1,0x3
   10db6:	00c30eb3          	add	t4,t1,a2
   10dba:	ddcd                	beqz	a1,10d74 <.LBB291_28+0x20>
   10dbc:	4581                	li	a1,0
   10dbe:	861a                	mv	a2,t1
   10dc0:	da55                	beqz	a2,10d74 <.LBB291_28+0x20>
   10dc2:	6218                	ld	a4,0(a2)
   10dc4:	fff74793          	not	a5,a4
   10dc8:	839d                	srli	a5,a5,0x7
   10dca:	8319                	srli	a4,a4,0x6
   10dcc:	6614                	ld	a3,8(a2)
   10dce:	8f5d                	or	a4,a4,a5
   10dd0:	01e77733          	and	a4,a4,t5
   10dd4:	95ba                	add	a1,a1,a4
   10dd6:	fff6c713          	not	a4,a3
   10dda:	831d                	srli	a4,a4,0x7
   10ddc:	8299                	srli	a3,a3,0x6
   10dde:	6a1c                	ld	a5,16(a2)
   10de0:	8ed9                	or	a3,a3,a4
   10de2:	01e6f6b3          	and	a3,a3,t5
   10de6:	95b6                	add	a1,a1,a3
   10de8:	fff7c693          	not	a3,a5
   10dec:	829d                	srli	a3,a3,0x7
   10dee:	0067d713          	srli	a4,a5,0x6
   10df2:	6e1c                	ld	a5,24(a2)
   10df4:	8ed9                	or	a3,a3,a4
   10df6:	01e6f6b3          	and	a3,a3,t5
   10dfa:	95b6                	add	a1,a1,a3
   10dfc:	fff7c693          	not	a3,a5
   10e00:	829d                	srli	a3,a3,0x7
   10e02:	0067d713          	srli	a4,a5,0x6
   10e06:	8ed9                	or	a3,a3,a4
   10e08:	01e6f6b3          	and	a3,a3,t5
   10e0c:	02060613          	addi	a2,a2,32
   10e10:	95b6                	add	a1,a1,a3
   10e12:	fbd617e3          	bne	a2,t4,10dc0 <.LBB291_28+0x6c>
   10e16:	bfb9                	j	10d74 <.LBB291_28+0x20>
   10e18:	02030a63          	beqz	t1,10e4c <.LBB291_28+0xf8>
   10e1c:	0c000593          	li	a1,192
   10e20:	00b3e463          	bltu	t2,a1,10e28 <.LBB291_28+0xd4>
   10e24:	0c000393          	li	t2,192
   10e28:	4581                	li	a1,0
   10e2a:	0033f613          	andi	a2,t2,3
   10e2e:	060e                	slli	a2,a2,0x3
   10e30:	000eb683          	ld	a3,0(t4)
   10e34:	0ea1                	addi	t4,t4,8
   10e36:	fff6c713          	not	a4,a3
   10e3a:	831d                	srli	a4,a4,0x7
   10e3c:	8299                	srli	a3,a3,0x6
   10e3e:	8ed9                	or	a3,a3,a4
   10e40:	01e6f6b3          	and	a3,a3,t5
   10e44:	1661                	addi	a2,a2,-8
   10e46:	95b6                	add	a1,a1,a3
   10e48:	f665                	bnez	a2,10e30 <.LBB291_28+0xdc>
   10e4a:	a011                	j	10e4e <.LBB291_28+0xfa>
   10e4c:	4581                	li	a1,0
   10e4e:	0115f633          	and	a2,a1,a7
   10e52:	81a1                	srli	a1,a1,0x8
   10e54:	0115f5b3          	and	a1,a1,a7
   10e58:	95b2                	add	a1,a1,a2
   10e5a:	030585b3          	mul	a1,a1,a6
   10e5e:	91c1                	srli	a1,a1,0x30
   10e60:	952e                	add	a0,a0,a1
   10e62:	8082                	ret

0000000000010e64 <_ZN4core3fmt3num3imp7fmt_u6417h673673462c6acfe6E>:
   10e64:	7139                	addi	sp,sp,-64
   10e66:	fc06                	sd	ra,56(sp)
   10e68:	f822                	sd	s0,48(sp)
   10e6a:	f426                	sd	s1,40(sp)
   10e6c:	8832                	mv	a6,a2
   10e6e:	00455693          	srli	a3,a0,0x4
   10e72:	02700713          	li	a4,39
   10e76:	27100793          	li	a5,625

0000000000010e7a <.LBB571_10>:
   10e7a:	00001e97          	auipc	t4,0x1
   10e7e:	4d6e8e93          	addi	t4,t4,1238 # 12350 <.Lanon.821f7da411c20e9d3ff3aab887ae7593.287>
   10e82:	02f6f363          	bgeu	a3,a5,10ea8 <.LBB571_10+0x2e>
   10e86:	06300693          	li	a3,99
   10e8a:	0aa6e963          	bltu	a3,a0,10f3c <.LBB571_11+0x92>
   10e8e:	4629                	li	a2,10
   10e90:	0ec57763          	bgeu	a0,a2,10f7e <.LBB571_11+0xd4>
   10e94:	fff70693          	addi	a3,a4,-1
   10e98:	00110613          	addi	a2,sp,1
   10e9c:	9636                	add	a2,a2,a3
   10e9e:	0305051b          	addiw	a0,a0,48
   10ea2:	00a60023          	sb	a0,0(a2)
   10ea6:	a8dd                	j	10f9c <.LBB571_11+0xf2>
   10ea8:	4701                	li	a4,0

0000000000010eaa <.LBB571_11>:
   10eaa:	00002697          	auipc	a3,0x2
   10eae:	3c668693          	addi	a3,a3,966 # 13270 <.LCPI571_0>
   10eb2:	0006b883          	ld	a7,0(a3)
   10eb6:	6689                	lui	a3,0x2
   10eb8:	7106839b          	addiw	t2,a3,1808
   10ebc:	6685                	lui	a3,0x1
   10ebe:	47b68e1b          	addiw	t3,a3,1147
   10ec2:	06400293          	li	t0,100
   10ec6:	00110313          	addi	t1,sp,1
   10eca:	05f5e6b7          	lui	a3,0x5f5e
   10ece:	0ff68f1b          	addiw	t5,a3,255
   10ed2:	87aa                	mv	a5,a0
   10ed4:	03153533          	mulhu	a0,a0,a7
   10ed8:	812d                	srli	a0,a0,0xb
   10eda:	0275063b          	mulw	a2,a0,t2
   10ede:	40c7863b          	subw	a2,a5,a2
   10ee2:	03061693          	slli	a3,a2,0x30
   10ee6:	92c9                	srli	a3,a3,0x32
   10ee8:	03c686b3          	mul	a3,a3,t3
   10eec:	0116df93          	srli	t6,a3,0x11
   10ef0:	82c1                	srli	a3,a3,0x10
   10ef2:	7fe6f413          	andi	s0,a3,2046
   10ef6:	025f86bb          	mulw	a3,t6,t0
   10efa:	9e15                	subw	a2,a2,a3
   10efc:	1646                	slli	a2,a2,0x31
   10efe:	9241                	srli	a2,a2,0x30
   10f00:	008e86b3          	add	a3,t4,s0
   10f04:	00e30433          	add	s0,t1,a4
   10f08:	0006cf83          	lbu	t6,0(a3) # 5f5e000 <_ZN7userlib5panic5ERROR17hde6e59aa01650af9E+0x5f46be0>
   10f0c:	00168683          	lb	a3,1(a3)
   10f10:	9676                	add	a2,a2,t4
   10f12:	00160483          	lb	s1,1(a2)
   10f16:	00064603          	lbu	a2,0(a2)
   10f1a:	02d40223          	sb	a3,36(s0)
   10f1e:	03f401a3          	sb	t6,35(s0)
   10f22:	02940323          	sb	s1,38(s0)
   10f26:	02c402a3          	sb	a2,37(s0)
   10f2a:	1771                	addi	a4,a4,-4
   10f2c:	faff63e3          	bltu	t5,a5,10ed2 <.LBB571_11+0x28>
   10f30:	02770713          	addi	a4,a4,39
   10f34:	06300693          	li	a3,99
   10f38:	f4a6fbe3          	bgeu	a3,a0,10e8e <.LBB571_10+0x14>
   10f3c:	03051613          	slli	a2,a0,0x30
   10f40:	9249                	srli	a2,a2,0x32
   10f42:	6685                	lui	a3,0x1
   10f44:	47b6869b          	addiw	a3,a3,1147
   10f48:	02d60633          	mul	a2,a2,a3
   10f4c:	8245                	srli	a2,a2,0x11
   10f4e:	06400693          	li	a3,100
   10f52:	02d606bb          	mulw	a3,a2,a3
   10f56:	9d15                	subw	a0,a0,a3
   10f58:	1546                	slli	a0,a0,0x31
   10f5a:	9141                	srli	a0,a0,0x30
   10f5c:	1779                	addi	a4,a4,-2
   10f5e:	9576                	add	a0,a0,t4
   10f60:	00150683          	lb	a3,1(a0)
   10f64:	00054503          	lbu	a0,0(a0)
   10f68:	00110793          	addi	a5,sp,1
   10f6c:	97ba                	add	a5,a5,a4
   10f6e:	00d780a3          	sb	a3,1(a5)
   10f72:	00a78023          	sb	a0,0(a5)
   10f76:	8532                	mv	a0,a2
   10f78:	4629                	li	a2,10
   10f7a:	f0c56de3          	bltu	a0,a2,10e94 <.LBB571_10+0x1a>
   10f7e:	0506                	slli	a0,a0,0x1
   10f80:	ffe70693          	addi	a3,a4,-2
   10f84:	9576                	add	a0,a0,t4
   10f86:	00150603          	lb	a2,1(a0)
   10f8a:	00054503          	lbu	a0,0(a0)
   10f8e:	00110713          	addi	a4,sp,1
   10f92:	9736                	add	a4,a4,a3
   10f94:	00c700a3          	sb	a2,1(a4)
   10f98:	00a70023          	sb	a0,0(a4)
   10f9c:	00110513          	addi	a0,sp,1
   10fa0:	00d50733          	add	a4,a0,a3
   10fa4:	02700513          	li	a0,39
   10fa8:	40d507b3          	sub	a5,a0,a3

0000000000010fac <.LBB571_12>:
   10fac:	00001617          	auipc	a2,0x1
   10fb0:	30460613          	addi	a2,a2,772 # 122b0 <.Lanon.821f7da411c20e9d3ff3aab887ae7593.2>
   10fb4:	8542                	mv	a0,a6
   10fb6:	4681                	li	a3,0
   10fb8:	00000097          	auipc	ra,0x0
   10fbc:	82e080e7          	jalr	-2002(ra) # 107e6 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E>
   10fc0:	70e2                	ld	ra,56(sp)
   10fc2:	7442                	ld	s0,48(sp)
   10fc4:	74a2                	ld	s1,40(sp)
   10fc6:	6121                	addi	sp,sp,64
   10fc8:	8082                	ret

0000000000010fca <_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17hef9fe5e8b139d194E>:
   10fca:	00056503          	lwu	a0,0(a0)
   10fce:	862e                	mv	a2,a1
   10fd0:	4585                	li	a1,1
   10fd2:	00000317          	auipc	t1,0x0
   10fd6:	e9230067          	jr	-366(t1) # 10e64 <_ZN4core3fmt3num3imp7fmt_u6417h673673462c6acfe6E>

0000000000010fda <_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u64$GT$3fmt17h6fab6fe087fa630eE>:
   10fda:	6108                	ld	a0,0(a0)
   10fdc:	862e                	mv	a2,a1
   10fde:	4585                	li	a1,1
   10fe0:	00000317          	auipc	t1,0x0
   10fe4:	e8430067          	jr	-380(t1) # 10e64 <_ZN4core3fmt3num3imp7fmt_u6417h673673462c6acfe6E>

0000000000010fe8 <_ZN53_$LT$core..fmt..Error$u20$as$u20$core..fmt..Debug$GT$3fmt17h242baf87e8ca9f0bE>:
   10fe8:	6590                	ld	a2,8(a1)
   10fea:	6188                	ld	a0,0(a1)
   10fec:	6e1c                	ld	a5,24(a2)

0000000000010fee <.LBB603_1>:
   10fee:	00001597          	auipc	a1,0x1
   10ff2:	42a58593          	addi	a1,a1,1066 # 12418 <.Lanon.821f7da411c20e9d3ff3aab887ae7593.627>
   10ff6:	4615                	li	a2,5
   10ff8:	8782                	jr	a5

0000000000010ffa <_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h45ca0030bea6599bE>:
   10ffa:	6510                	ld	a2,8(a0)
   10ffc:	6108                	ld	a0,0(a0)
   10ffe:	6e1c                	ld	a5,24(a2)
   11000:	8782                	jr	a5

0000000000011002 <_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17h4575078792fa2417E>:
   11002:	6114                	ld	a3,0(a0)
   11004:	6510                	ld	a2,8(a0)
   11006:	852e                	mv	a0,a1
   11008:	85b6                	mv	a1,a3
   1100a:	00000317          	auipc	t1,0x0
   1100e:	a5030067          	jr	-1456(t1) # 10a5a <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E>
