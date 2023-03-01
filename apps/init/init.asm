
target/riscv64gc-unknown-none-elf/release/init：     文件格式 elf64-littleriscv


Disassembly of section .text:

0000000000010000 <main>:
   10000:	7171                	addi	sp,sp,-176
   10002:	f506                	sd	ra,168(sp)
   10004:	f122                	sd	s0,160(sp)
   10006:	ed26                	sd	s1,152(sp)
   10008:	e94a                	sd	s2,144(sp)
   1000a:	e54e                	sd	s3,136(sp)
   1000c:	e152                	sd	s4,128(sp)
   1000e:	fcd6                	sd	s5,120(sp)
   10010:	f8da                	sd	s6,112(sp)
   10012:	f4de                	sd	s7,104(sp)
   10014:	f0e2                	sd	s8,96(sp)

0000000000010016 <.LBB0_6>:
   10016:	00002517          	auipc	a0,0x2
   1001a:	ffa50513          	addi	a0,a0,-6 # 12010 <.Lanon.fad58de7366495db4650cfefac2fcd61.6>
   1001e:	f02a                	sd	a0,32(sp)
   10020:	4505                	li	a0,1
   10022:	f42a                	sd	a0,40(sp)
   10024:	e802                	sd	zero,16(sp)

0000000000010026 <.LBB0_7>:
   10026:	00002517          	auipc	a0,0x2
   1002a:	fda50513          	addi	a0,a0,-38 # 12000 <.Lanon.fad58de7366495db4650cfefac2fcd61.5>
   1002e:	f82a                	sd	a0,48(sp)
   10030:	fc02                	sd	zero,56(sp)
   10032:	0808                	addi	a0,sp,16
   10034:	00000097          	auipc	ra,0x0
   10038:	330080e7          	jalr	816(ra) # 10364 <_ZN7userlib2io7__print17hcc553115631cb938E>
   1003c:	00000097          	auipc	ra,0x0
   10040:	3b0080e7          	jalr	944(ra) # 103ec <_ZN7userlib7process4fork17hecc9822feef31171E>
   10044:	c13d                	beqz	a0,100aa <.LBB0_11>
   10046:	00410913          	addi	s2,sp,4
   1004a:	5a7d                	li	s4,-1
   1004c:	00810993          	addi	s3,sp,8

0000000000010050 <.LBB0_8>:
   10050:	00001a97          	auipc	s5,0x1
   10054:	0b4a8a93          	addi	s5,s5,180 # 11104 <_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h790955b0a0300c8dE>

0000000000010058 <.LBB0_9>:
   10058:	00001497          	auipc	s1,0x1
   1005c:	07a48493          	addi	s1,s1,122 # 110d2 <_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i32$GT$3fmt17h9e7237c084e472baE>

0000000000010060 <.LBB0_10>:
   10060:	00002417          	auipc	s0,0x2
   10064:	ff840413          	addi	s0,s0,-8 # 12058 <.Lanon.fad58de7366495db4650cfefac2fcd61.10>
   10068:	4b0d                	li	s6,3
   1006a:	04010b93          	addi	s7,sp,64
   1006e:	4c09                	li	s8,2
   10070:	a029                	j	1007a <.LBB0_10+0x1a>
   10072:	00000097          	auipc	ra,0x0
   10076:	1dc080e7          	jalr	476(ra) # 1024e <_ZN7userlib6thread7m_yield17h9cd92c34b2b96911E>
   1007a:	c202                	sw	zero,4(sp)
   1007c:	0048                	addi	a0,sp,4
   1007e:	00000097          	auipc	ra,0x0
   10082:	38c080e7          	jalr	908(ra) # 1040a <_ZN7userlib7process4wait17hd5af483082eb9d7cE>
   10086:	e42a                	sd	a0,8(sp)
   10088:	ff4505e3          	beq	a0,s4,10072 <.LBB0_10+0x12>
   1008c:	e0ce                	sd	s3,64(sp)
   1008e:	e4d6                	sd	s5,72(sp)
   10090:	e8ca                	sd	s2,80(sp)
   10092:	eca6                	sd	s1,88(sp)
   10094:	f022                	sd	s0,32(sp)
   10096:	f45a                	sd	s6,40(sp)
   10098:	e802                	sd	zero,16(sp)
   1009a:	f85e                	sd	s7,48(sp)
   1009c:	fc62                	sd	s8,56(sp)
   1009e:	0808                	addi	a0,sp,16
   100a0:	00000097          	auipc	ra,0x0
   100a4:	2c4080e7          	jalr	708(ra) # 10364 <_ZN7userlib2io7__print17hcc553115631cb938E>
   100a8:	bfc9                	j	1007a <.LBB0_10+0x1a>

00000000000100aa <.LBB0_11>:
   100aa:	00002517          	auipc	a0,0x2
   100ae:	fde50513          	addi	a0,a0,-34 # 12088 <.Lanon.fad58de7366495db4650cfefac2fcd61.11>
   100b2:	4599                	li	a1,6
   100b4:	70aa                	ld	ra,168(sp)
   100b6:	740a                	ld	s0,160(sp)
   100b8:	64ea                	ld	s1,152(sp)
   100ba:	694a                	ld	s2,144(sp)
   100bc:	69aa                	ld	s3,136(sp)
   100be:	6a0a                	ld	s4,128(sp)
   100c0:	7ae6                	ld	s5,120(sp)
   100c2:	7b46                	ld	s6,112(sp)
   100c4:	7ba6                	ld	s7,104(sp)
   100c6:	7c06                	ld	s8,96(sp)
   100c8:	614d                	addi	sp,sp,176
   100ca:	00000317          	auipc	t1,0x0
   100ce:	33230067          	jr	818(t1) # 103fc <_ZN7userlib7process4exec17h208f3b39090cb3caE>

00000000000100d2 <init_heap>:
   100d2:	711d                	addi	sp,sp,-96
   100d4:	ec86                	sd	ra,88(sp)
   100d6:	e8a2                	sd	s0,80(sp)
   100d8:	e4a6                	sd	s1,72(sp)
   100da:	e0ca                	sd	s2,64(sp)
   100dc:	fc4e                	sd	s3,56(sp)
   100de:	f852                	sd	s4,48(sp)
   100e0:	f456                	sd	s5,40(sp)
   100e2:	f05a                	sd	s6,32(sp)
   100e4:	ec5e                	sd	s7,24(sp)
   100e6:	e862                	sd	s8,16(sp)
   100e8:	e466                	sd	s9,8(sp)
   100ea:	e06a                	sd	s10,0(sp)

00000000000100ec <.LBB0_14>:
   100ec:	00007997          	auipc	s3,0x7
   100f0:	20c98993          	addi	s3,s3,524 # 172f8 <_ZN7userlib5alloc4HEAP17h573c842b00e3a432E.llvm.6931228215858126918>
   100f4:	4505                	li	a0,1
   100f6:	00a9b92f          	amoadd.d	s2,a0,(s3)
   100fa:	0089b503          	ld	a0,8(s3)
   100fe:	0230000f          	fence	r,rw
   10102:	01250a63          	beq	a0,s2,10116 <.LBB0_15>
   10106:	0100000f          	fence	w,unknown
   1010a:	0089b503          	ld	a0,8(s3)
   1010e:	0230000f          	fence	r,rw
   10112:	ff251ae3          	bne	a0,s2,10106 <.LBB0_14+0x1a>

0000000000010116 <.LBB0_15>:
   10116:	00003517          	auipc	a0,0x3
   1011a:	1e250513          	addi	a0,a0,482 # 132f8 <_ZN7userlib5alloc10HEAP_SPACE17h114e27e1461dc493E.llvm.6931228215858126918>
   1011e:	6591                	lui	a1,0x4
   10120:	95aa                	add	a1,a1,a0
   10122:	99e1                	andi	a1,a1,-8
   10124:	00750613          	addi	a2,a0,7
   10128:	ff867413          	andi	s0,a2,-8
   1012c:	1085e363          	bltu	a1,s0,10232 <.LBB0_21>
   10130:	4d01                	li	s10,0
   10132:	00840613          	addi	a2,s0,8
   10136:	0ac5ea63          	bltu	a1,a2,101ea <.LBB0_19+0x8a>
   1013a:	6591                	lui	a1,0x4

000000000001013c <.LBB0_16>:
   1013c:	00003617          	auipc	a2,0x3
   10140:	ec460613          	addi	a2,a2,-316 # 13000 <.LCPI0_0>
   10144:	00063a03          	ld	s4,0(a2)

0000000000010148 <.LBB0_17>:
   10148:	00003617          	auipc	a2,0x3
   1014c:	ec060613          	addi	a2,a2,-320 # 13008 <.LCPI0_1>
   10150:	00063a83          	ld	s5,0(a2)

0000000000010154 <.LBB0_18>:
   10154:	00003617          	auipc	a2,0x3
   10158:	ebc60613          	addi	a2,a2,-324 # 13010 <.LCPI0_2>
   1015c:	00063b03          	ld	s6,0(a2)

0000000000010160 <.LBB0_19>:
   10160:	00003617          	auipc	a2,0x3
   10164:	eb860613          	addi	a2,a2,-328 # 13018 <.LCPI0_3>
   10168:	00063b83          	ld	s7,0(a2)
   1016c:	952e                	add	a0,a0,a1
   1016e:	ff857c13          	andi	s8,a0,-8
   10172:	4cfd                	li	s9,31
   10174:	40800533          	neg	a0,s0
   10178:	00a474b3          	and	s1,s0,a0
   1017c:	408c0533          	sub	a0,s8,s0
   10180:	00000097          	auipc	ra,0x0
   10184:	420080e7          	jalr	1056(ra) # 105a0 <_ZN22buddy_system_allocator17prev_power_of_two17he8186359febea13fE>
   10188:	00a4e363          	bltu	s1,a0,1018e <.LBB0_19+0x2e>
   1018c:	84aa                	mv	s1,a0
   1018e:	cc85                	beqz	s1,101c6 <.LBB0_19+0x66>
   10190:	fff48513          	addi	a0,s1,-1
   10194:	fff4c593          	not	a1,s1
   10198:	8d6d                	and	a0,a0,a1
   1019a:	00155593          	srli	a1,a0,0x1
   1019e:	0145f5b3          	and	a1,a1,s4
   101a2:	8d0d                	sub	a0,a0,a1
   101a4:	015575b3          	and	a1,a0,s5
   101a8:	8109                	srli	a0,a0,0x2
   101aa:	01557533          	and	a0,a0,s5
   101ae:	952e                	add	a0,a0,a1
   101b0:	00455593          	srli	a1,a0,0x4
   101b4:	952e                	add	a0,a0,a1
   101b6:	01657533          	and	a0,a0,s6
   101ba:	03750533          	mul	a0,a0,s7
   101be:	9161                	srli	a0,a0,0x38
   101c0:	00acf763          	bgeu	s9,a0,101ce <.LBB0_19+0x6e>
   101c4:	a8a1                	j	1021c <.LBB0_20>
   101c6:	04000513          	li	a0,64
   101ca:	04ace963          	bltu	s9,a0,1021c <.LBB0_20>
   101ce:	9d26                	add	s10,s10,s1
   101d0:	050e                	slli	a0,a0,0x3
   101d2:	954e                	add	a0,a0,s3
   101d4:	0541                	addi	a0,a0,16
   101d6:	85a2                	mv	a1,s0
   101d8:	00000097          	auipc	ra,0x0
   101dc:	450080e7          	jalr	1104(ra) # 10628 <_ZN22buddy_system_allocator11linked_list10LinkedList4push17h6a533afaa3e8dc6bE>
   101e0:	9426                	add	s0,s0,s1
   101e2:	00840513          	addi	a0,s0,8
   101e6:	f8ac77e3          	bgeu	s8,a0,10174 <.LBB0_19+0x14>
   101ea:	1209b503          	ld	a0,288(s3)
   101ee:	956a                	add	a0,a0,s10
   101f0:	12a9b023          	sd	a0,288(s3)
   101f4:	00190513          	addi	a0,s2,1
   101f8:	0310000f          	fence	rw,w
   101fc:	00a9b423          	sd	a0,8(s3)
   10200:	60e6                	ld	ra,88(sp)
   10202:	6446                	ld	s0,80(sp)
   10204:	64a6                	ld	s1,72(sp)
   10206:	6906                	ld	s2,64(sp)
   10208:	79e2                	ld	s3,56(sp)
   1020a:	7a42                	ld	s4,48(sp)
   1020c:	7aa2                	ld	s5,40(sp)
   1020e:	7b02                	ld	s6,32(sp)
   10210:	6be2                	ld	s7,24(sp)
   10212:	6c42                	ld	s8,16(sp)
   10214:	6ca2                	ld	s9,8(sp)
   10216:	6d02                	ld	s10,0(sp)
   10218:	6125                	addi	sp,sp,96
   1021a:	8082                	ret

000000000001021c <.LBB0_20>:
   1021c:	00002617          	auipc	a2,0x2
   10220:	f1c60613          	addi	a2,a2,-228 # 12138 <anon.1c93528cad3da575e6989ee989895b5f.3.llvm.13359080111618654021>
   10224:	02000593          	li	a1,32
   10228:	00000097          	auipc	ra,0x0
   1022c:	47c080e7          	jalr	1148(ra) # 106a4 <_ZN4core9panicking18panic_bounds_check17h7918fc3ccbae3e2fE>
	...

0000000000010232 <.LBB0_21>:
   10232:	00002517          	auipc	a0,0x2
   10236:	e5c50513          	addi	a0,a0,-420 # 1208e <anon.1c93528cad3da575e6989ee989895b5f.0.llvm.13359080111618654021>

000000000001023a <.LBB0_22>:
   1023a:	00002617          	auipc	a2,0x2
   1023e:	ee660613          	addi	a2,a2,-282 # 12120 <anon.1c93528cad3da575e6989ee989895b5f.2.llvm.13359080111618654021>
   10242:	45f9                	li	a1,30
   10244:	00000097          	auipc	ra,0x0
   10248:	434080e7          	jalr	1076(ra) # 10678 <_ZN4core9panicking5panic17h597b4f53f5061709E>
	...

000000000001024e <_ZN7userlib6thread7m_yield17h9cd92c34b2b96911E>:
   1024e:	07c00893          	li	a7,124
   10252:	4501                	li	a0,0
   10254:	4581                	li	a1,0
   10256:	4601                	li	a2,0
   10258:	00000073          	ecall
   1025c:	8082                	ret

000000000001025e <_ZN4core3ptr37drop_in_place$LT$core..fmt..Error$GT$17h5835390175285eccE.llvm.4825624523723557213>:
   1025e:	8082                	ret

0000000000010260 <_ZN50_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write$GT$10write_char17h85f3c2e72e5958b3E.llvm.4825624523723557213>:
   10260:	1141                	addi	sp,sp,-16
   10262:	0005851b          	sext.w	a0,a1
   10266:	08000613          	li	a2,128
   1026a:	c602                	sw	zero,12(sp)
   1026c:	00c57663          	bgeu	a0,a2,10278 <_ZN50_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write$GT$10write_char17h85f3c2e72e5958b3E.llvm.4825624523723557213+0x18>
   10270:	00b10623          	sb	a1,12(sp)
   10274:	4605                	li	a2,1
   10276:	a851                	j	1030a <_ZN50_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write$GT$10write_char17h85f3c2e72e5958b3E.llvm.4825624523723557213+0xaa>
   10278:	00b5d51b          	srliw	a0,a1,0xb
   1027c:	ed19                	bnez	a0,1029a <_ZN50_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write$GT$10write_char17h85f3c2e72e5958b3E.llvm.4825624523723557213+0x3a>
   1027e:	0065d513          	srli	a0,a1,0x6
   10282:	0c056513          	ori	a0,a0,192
   10286:	00a10623          	sb	a0,12(sp)
   1028a:	03f5f513          	andi	a0,a1,63
   1028e:	08056513          	ori	a0,a0,128
   10292:	00a106a3          	sb	a0,13(sp)
   10296:	4609                	li	a2,2
   10298:	a88d                	j	1030a <_ZN50_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write$GT$10write_char17h85f3c2e72e5958b3E.llvm.4825624523723557213+0xaa>
   1029a:	0105d51b          	srliw	a0,a1,0x10
   1029e:	e905                	bnez	a0,102ce <_ZN50_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write$GT$10write_char17h85f3c2e72e5958b3E.llvm.4825624523723557213+0x6e>
   102a0:	02059513          	slli	a0,a1,0x20
   102a4:	9101                	srli	a0,a0,0x20
   102a6:	00c5d61b          	srliw	a2,a1,0xc
   102aa:	0e066613          	ori	a2,a2,224
   102ae:	00c10623          	sb	a2,12(sp)
   102b2:	1552                	slli	a0,a0,0x34
   102b4:	9169                	srli	a0,a0,0x3a
   102b6:	08056513          	ori	a0,a0,128
   102ba:	00a106a3          	sb	a0,13(sp)
   102be:	03f5f513          	andi	a0,a1,63
   102c2:	08056513          	ori	a0,a0,128
   102c6:	00a10723          	sb	a0,14(sp)
   102ca:	460d                	li	a2,3
   102cc:	a83d                	j	1030a <_ZN50_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write$GT$10write_char17h85f3c2e72e5958b3E.llvm.4825624523723557213+0xaa>
   102ce:	02059513          	slli	a0,a1,0x20
   102d2:	9101                	srli	a0,a0,0x20
   102d4:	02b51613          	slli	a2,a0,0x2b
   102d8:	9275                	srli	a2,a2,0x3d
   102da:	0f066613          	ori	a2,a2,240
   102de:	00c10623          	sb	a2,12(sp)
   102e2:	02e51613          	slli	a2,a0,0x2e
   102e6:	9269                	srli	a2,a2,0x3a
   102e8:	08066613          	ori	a2,a2,128
   102ec:	00c106a3          	sb	a2,13(sp)
   102f0:	1552                	slli	a0,a0,0x34
   102f2:	9169                	srli	a0,a0,0x3a
   102f4:	08056513          	ori	a0,a0,128
   102f8:	00a10723          	sb	a0,14(sp)
   102fc:	03f5f513          	andi	a0,a1,63
   10300:	08056513          	ori	a0,a0,128
   10304:	00a107a3          	sb	a0,15(sp)
   10308:	4611                	li	a2,4
   1030a:	4505                	li	a0,1
   1030c:	006c                	addi	a1,sp,12
   1030e:	04000893          	li	a7,64
   10312:	00000073          	ecall
   10316:	4501                	li	a0,0
   10318:	0141                	addi	sp,sp,16
   1031a:	8082                	ret

000000000001031c <_ZN50_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write$GT$9write_fmt17h0c85e093eeb846e5E.llvm.4825624523723557213>:
   1031c:	7139                	addi	sp,sp,-64
   1031e:	fc06                	sd	ra,56(sp)
   10320:	6108                	ld	a0,0(a0)
   10322:	7590                	ld	a2,40(a1)
   10324:	7194                	ld	a3,32(a1)
   10326:	e02a                	sd	a0,0(sp)
   10328:	f832                	sd	a2,48(sp)
   1032a:	f436                	sd	a3,40(sp)
   1032c:	6d88                	ld	a0,24(a1)
   1032e:	6990                	ld	a2,16(a1)
   10330:	6594                	ld	a3,8(a1)
   10332:	618c                	ld	a1,0(a1)
   10334:	f02a                	sd	a0,32(sp)
   10336:	ec32                	sd	a2,24(sp)
   10338:	e836                	sd	a3,16(sp)
   1033a:	e42e                	sd	a1,8(sp)

000000000001033c <.LBB2_1>:
   1033c:	00002597          	auipc	a1,0x2
   10340:	e1458593          	addi	a1,a1,-492 # 12150 <anon.f66e08f6b2d961e480fb3b12cb7622cb.0.llvm.4825624523723557213>
   10344:	850a                	mv	a0,sp
   10346:	0030                	addi	a2,sp,8
   10348:	00000097          	auipc	ra,0x0
   1034c:	41a080e7          	jalr	1050(ra) # 10762 <_ZN4core3fmt5write17he707b088ca7ea77bE>
   10350:	70e2                	ld	ra,56(sp)
   10352:	6121                	addi	sp,sp,64
   10354:	8082                	ret

0000000000010356 <_ZN50_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write$GT$9write_str17h56308d6d2b6fb9c6E.llvm.4825624523723557213>:
   10356:	4505                	li	a0,1
   10358:	04000893          	li	a7,64
   1035c:	00000073          	ecall
   10360:	4501                	li	a0,0
   10362:	8082                	ret

0000000000010364 <_ZN7userlib2io7__print17hcc553115631cb938E>:
   10364:	715d                	addi	sp,sp,-80
   10366:	e486                	sd	ra,72(sp)
   10368:	750c                	ld	a1,40(a0)
   1036a:	7110                	ld	a2,32(a0)
   1036c:	0094                	addi	a3,sp,64
   1036e:	e436                	sd	a3,8(sp)
   10370:	fc2e                	sd	a1,56(sp)
   10372:	f832                	sd	a2,48(sp)
   10374:	6d0c                	ld	a1,24(a0)
   10376:	6910                	ld	a2,16(a0)
   10378:	6514                	ld	a3,8(a0)
   1037a:	6108                	ld	a0,0(a0)
   1037c:	f42e                	sd	a1,40(sp)
   1037e:	f032                	sd	a2,32(sp)
   10380:	ec36                	sd	a3,24(sp)
   10382:	e82a                	sd	a0,16(sp)

0000000000010384 <.LBB5_3>:
   10384:	00002597          	auipc	a1,0x2
   10388:	dcc58593          	addi	a1,a1,-564 # 12150 <anon.f66e08f6b2d961e480fb3b12cb7622cb.0.llvm.4825624523723557213>
   1038c:	0028                	addi	a0,sp,8
   1038e:	0810                	addi	a2,sp,16
   10390:	00000097          	auipc	ra,0x0
   10394:	3d2080e7          	jalr	978(ra) # 10762 <_ZN4core3fmt5write17he707b088ca7ea77bE>
   10398:	e501                	bnez	a0,103a0 <.LBB5_4>
   1039a:	60a6                	ld	ra,72(sp)
   1039c:	6161                	addi	sp,sp,80
   1039e:	8082                	ret

00000000000103a0 <.LBB5_4>:
   103a0:	00002517          	auipc	a0,0x2
   103a4:	de050513          	addi	a0,a0,-544 # 12180 <anon.f66e08f6b2d961e480fb3b12cb7622cb.1.llvm.4825624523723557213>

00000000000103a8 <.LBB5_5>:
   103a8:	00002697          	auipc	a3,0x2
   103ac:	e0868693          	addi	a3,a3,-504 # 121b0 <anon.f66e08f6b2d961e480fb3b12cb7622cb.2.llvm.4825624523723557213>

00000000000103b0 <.LBB5_6>:
   103b0:	00002717          	auipc	a4,0x2
   103b4:	e5870713          	addi	a4,a4,-424 # 12208 <anon.f66e08f6b2d961e480fb3b12cb7622cb.4.llvm.4825624523723557213>
   103b8:	02b00593          	li	a1,43
   103bc:	0090                	addi	a2,sp,64
   103be:	00000097          	auipc	ra,0x0
   103c2:	326080e7          	jalr	806(ra) # 106e4 <_ZN4core6result13unwrap_failed17h5e58f0c34337c8fcE>
	...

00000000000103c8 <_start>:
   103c8:	1141                	addi	sp,sp,-16
   103ca:	e406                	sd	ra,8(sp)
   103cc:	00000097          	auipc	ra,0x0
   103d0:	d06080e7          	jalr	-762(ra) # 100d2 <init_heap>
   103d4:	00000097          	auipc	ra,0x0
   103d8:	c2c080e7          	jalr	-980(ra) # 10000 <main>
   103dc:	2501                	sext.w	a0,a0
   103de:	05d00893          	li	a7,93
   103e2:	4581                	li	a1,0
   103e4:	4601                	li	a2,0
   103e6:	00000073          	ecall
   103ea:	a001                	j	103ea <_start+0x22>

00000000000103ec <_ZN7userlib7process4fork17hecc9822feef31171E>:
   103ec:	0dc00893          	li	a7,220
   103f0:	4501                	li	a0,0
   103f2:	4581                	li	a1,0
   103f4:	4601                	li	a2,0
   103f6:	00000073          	ecall
   103fa:	8082                	ret

00000000000103fc <_ZN7userlib7process4exec17h208f3b39090cb3caE>:
   103fc:	0dd00893          	li	a7,221
   10400:	4581                	li	a1,0
   10402:	4601                	li	a2,0
   10404:	00000073          	ecall
   10408:	8082                	ret

000000000001040a <_ZN7userlib7process4wait17hd5af483082eb9d7cE>:
   1040a:	86aa                	mv	a3,a0
   1040c:	557d                	li	a0,-1
   1040e:	10400893          	li	a7,260
   10412:	5779                	li	a4,-2
   10414:	85b6                	mv	a1,a3
   10416:	4601                	li	a2,0
   10418:	00000073          	ecall
   1041c:	02e51263          	bne	a0,a4,10440 <_ZN7userlib7process4wait17hd5af483082eb9d7cE+0x36>
   10420:	07c00893          	li	a7,124
   10424:	4501                	li	a0,0
   10426:	4581                	li	a1,0
   10428:	4601                	li	a2,0
   1042a:	00000073          	ecall
   1042e:	557d                	li	a0,-1
   10430:	10400893          	li	a7,260
   10434:	85b6                	mv	a1,a3
   10436:	4601                	li	a2,0
   10438:	00000073          	ecall
   1043c:	fee502e3          	beq	a0,a4,10420 <_ZN7userlib7process4wait17hd5af483082eb9d7cE+0x16>
   10440:	8082                	ret

0000000000010442 <rust_begin_unwind>:
   10442:	7135                	addi	sp,sp,-160
   10444:	ed06                	sd	ra,152(sp)
   10446:	e922                	sd	s0,144(sp)
   10448:	842a                	mv	s0,a0
   1044a:	00000097          	auipc	ra,0x0
   1044e:	1f8080e7          	jalr	504(ra) # 10642 <_ZN4core5panic10panic_info9PanicInfo7message17h5b5f79d040178b14E>
   10452:	10050963          	beqz	a0,10564 <.LBB0_23>
   10456:	e02a                	sd	a0,0(sp)

0000000000010458 <.LBB0_11>:
   10458:	00007517          	auipc	a0,0x7
   1045c:	fc850513          	addi	a0,a0,-56 # 17420 <_ZN7userlib5panic5ERROR17hde6e59aa01650af9E>
   10460:	4585                	li	a1,1
   10462:	00b5352f          	amoadd.d	a0,a1,(a0)
   10466:	c909                	beqz	a0,10478 <.LBB0_11+0x20>
   10468:	0d200893          	li	a7,210
   1046c:	4501                	li	a0,0
   1046e:	4581                	li	a1,0
   10470:	4601                	li	a2,0
   10472:	00000073          	ecall
   10476:	a001                	j	10476 <.LBB0_11+0x1e>
   10478:	8522                	mv	a0,s0
   1047a:	00000097          	auipc	ra,0x0
   1047e:	1cc080e7          	jalr	460(ra) # 10646 <_ZN4core5panic10panic_info9PanicInfo8location17ha93c43ca80ac2856E>
   10482:	cd2d                	beqz	a0,104fc <.LBB0_16+0x26>
   10484:	610c                	ld	a1,0(a0)
   10486:	6510                	ld	a2,8(a0)
   10488:	fc2e                	sd	a1,56(sp)
   1048a:	e0b2                	sd	a2,64(sp)
   1048c:	4908                	lw	a0,16(a0)
   1048e:	c6aa                	sw	a0,76(sp)
   10490:	1828                	addi	a0,sp,56
   10492:	e42a                	sd	a0,8(sp)

0000000000010494 <.LBB0_12>:
   10494:	00000517          	auipc	a0,0x0
   10498:	0ee50513          	addi	a0,a0,238 # 10582 <_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17hc153841de3b4330bE>
   1049c:	e82a                	sd	a0,16(sp)
   1049e:	00e8                	addi	a0,sp,76
   104a0:	ec2a                	sd	a0,24(sp)

00000000000104a2 <.LBB0_13>:
   104a2:	00001517          	auipc	a0,0x1
   104a6:	c5250513          	addi	a0,a0,-942 # 110f4 <_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17hef9fe5e8b139d194E>
   104aa:	f02a                	sd	a0,32(sp)
   104ac:	850a                	mv	a0,sp
   104ae:	f42a                	sd	a0,40(sp)

00000000000104b0 <.LBB0_14>:
   104b0:	00000517          	auipc	a0,0x0
   104b4:	0e650513          	addi	a0,a0,230 # 10596 <_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17hee02ba83243e6d55E>
   104b8:	f82a                	sd	a0,48(sp)
   104ba:	0128                	addi	a0,sp,136
   104bc:	e8aa                	sd	a0,80(sp)
   104be:	ec82                	sd	zero,88(sp)

00000000000104c0 <.LBB0_15>:
   104c0:	00002517          	auipc	a0,0x2
   104c4:	da050513          	addi	a0,a0,-608 # 12260 <.Lanon.86a3613c128665d32fc75176e6ae67c2.11>
   104c8:	f4aa                	sd	a0,104(sp)
   104ca:	4511                	li	a0,4
   104cc:	f8aa                	sd	a0,112(sp)
   104ce:	0028                	addi	a0,sp,8
   104d0:	fcaa                	sd	a0,120(sp)
   104d2:	450d                	li	a0,3
   104d4:	e12a                	sd	a0,128(sp)

00000000000104d6 <.LBB0_16>:
   104d6:	00002597          	auipc	a1,0x2
   104da:	c7a58593          	addi	a1,a1,-902 # 12150 <anon.f66e08f6b2d961e480fb3b12cb7622cb.0.llvm.4825624523723557213>
   104de:	0888                	addi	a0,sp,80
   104e0:	08b0                	addi	a2,sp,88
   104e2:	00000097          	auipc	ra,0x0
   104e6:	280080e7          	jalr	640(ra) # 10762 <_ZN4core3fmt5write17he707b088ca7ea77bE>
   104ea:	e929                	bnez	a0,1053c <.LBB0_20>
   104ec:	0d200893          	li	a7,210
   104f0:	4501                	li	a0,0
   104f2:	4581                	li	a1,0
   104f4:	4601                	li	a2,0
   104f6:	00000073          	ecall
   104fa:	a001                	j	104fa <.LBB0_16+0x24>
   104fc:	850a                	mv	a0,sp
   104fe:	e42a                	sd	a0,8(sp)

0000000000010500 <.LBB0_17>:
   10500:	00000517          	auipc	a0,0x0
   10504:	09650513          	addi	a0,a0,150 # 10596 <_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17hee02ba83243e6d55E>
   10508:	e82a                	sd	a0,16(sp)
   1050a:	0128                	addi	a0,sp,136
   1050c:	fc2a                	sd	a0,56(sp)
   1050e:	ec82                	sd	zero,88(sp)

0000000000010510 <.LBB0_18>:
   10510:	00002517          	auipc	a0,0x2
   10514:	d2050513          	addi	a0,a0,-736 # 12230 <.Lanon.86a3613c128665d32fc75176e6ae67c2.7>
   10518:	f4aa                	sd	a0,104(sp)
   1051a:	4509                	li	a0,2
   1051c:	f8aa                	sd	a0,112(sp)
   1051e:	0028                	addi	a0,sp,8
   10520:	fcaa                	sd	a0,120(sp)
   10522:	4505                	li	a0,1
   10524:	e12a                	sd	a0,128(sp)

0000000000010526 <.LBB0_19>:
   10526:	00002597          	auipc	a1,0x2
   1052a:	c2a58593          	addi	a1,a1,-982 # 12150 <anon.f66e08f6b2d961e480fb3b12cb7622cb.0.llvm.4825624523723557213>
   1052e:	1828                	addi	a0,sp,56
   10530:	08b0                	addi	a2,sp,88
   10532:	00000097          	auipc	ra,0x0
   10536:	230080e7          	jalr	560(ra) # 10762 <_ZN4core3fmt5write17he707b088ca7ea77bE>
   1053a:	d94d                	beqz	a0,104ec <.LBB0_16+0x16>

000000000001053c <.LBB0_20>:
   1053c:	00002517          	auipc	a0,0x2
   10540:	c4450513          	addi	a0,a0,-956 # 12180 <anon.f66e08f6b2d961e480fb3b12cb7622cb.1.llvm.4825624523723557213>

0000000000010544 <.LBB0_21>:
   10544:	00002697          	auipc	a3,0x2
   10548:	c6c68693          	addi	a3,a3,-916 # 121b0 <anon.f66e08f6b2d961e480fb3b12cb7622cb.2.llvm.4825624523723557213>

000000000001054c <.LBB0_22>:
   1054c:	00002717          	auipc	a4,0x2
   10550:	cbc70713          	addi	a4,a4,-836 # 12208 <anon.f66e08f6b2d961e480fb3b12cb7622cb.4.llvm.4825624523723557213>
   10554:	02b00593          	li	a1,43
   10558:	0130                	addi	a2,sp,136
   1055a:	00000097          	auipc	ra,0x0
   1055e:	18a080e7          	jalr	394(ra) # 106e4 <_ZN4core6result13unwrap_failed17h5e58f0c34337c8fcE>
	...

0000000000010564 <.LBB0_23>:
   10564:	00002517          	auipc	a0,0x2
   10568:	d3c50513          	addi	a0,a0,-708 # 122a0 <.Lanon.86a3613c128665d32fc75176e6ae67c2.12>

000000000001056c <.LBB0_24>:
   1056c:	00002617          	auipc	a2,0x2
   10570:	d9c60613          	addi	a2,a2,-612 # 12308 <.Lanon.86a3613c128665d32fc75176e6ae67c2.14>
   10574:	02b00593          	li	a1,43
   10578:	00000097          	auipc	ra,0x0
   1057c:	100080e7          	jalr	256(ra) # 10678 <_ZN4core9panicking5panic17h597b4f53f5061709E>
	...

0000000000010582 <_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17hc153841de3b4330bE>:
   10582:	6110                	ld	a2,0(a0)
   10584:	6514                	ld	a3,8(a0)
   10586:	872e                	mv	a4,a1
   10588:	8532                	mv	a0,a2
   1058a:	85b6                	mv	a1,a3
   1058c:	863a                	mv	a2,a4
   1058e:	00001317          	auipc	t1,0x1
   10592:	82830067          	jr	-2008(t1) # 10db6 <_ZN42_$LT$str$u20$as$u20$core..fmt..Display$GT$3fmt17h5eb5edd471b47f2bE>

0000000000010596 <_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17hee02ba83243e6d55E>:
   10596:	6108                	ld	a0,0(a0)
   10598:	00000317          	auipc	t1,0x0
   1059c:	19830067          	jr	408(t1) # 10730 <_ZN59_$LT$core..fmt..Arguments$u20$as$u20$core..fmt..Display$GT$3fmt17hab38ea65330d2494E>

00000000000105a0 <_ZN22buddy_system_allocator17prev_power_of_two17he8186359febea13fE>:
   105a0:	c935                	beqz	a0,10614 <.LBB0_7+0x1a>
   105a2:	00155593          	srli	a1,a0,0x1
   105a6:	8d4d                	or	a0,a0,a1
   105a8:	00255593          	srli	a1,a0,0x2
   105ac:	8d4d                	or	a0,a0,a1
   105ae:	00455593          	srli	a1,a0,0x4
   105b2:	8d4d                	or	a0,a0,a1
   105b4:	00855593          	srli	a1,a0,0x8
   105b8:	8d4d                	or	a0,a0,a1
   105ba:	01055593          	srli	a1,a0,0x10
   105be:	8d4d                	or	a0,a0,a1
   105c0:	02055593          	srli	a1,a0,0x20
   105c4:	8d4d                	or	a0,a0,a1
   105c6:	fff54513          	not	a0,a0

00000000000105ca <.LBB0_4>:
   105ca:	00003597          	auipc	a1,0x3
   105ce:	a5658593          	addi	a1,a1,-1450 # 13020 <.LCPI0_0>
   105d2:	618c                	ld	a1,0(a1)

00000000000105d4 <.LBB0_5>:
   105d4:	00003617          	auipc	a2,0x3
   105d8:	a5460613          	addi	a2,a2,-1452 # 13028 <.LCPI0_1>
   105dc:	6210                	ld	a2,0(a2)
   105de:	00155693          	srli	a3,a0,0x1
   105e2:	8df5                	and	a1,a1,a3
   105e4:	8d0d                	sub	a0,a0,a1
   105e6:	00c575b3          	and	a1,a0,a2
   105ea:	8109                	srli	a0,a0,0x2
   105ec:	8d71                	and	a0,a0,a2
   105ee:	952e                	add	a0,a0,a1

00000000000105f0 <.LBB0_6>:
   105f0:	00003597          	auipc	a1,0x3
   105f4:	a4058593          	addi	a1,a1,-1472 # 13030 <.LCPI0_2>
   105f8:	618c                	ld	a1,0(a1)

00000000000105fa <.LBB0_7>:
   105fa:	00003617          	auipc	a2,0x3
   105fe:	a3e60613          	addi	a2,a2,-1474 # 13038 <.LCPI0_3>
   10602:	6210                	ld	a2,0(a2)
   10604:	00455693          	srli	a3,a0,0x4
   10608:	9536                	add	a0,a0,a3
   1060a:	8d6d                	and	a0,a0,a1
   1060c:	02c50533          	mul	a0,a0,a2
   10610:	9161                	srli	a0,a0,0x38
   10612:	a019                	j	10618 <.LBB0_7+0x1e>
   10614:	04000513          	li	a0,64
   10618:	03f00593          	li	a1,63
   1061c:	40a58533          	sub	a0,a1,a0
   10620:	4585                	li	a1,1
   10622:	00a59533          	sll	a0,a1,a0
   10626:	8082                	ret

0000000000010628 <_ZN22buddy_system_allocator11linked_list10LinkedList4push17h6a533afaa3e8dc6bE>:
   10628:	6110                	ld	a2,0(a0)
   1062a:	e190                	sd	a2,0(a1)
   1062c:	e10c                	sd	a1,0(a0)
   1062e:	8082                	ret

0000000000010630 <_ZN4core3ops8function6FnOnce9call_once17h7d5538df98a02550E>:
   10630:	6108                	ld	a0,0(a0)
   10632:	a001                	j	10632 <_ZN4core3ops8function6FnOnce9call_once17h7d5538df98a02550E+0x2>

0000000000010634 <_ZN4core3ptr102drop_in_place$LT$$RF$core..iter..adapters..copied..Copied$LT$core..slice..iter..Iter$LT$u8$GT$$GT$$GT$17h7b29e87dce2f01cdE>:
   10634:	8082                	ret

0000000000010636 <_ZN36_$LT$T$u20$as$u20$core..any..Any$GT$7type_id17h8c18475f5d6753f4E>:
   10636:	00003517          	auipc	a0,0x3
   1063a:	af250513          	addi	a0,a0,-1294 # 13128 <.LCPI97_0>
   1063e:	6108                	ld	a0,0(a0)
   10640:	8082                	ret

0000000000010642 <_ZN4core5panic10panic_info9PanicInfo7message17h5b5f79d040178b14E>:
   10642:	6908                	ld	a0,16(a0)
   10644:	8082                	ret

0000000000010646 <_ZN4core5panic10panic_info9PanicInfo8location17ha93c43ca80ac2856E>:
   10646:	6d08                	ld	a0,24(a0)
   10648:	8082                	ret

000000000001064a <_ZN4core9panicking9panic_fmt17h60c7ff3c5bb8029dE>:
   1064a:	7179                	addi	sp,sp,-48
   1064c:	f406                	sd	ra,40(sp)

000000000001064e <.LBB169_1>:
   1064e:	00002617          	auipc	a2,0x2
   10652:	cd260613          	addi	a2,a2,-814 # 12320 <.Lanon.821f7da411c20e9d3ff3aab887ae7593.2>
   10656:	e032                	sd	a2,0(sp)

0000000000010658 <.LBB169_2>:
   10658:	00002617          	auipc	a2,0x2
   1065c:	d2060613          	addi	a2,a2,-736 # 12378 <.Lanon.821f7da411c20e9d3ff3aab887ae7593.231>
   10660:	e432                	sd	a2,8(sp)
   10662:	e82a                	sd	a0,16(sp)
   10664:	ec2e                	sd	a1,24(sp)
   10666:	4505                	li	a0,1
   10668:	02a10023          	sb	a0,32(sp)
   1066c:	850a                	mv	a0,sp
   1066e:	00000097          	auipc	ra,0x0
   10672:	dd4080e7          	jalr	-556(ra) # 10442 <rust_begin_unwind>
	...

0000000000010678 <_ZN4core9panicking5panic17h597b4f53f5061709E>:
   10678:	715d                	addi	sp,sp,-80
   1067a:	e486                	sd	ra,72(sp)
   1067c:	fc2a                	sd	a0,56(sp)
   1067e:	e0ae                	sd	a1,64(sp)
   10680:	1828                	addi	a0,sp,56
   10682:	ec2a                	sd	a0,24(sp)
   10684:	4505                	li	a0,1
   10686:	f02a                	sd	a0,32(sp)
   10688:	e402                	sd	zero,8(sp)

000000000001068a <.LBB171_1>:
   1068a:	00002517          	auipc	a0,0x2
   1068e:	c9650513          	addi	a0,a0,-874 # 12320 <.Lanon.821f7da411c20e9d3ff3aab887ae7593.2>
   10692:	f42a                	sd	a0,40(sp)
   10694:	f802                	sd	zero,48(sp)
   10696:	0028                	addi	a0,sp,8
   10698:	85b2                	mv	a1,a2
   1069a:	00000097          	auipc	ra,0x0
   1069e:	fb0080e7          	jalr	-80(ra) # 1064a <_ZN4core9panicking9panic_fmt17h60c7ff3c5bb8029dE>
	...

00000000000106a4 <_ZN4core9panicking18panic_bounds_check17h7918fc3ccbae3e2fE>:
   106a4:	7159                	addi	sp,sp,-112
   106a6:	f486                	sd	ra,104(sp)
   106a8:	e42a                	sd	a0,8(sp)
   106aa:	e82e                	sd	a1,16(sp)
   106ac:	0808                	addi	a0,sp,16
   106ae:	e4aa                	sd	a0,72(sp)

00000000000106b0 <.LBB175_1>:
   106b0:	00001517          	auipc	a0,0x1
   106b4:	a7650513          	addi	a0,a0,-1418 # 11126 <_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u64$GT$3fmt17h6fab6fe087fa630eE>
   106b8:	e8aa                	sd	a0,80(sp)
   106ba:	002c                	addi	a1,sp,8
   106bc:	ecae                	sd	a1,88(sp)
   106be:	f0aa                	sd	a0,96(sp)

00000000000106c0 <.LBB175_2>:
   106c0:	00002517          	auipc	a0,0x2
   106c4:	c9850513          	addi	a0,a0,-872 # 12358 <.Lanon.821f7da411c20e9d3ff3aab887ae7593.206>
   106c8:	f42a                	sd	a0,40(sp)
   106ca:	4509                	li	a0,2
   106cc:	f82a                	sd	a0,48(sp)
   106ce:	ec02                	sd	zero,24(sp)
   106d0:	00ac                	addi	a1,sp,72
   106d2:	fc2e                	sd	a1,56(sp)
   106d4:	e0aa                	sd	a0,64(sp)
   106d6:	0828                	addi	a0,sp,24
   106d8:	85b2                	mv	a1,a2
   106da:	00000097          	auipc	ra,0x0
   106de:	f70080e7          	jalr	-144(ra) # 1064a <_ZN4core9panicking9panic_fmt17h60c7ff3c5bb8029dE>
	...

00000000000106e4 <_ZN4core6result13unwrap_failed17h5e58f0c34337c8fcE>:
   106e4:	7119                	addi	sp,sp,-128
   106e6:	fc86                	sd	ra,120(sp)
   106e8:	e42a                	sd	a0,8(sp)
   106ea:	e82e                	sd	a1,16(sp)
   106ec:	ec32                	sd	a2,24(sp)
   106ee:	f036                	sd	a3,32(sp)
   106f0:	0028                	addi	a0,sp,8
   106f2:	ecaa                	sd	a0,88(sp)

00000000000106f4 <.LBB182_1>:
   106f4:	00001517          	auipc	a0,0x1
   106f8:	a5a50513          	addi	a0,a0,-1446 # 1114e <_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17h4575078792fa2417E>
   106fc:	f0aa                	sd	a0,96(sp)
   106fe:	0828                	addi	a0,sp,24
   10700:	f4aa                	sd	a0,104(sp)

0000000000010702 <.LBB182_2>:
   10702:	00001517          	auipc	a0,0x1
   10706:	a4450513          	addi	a0,a0,-1468 # 11146 <_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h45ca0030bea6599bE>
   1070a:	f8aa                	sd	a0,112(sp)

000000000001070c <.LBB182_3>:
   1070c:	00002517          	auipc	a0,0x2
   10710:	c9450513          	addi	a0,a0,-876 # 123a0 <.Lanon.821f7da411c20e9d3ff3aab887ae7593.250>
   10714:	fc2a                	sd	a0,56(sp)
   10716:	4509                	li	a0,2
   10718:	e0aa                	sd	a0,64(sp)
   1071a:	f402                	sd	zero,40(sp)
   1071c:	08ac                	addi	a1,sp,88
   1071e:	e4ae                	sd	a1,72(sp)
   10720:	e8aa                	sd	a0,80(sp)
   10722:	1028                	addi	a0,sp,40
   10724:	85ba                	mv	a1,a4
   10726:	00000097          	auipc	ra,0x0
   1072a:	f24080e7          	jalr	-220(ra) # 1064a <_ZN4core9panicking9panic_fmt17h60c7ff3c5bb8029dE>
	...

0000000000010730 <_ZN59_$LT$core..fmt..Arguments$u20$as$u20$core..fmt..Display$GT$3fmt17hab38ea65330d2494E>:
   10730:	7139                	addi	sp,sp,-64
   10732:	fc06                	sd	ra,56(sp)
   10734:	7510                	ld	a2,40(a0)
   10736:	7118                	ld	a4,32(a0)
   10738:	6d1c                	ld	a5,24(a0)
   1073a:	f832                	sd	a2,48(sp)
   1073c:	6194                	ld	a3,0(a1)
   1073e:	f43a                	sd	a4,40(sp)
   10740:	f03e                	sd	a5,32(sp)
   10742:	6910                	ld	a2,16(a0)
   10744:	6518                	ld	a4,8(a0)
   10746:	6108                	ld	a0,0(a0)
   10748:	658c                	ld	a1,8(a1)
   1074a:	ec32                	sd	a2,24(sp)
   1074c:	e83a                	sd	a4,16(sp)
   1074e:	e42a                	sd	a0,8(sp)
   10750:	0030                	addi	a2,sp,8
   10752:	8536                	mv	a0,a3
   10754:	00000097          	auipc	ra,0x0
   10758:	00e080e7          	jalr	14(ra) # 10762 <_ZN4core3fmt5write17he707b088ca7ea77bE>
   1075c:	70e2                	ld	ra,56(sp)
   1075e:	6121                	addi	sp,sp,64
   10760:	8082                	ret

0000000000010762 <_ZN4core3fmt5write17he707b088ca7ea77bE>:
   10762:	7119                	addi	sp,sp,-128
   10764:	fc86                	sd	ra,120(sp)
   10766:	f8a2                	sd	s0,112(sp)
   10768:	f4a6                	sd	s1,104(sp)
   1076a:	f0ca                	sd	s2,96(sp)
   1076c:	ecce                	sd	s3,88(sp)
   1076e:	e8d2                	sd	s4,80(sp)
   10770:	e4d6                	sd	s5,72(sp)
   10772:	e0da                	sd	s6,64(sp)
   10774:	89b2                	mv	s3,a2
   10776:	4605                	li	a2,1
   10778:	1616                	slli	a2,a2,0x25
   1077a:	f832                	sd	a2,48(sp)
   1077c:	460d                	li	a2,3
   1077e:	02c10c23          	sb	a2,56(sp)
   10782:	0009b603          	ld	a2,0(s3)
   10786:	e802                	sd	zero,16(sp)
   10788:	f002                	sd	zero,32(sp)
   1078a:	e02a                	sd	a0,0(sp)
   1078c:	e42e                	sd	a1,8(sp)
   1078e:	c669                	beqz	a2,10858 <.LBB219_31+0x9e>
   10790:	0089b503          	ld	a0,8(s3)
   10794:	10050e63          	beqz	a0,108b0 <.LBB219_31+0xf6>
   10798:	0109b583          	ld	a1,16(s3)
   1079c:	fff50693          	addi	a3,a0,-1
   107a0:	068e                	slli	a3,a3,0x3
   107a2:	828d                	srli	a3,a3,0x3
   107a4:	00168913          	addi	s2,a3,1
   107a8:	00858493          	addi	s1,a1,8
   107ac:	03800593          	li	a1,56
   107b0:	02b50a33          	mul	s4,a0,a1
   107b4:	01860413          	addi	s0,a2,24
   107b8:	4a85                	li	s5,1

00000000000107ba <.LBB219_31>:
   107ba:	00000b17          	auipc	s6,0x0
   107be:	e76b0b13          	addi	s6,s6,-394 # 10630 <_ZN4core3ops8function6FnOnce9call_once17h7d5538df98a02550E>
   107c2:	6090                	ld	a2,0(s1)
   107c4:	ca09                	beqz	a2,107d6 <.LBB219_31+0x1c>
   107c6:	66a2                	ld	a3,8(sp)
   107c8:	6502                	ld	a0,0(sp)
   107ca:	ff84b583          	ld	a1,-8(s1)
   107ce:	6e94                	ld	a3,24(a3)
   107d0:	9682                	jalr	a3
   107d2:	10051163          	bnez	a0,108d4 <.LBB219_31+0x11a>
   107d6:	4448                	lw	a0,12(s0)
   107d8:	da2a                	sw	a0,52(sp)
   107da:	01040503          	lb	a0,16(s0)
   107de:	02a10c23          	sb	a0,56(sp)
   107e2:	440c                	lw	a1,8(s0)
   107e4:	0209b503          	ld	a0,32(s3)
   107e8:	d82e                	sw	a1,48(sp)
   107ea:	ff843683          	ld	a3,-8(s0)
   107ee:	600c                	ld	a1,0(s0)
   107f0:	ce89                	beqz	a3,1080a <.LBB219_31+0x50>
   107f2:	4601                	li	a2,0
   107f4:	01569c63          	bne	a3,s5,1080c <.LBB219_31+0x52>
   107f8:	0592                	slli	a1,a1,0x4
   107fa:	95aa                	add	a1,a1,a0
   107fc:	6590                	ld	a2,8(a1)
   107fe:	01660463          	beq	a2,s6,10806 <.LBB219_31+0x4c>
   10802:	4601                	li	a2,0
   10804:	a021                	j	1080c <.LBB219_31+0x52>
   10806:	618c                	ld	a1,0(a1)
   10808:	618c                	ld	a1,0(a1)
   1080a:	4605                	li	a2,1
   1080c:	e832                	sd	a2,16(sp)
   1080e:	ec2e                	sd	a1,24(sp)
   10810:	fe843683          	ld	a3,-24(s0)
   10814:	ff043583          	ld	a1,-16(s0)
   10818:	ce89                	beqz	a3,10832 <.LBB219_31+0x78>
   1081a:	4601                	li	a2,0
   1081c:	01569c63          	bne	a3,s5,10834 <.LBB219_31+0x7a>
   10820:	0592                	slli	a1,a1,0x4
   10822:	95aa                	add	a1,a1,a0
   10824:	6590                	ld	a2,8(a1)
   10826:	01660463          	beq	a2,s6,1082e <.LBB219_31+0x74>
   1082a:	4601                	li	a2,0
   1082c:	a021                	j	10834 <.LBB219_31+0x7a>
   1082e:	618c                	ld	a1,0(a1)
   10830:	618c                	ld	a1,0(a1)
   10832:	4605                	li	a2,1
   10834:	f032                	sd	a2,32(sp)
   10836:	f42e                	sd	a1,40(sp)
   10838:	6c0c                	ld	a1,24(s0)
   1083a:	0592                	slli	a1,a1,0x4
   1083c:	952e                	add	a0,a0,a1
   1083e:	6510                	ld	a2,8(a0)
   10840:	6108                	ld	a0,0(a0)
   10842:	858a                	mv	a1,sp
   10844:	9602                	jalr	a2
   10846:	e559                	bnez	a0,108d4 <.LBB219_31+0x11a>
   10848:	04c1                	addi	s1,s1,16
   1084a:	fc8a0a13          	addi	s4,s4,-56
   1084e:	03840413          	addi	s0,s0,56
   10852:	f60a18e3          	bnez	s4,107c2 <.LBB219_31+0x8>
   10856:	a881                	j	108a6 <.LBB219_31+0xec>
   10858:	0289b503          	ld	a0,40(s3)
   1085c:	c931                	beqz	a0,108b0 <.LBB219_31+0xf6>
   1085e:	0209b583          	ld	a1,32(s3)
   10862:	0109b603          	ld	a2,16(s3)
   10866:	fff50693          	addi	a3,a0,-1
   1086a:	0692                	slli	a3,a3,0x4
   1086c:	8291                	srli	a3,a3,0x4
   1086e:	00168913          	addi	s2,a3,1
   10872:	00860413          	addi	s0,a2,8
   10876:	00451a13          	slli	s4,a0,0x4
   1087a:	00858493          	addi	s1,a1,8
   1087e:	6010                	ld	a2,0(s0)
   10880:	ca01                	beqz	a2,10890 <.LBB219_31+0xd6>
   10882:	66a2                	ld	a3,8(sp)
   10884:	6502                	ld	a0,0(sp)
   10886:	ff843583          	ld	a1,-8(s0)
   1088a:	6e94                	ld	a3,24(a3)
   1088c:	9682                	jalr	a3
   1088e:	e139                	bnez	a0,108d4 <.LBB219_31+0x11a>
   10890:	6090                	ld	a2,0(s1)
   10892:	ff84b503          	ld	a0,-8(s1)
   10896:	858a                	mv	a1,sp
   10898:	9602                	jalr	a2
   1089a:	ed0d                	bnez	a0,108d4 <.LBB219_31+0x11a>
   1089c:	0441                	addi	s0,s0,16
   1089e:	1a41                	addi	s4,s4,-16
   108a0:	04c1                	addi	s1,s1,16
   108a2:	fc0a1ee3          	bnez	s4,1087e <.LBB219_31+0xc4>
   108a6:	0189b503          	ld	a0,24(s3)
   108aa:	00a96863          	bltu	s2,a0,108ba <.LBB219_31+0x100>
   108ae:	a02d                	j	108d8 <.LBB219_31+0x11e>
   108b0:	4901                	li	s2,0
   108b2:	0189b503          	ld	a0,24(s3)
   108b6:	02a97163          	bgeu	s2,a0,108d8 <.LBB219_31+0x11e>
   108ba:	0109b503          	ld	a0,16(s3)
   108be:	00491593          	slli	a1,s2,0x4
   108c2:	00b50633          	add	a2,a0,a1
   108c6:	66a2                	ld	a3,8(sp)
   108c8:	6502                	ld	a0,0(sp)
   108ca:	620c                	ld	a1,0(a2)
   108cc:	6610                	ld	a2,8(a2)
   108ce:	6e94                	ld	a3,24(a3)
   108d0:	9682                	jalr	a3
   108d2:	c119                	beqz	a0,108d8 <.LBB219_31+0x11e>
   108d4:	4505                	li	a0,1
   108d6:	a011                	j	108da <.LBB219_31+0x120>
   108d8:	4501                	li	a0,0
   108da:	70e6                	ld	ra,120(sp)
   108dc:	7446                	ld	s0,112(sp)
   108de:	74a6                	ld	s1,104(sp)
   108e0:	7906                	ld	s2,96(sp)
   108e2:	69e6                	ld	s3,88(sp)
   108e4:	6a46                	ld	s4,80(sp)
   108e6:	6aa6                	ld	s5,72(sp)
   108e8:	6b06                	ld	s6,64(sp)
   108ea:	6109                	addi	sp,sp,128
   108ec:	8082                	ret

00000000000108ee <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E>:
   108ee:	7159                	addi	sp,sp,-112
   108f0:	f486                	sd	ra,104(sp)
   108f2:	f0a2                	sd	s0,96(sp)
   108f4:	eca6                	sd	s1,88(sp)
   108f6:	e8ca                	sd	s2,80(sp)
   108f8:	e4ce                	sd	s3,72(sp)
   108fa:	e0d2                	sd	s4,64(sp)
   108fc:	fc56                	sd	s5,56(sp)
   108fe:	f85a                	sd	s6,48(sp)
   10900:	f45e                	sd	s7,40(sp)
   10902:	f062                	sd	s8,32(sp)
   10904:	ec66                	sd	s9,24(sp)
   10906:	e86a                	sd	s10,16(sp)
   10908:	e46e                	sd	s11,8(sp)
   1090a:	89be                	mv	s3,a5
   1090c:	893a                	mv	s2,a4
   1090e:	8b36                	mv	s6,a3
   10910:	8a32                	mv	s4,a2
   10912:	8c2a                	mv	s8,a0
   10914:	c1b9                	beqz	a1,1095a <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x6c>
   10916:	030c6403          	lwu	s0,48(s8)
   1091a:	00147513          	andi	a0,s0,1
   1091e:	00110ab7          	lui	s5,0x110
   10922:	c119                	beqz	a0,10928 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x3a>
   10924:	02b00a93          	li	s5,43
   10928:	01350cb3          	add	s9,a0,s3
   1092c:	00447513          	andi	a0,s0,4
   10930:	cd15                	beqz	a0,1096c <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x7e>
   10932:	02000513          	li	a0,32
   10936:	04ab7063          	bgeu	s6,a0,10976 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x88>
   1093a:	4501                	li	a0,0
   1093c:	040b0363          	beqz	s6,10982 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x94>
   10940:	85da                	mv	a1,s6
   10942:	8652                	mv	a2,s4
   10944:	00060683          	lb	a3,0(a2)
   10948:	0605                	addi	a2,a2,1
   1094a:	fc06a693          	slti	a3,a3,-64
   1094e:	0016c693          	xori	a3,a3,1
   10952:	15fd                	addi	a1,a1,-1
   10954:	9536                	add	a0,a0,a3
   10956:	f5fd                	bnez	a1,10944 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x56>
   10958:	a02d                	j	10982 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x94>
   1095a:	030c2403          	lw	s0,48(s8)
   1095e:	00198c93          	addi	s9,s3,1
   10962:	02d00a93          	li	s5,45
   10966:	00447513          	andi	a0,s0,4
   1096a:	f561                	bnez	a0,10932 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x44>
   1096c:	4a01                	li	s4,0
   1096e:	010c3503          	ld	a0,16(s8)
   10972:	ed01                	bnez	a0,1098a <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x9c>
   10974:	a099                	j	109ba <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xcc>
   10976:	8552                	mv	a0,s4
   10978:	85da                	mv	a1,s6
   1097a:	00000097          	auipc	ra,0x0
   1097e:	44c080e7          	jalr	1100(ra) # 10dc6 <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E>
   10982:	9caa                	add	s9,s9,a0
   10984:	010c3503          	ld	a0,16(s8)
   10988:	c90d                	beqz	a0,109ba <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xcc>
   1098a:	018c3d03          	ld	s10,24(s8)
   1098e:	03acf663          	bgeu	s9,s10,109ba <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xcc>
   10992:	00847513          	andi	a0,s0,8
   10996:	e541                	bnez	a0,10a1e <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x130>
   10998:	038c4583          	lbu	a1,56(s8)
   1099c:	460d                	li	a2,3
   1099e:	4505                	li	a0,1
   109a0:	00c58363          	beq	a1,a2,109a6 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xb8>
   109a4:	852e                	mv	a0,a1
   109a6:	00357593          	andi	a1,a0,3
   109aa:	419d0533          	sub	a0,s10,s9
   109ae:	c1e1                	beqz	a1,10a6e <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x180>
   109b0:	4605                	li	a2,1
   109b2:	0cc59163          	bne	a1,a2,10a74 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x186>
   109b6:	4d01                	li	s10,0
   109b8:	a0d9                	j	10a7e <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x190>
   109ba:	000c3403          	ld	s0,0(s8)
   109be:	008c3483          	ld	s1,8(s8)
   109c2:	8522                	mv	a0,s0
   109c4:	85a6                	mv	a1,s1
   109c6:	8656                	mv	a2,s5
   109c8:	86d2                	mv	a3,s4
   109ca:	875a                	mv	a4,s6
   109cc:	00000097          	auipc	ra,0x0
   109d0:	140080e7          	jalr	320(ra) # 10b0c <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h1d292530dab90c4eE>
   109d4:	4b85                	li	s7,1
   109d6:	c10d                	beqz	a0,109f8 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x10a>
   109d8:	855e                	mv	a0,s7
   109da:	70a6                	ld	ra,104(sp)
   109dc:	7406                	ld	s0,96(sp)
   109de:	64e6                	ld	s1,88(sp)
   109e0:	6946                	ld	s2,80(sp)
   109e2:	69a6                	ld	s3,72(sp)
   109e4:	6a06                	ld	s4,64(sp)
   109e6:	7ae2                	ld	s5,56(sp)
   109e8:	7b42                	ld	s6,48(sp)
   109ea:	7ba2                	ld	s7,40(sp)
   109ec:	7c02                	ld	s8,32(sp)
   109ee:	6ce2                	ld	s9,24(sp)
   109f0:	6d42                	ld	s10,16(sp)
   109f2:	6da2                	ld	s11,8(sp)
   109f4:	6165                	addi	sp,sp,112
   109f6:	8082                	ret
   109f8:	6c9c                	ld	a5,24(s1)
   109fa:	8522                	mv	a0,s0
   109fc:	85ca                	mv	a1,s2
   109fe:	864e                	mv	a2,s3
   10a00:	70a6                	ld	ra,104(sp)
   10a02:	7406                	ld	s0,96(sp)
   10a04:	64e6                	ld	s1,88(sp)
   10a06:	6946                	ld	s2,80(sp)
   10a08:	69a6                	ld	s3,72(sp)
   10a0a:	6a06                	ld	s4,64(sp)
   10a0c:	7ae2                	ld	s5,56(sp)
   10a0e:	7b42                	ld	s6,48(sp)
   10a10:	7ba2                	ld	s7,40(sp)
   10a12:	7c02                	ld	s8,32(sp)
   10a14:	6ce2                	ld	s9,24(sp)
   10a16:	6d42                	ld	s10,16(sp)
   10a18:	6da2                	ld	s11,8(sp)
   10a1a:	6165                	addi	sp,sp,112
   10a1c:	8782                	jr	a5
   10a1e:	034c2403          	lw	s0,52(s8)
   10a22:	03000513          	li	a0,48
   10a26:	038c4583          	lbu	a1,56(s8)
   10a2a:	e02e                	sd	a1,0(sp)
   10a2c:	000c3d83          	ld	s11,0(s8)
   10a30:	008c3483          	ld	s1,8(s8)
   10a34:	02ac2a23          	sw	a0,52(s8)
   10a38:	4b85                	li	s7,1
   10a3a:	037c0c23          	sb	s7,56(s8)
   10a3e:	856e                	mv	a0,s11
   10a40:	85a6                	mv	a1,s1
   10a42:	8656                	mv	a2,s5
   10a44:	86d2                	mv	a3,s4
   10a46:	875a                	mv	a4,s6
   10a48:	00000097          	auipc	ra,0x0
   10a4c:	0c4080e7          	jalr	196(ra) # 10b0c <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h1d292530dab90c4eE>
   10a50:	f541                	bnez	a0,109d8 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xea>
   10a52:	8a22                	mv	s4,s0
   10a54:	419d0533          	sub	a0,s10,s9
   10a58:	00150413          	addi	s0,a0,1
   10a5c:	147d                	addi	s0,s0,-1
   10a5e:	c449                	beqz	s0,10ae8 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x1fa>
   10a60:	7090                	ld	a2,32(s1)
   10a62:	03000593          	li	a1,48
   10a66:	856e                	mv	a0,s11
   10a68:	9602                	jalr	a2
   10a6a:	d96d                	beqz	a0,10a5c <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x16e>
   10a6c:	b7b5                	j	109d8 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xea>
   10a6e:	8d2a                	mv	s10,a0
   10a70:	852e                	mv	a0,a1
   10a72:	a031                	j	10a7e <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x190>
   10a74:	00150593          	addi	a1,a0,1
   10a78:	8105                	srli	a0,a0,0x1
   10a7a:	0015dd13          	srli	s10,a1,0x1
   10a7e:	000c3c83          	ld	s9,0(s8)
   10a82:	008c3d83          	ld	s11,8(s8)
   10a86:	034c2403          	lw	s0,52(s8)
   10a8a:	00150493          	addi	s1,a0,1
   10a8e:	14fd                	addi	s1,s1,-1
   10a90:	c889                	beqz	s1,10aa2 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x1b4>
   10a92:	020db603          	ld	a2,32(s11)
   10a96:	8566                	mv	a0,s9
   10a98:	85a2                	mv	a1,s0
   10a9a:	9602                	jalr	a2
   10a9c:	d96d                	beqz	a0,10a8e <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x1a0>
   10a9e:	4b85                	li	s7,1
   10aa0:	bf25                	j	109d8 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xea>
   10aa2:	00110537          	lui	a0,0x110
   10aa6:	4b85                	li	s7,1
   10aa8:	f2a408e3          	beq	s0,a0,109d8 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xea>
   10aac:	8566                	mv	a0,s9
   10aae:	85ee                	mv	a1,s11
   10ab0:	8656                	mv	a2,s5
   10ab2:	86d2                	mv	a3,s4
   10ab4:	875a                	mv	a4,s6
   10ab6:	00000097          	auipc	ra,0x0
   10aba:	056080e7          	jalr	86(ra) # 10b0c <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h1d292530dab90c4eE>
   10abe:	fd09                	bnez	a0,109d8 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xea>
   10ac0:	018db683          	ld	a3,24(s11)
   10ac4:	8566                	mv	a0,s9
   10ac6:	85ca                	mv	a1,s2
   10ac8:	864e                	mv	a2,s3
   10aca:	9682                	jalr	a3
   10acc:	f511                	bnez	a0,109d8 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xea>
   10ace:	4481                	li	s1,0
   10ad0:	029d0a63          	beq	s10,s1,10b04 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x216>
   10ad4:	020db603          	ld	a2,32(s11)
   10ad8:	0485                	addi	s1,s1,1
   10ada:	8566                	mv	a0,s9
   10adc:	85a2                	mv	a1,s0
   10ade:	9602                	jalr	a2
   10ae0:	d965                	beqz	a0,10ad0 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x1e2>
   10ae2:	fff48513          	addi	a0,s1,-1
   10ae6:	a005                	j	10b06 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0x218>
   10ae8:	6c94                	ld	a3,24(s1)
   10aea:	856e                	mv	a0,s11
   10aec:	85ca                	mv	a1,s2
   10aee:	864e                	mv	a2,s3
   10af0:	9682                	jalr	a3
   10af2:	ee0513e3          	bnez	a0,109d8 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xea>
   10af6:	4b81                	li	s7,0
   10af8:	034c2a23          	sw	s4,52(s8)
   10afc:	6502                	ld	a0,0(sp)
   10afe:	02ac0c23          	sb	a0,56(s8)
   10b02:	bdd9                	j	109d8 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xea>
   10b04:	856a                	mv	a0,s10
   10b06:	01a53bb3          	sltu	s7,a0,s10
   10b0a:	b5f9                	j	109d8 <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E+0xea>

0000000000010b0c <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h1d292530dab90c4eE>:
   10b0c:	7179                	addi	sp,sp,-48
   10b0e:	f406                	sd	ra,40(sp)
   10b10:	f022                	sd	s0,32(sp)
   10b12:	ec26                	sd	s1,24(sp)
   10b14:	e84a                	sd	s2,16(sp)
   10b16:	e44e                	sd	s3,8(sp)
   10b18:	0006079b          	sext.w	a5,a2
   10b1c:	00110837          	lui	a6,0x110
   10b20:	893a                	mv	s2,a4
   10b22:	84b6                	mv	s1,a3
   10b24:	842e                	mv	s0,a1
   10b26:	89aa                	mv	s3,a0
   10b28:	01078963          	beq	a5,a6,10b3a <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h1d292530dab90c4eE+0x2e>
   10b2c:	7014                	ld	a3,32(s0)
   10b2e:	854e                	mv	a0,s3
   10b30:	85b2                	mv	a1,a2
   10b32:	9682                	jalr	a3
   10b34:	85aa                	mv	a1,a0
   10b36:	4505                	li	a0,1
   10b38:	ed91                	bnez	a1,10b54 <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h1d292530dab90c4eE+0x48>
   10b3a:	cc81                	beqz	s1,10b52 <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h1d292530dab90c4eE+0x46>
   10b3c:	6c1c                	ld	a5,24(s0)
   10b3e:	854e                	mv	a0,s3
   10b40:	85a6                	mv	a1,s1
   10b42:	864a                	mv	a2,s2
   10b44:	70a2                	ld	ra,40(sp)
   10b46:	7402                	ld	s0,32(sp)
   10b48:	64e2                	ld	s1,24(sp)
   10b4a:	6942                	ld	s2,16(sp)
   10b4c:	69a2                	ld	s3,8(sp)
   10b4e:	6145                	addi	sp,sp,48
   10b50:	8782                	jr	a5
   10b52:	4501                	li	a0,0
   10b54:	70a2                	ld	ra,40(sp)
   10b56:	7402                	ld	s0,32(sp)
   10b58:	64e2                	ld	s1,24(sp)
   10b5a:	6942                	ld	s2,16(sp)
   10b5c:	69a2                	ld	s3,8(sp)
   10b5e:	6145                	addi	sp,sp,48
   10b60:	8082                	ret

0000000000010b62 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E>:
   10b62:	715d                	addi	sp,sp,-80
   10b64:	e486                	sd	ra,72(sp)
   10b66:	e0a2                	sd	s0,64(sp)
   10b68:	fc26                	sd	s1,56(sp)
   10b6a:	f84a                	sd	s2,48(sp)
   10b6c:	f44e                	sd	s3,40(sp)
   10b6e:	f052                	sd	s4,32(sp)
   10b70:	ec56                	sd	s5,24(sp)
   10b72:	e85a                	sd	s6,16(sp)
   10b74:	e45e                	sd	s7,8(sp)
   10b76:	8a2a                	mv	s4,a0
   10b78:	01053283          	ld	t0,16(a0) # 110010 <_ZN7userlib5panic5ERROR17hde6e59aa01650af9E+0xf8bf0>
   10b7c:	7108                	ld	a0,32(a0)
   10b7e:	fff28693          	addi	a3,t0,-1
   10b82:	00d036b3          	snez	a3,a3
   10b86:	fff50713          	addi	a4,a0,-1
   10b8a:	00e03733          	snez	a4,a4
   10b8e:	8ef9                	and	a3,a3,a4
   10b90:	89b2                	mv	s3,a2
   10b92:	892e                	mv	s2,a1
   10b94:	16069d63          	bnez	a3,10d0e <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x1ac>
   10b98:	4585                	li	a1,1
   10b9a:	10b51863          	bne	a0,a1,10caa <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x148>
   10b9e:	028a3503          	ld	a0,40(s4)
   10ba2:	4581                	li	a1,0
   10ba4:	013906b3          	add	a3,s2,s3
   10ba8:	00150713          	addi	a4,a0,1
   10bac:	00110337          	lui	t1,0x110
   10bb0:	0df00893          	li	a7,223
   10bb4:	0f000813          	li	a6,240
   10bb8:	864a                	mv	a2,s2
   10bba:	a801                	j	10bca <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x68>
   10bbc:	00160513          	addi	a0,a2,1
   10bc0:	8d91                	sub	a1,a1,a2
   10bc2:	95aa                	add	a1,a1,a0
   10bc4:	862a                	mv	a2,a0
   10bc6:	0e640263          	beq	s0,t1,10caa <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x148>
   10bca:	177d                	addi	a4,a4,-1
   10bcc:	c725                	beqz	a4,10c34 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0xd2>
   10bce:	0cd60e63          	beq	a2,a3,10caa <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x148>
   10bd2:	00060503          	lb	a0,0(a2)
   10bd6:	0ff57413          	andi	s0,a0,255
   10bda:	fe0551e3          	bgez	a0,10bbc <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x5a>
   10bde:	00164503          	lbu	a0,1(a2)
   10be2:	01f47793          	andi	a5,s0,31
   10be6:	03f57493          	andi	s1,a0,63
   10bea:	0288f963          	bgeu	a7,s0,10c1c <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0xba>
   10bee:	00264503          	lbu	a0,2(a2)
   10bf2:	049a                	slli	s1,s1,0x6
   10bf4:	03f57513          	andi	a0,a0,63
   10bf8:	8cc9                	or	s1,s1,a0
   10bfa:	03046763          	bltu	s0,a6,10c28 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0xc6>
   10bfe:	00364503          	lbu	a0,3(a2)
   10c02:	17f6                	slli	a5,a5,0x3d
   10c04:	93ad                	srli	a5,a5,0x2b
   10c06:	049a                	slli	s1,s1,0x6
   10c08:	03f57513          	andi	a0,a0,63
   10c0c:	8d45                	or	a0,a0,s1
   10c0e:	00f56433          	or	s0,a0,a5
   10c12:	08640c63          	beq	s0,t1,10caa <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x148>
   10c16:	00460513          	addi	a0,a2,4
   10c1a:	b75d                	j	10bc0 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x5e>
   10c1c:	00260513          	addi	a0,a2,2
   10c20:	079a                	slli	a5,a5,0x6
   10c22:	0097e433          	or	s0,a5,s1
   10c26:	bf69                	j	10bc0 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x5e>
   10c28:	00360513          	addi	a0,a2,3
   10c2c:	07b2                	slli	a5,a5,0xc
   10c2e:	00f4e433          	or	s0,s1,a5
   10c32:	b779                	j	10bc0 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x5e>
   10c34:	06d60b63          	beq	a2,a3,10caa <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x148>
   10c38:	00060503          	lb	a0,0(a2)
   10c3c:	04055363          	bgez	a0,10c82 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x120>
   10c40:	0ff57513          	andi	a0,a0,255
   10c44:	0e000693          	li	a3,224
   10c48:	02d56d63          	bltu	a0,a3,10c82 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x120>
   10c4c:	0f000693          	li	a3,240
   10c50:	02d56963          	bltu	a0,a3,10c82 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x120>
   10c54:	00164683          	lbu	a3,1(a2)
   10c58:	00264703          	lbu	a4,2(a2)
   10c5c:	03f6f693          	andi	a3,a3,63
   10c60:	03f77713          	andi	a4,a4,63
   10c64:	00364603          	lbu	a2,3(a2)
   10c68:	1576                	slli	a0,a0,0x3d
   10c6a:	912d                	srli	a0,a0,0x2b
   10c6c:	06b2                	slli	a3,a3,0xc
   10c6e:	071a                	slli	a4,a4,0x6
   10c70:	8ed9                	or	a3,a3,a4
   10c72:	03f67613          	andi	a2,a2,63
   10c76:	8e55                	or	a2,a2,a3
   10c78:	8d51                	or	a0,a0,a2
   10c7a:	00110637          	lui	a2,0x110
   10c7e:	02c50663          	beq	a0,a2,10caa <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x148>
   10c82:	c185                	beqz	a1,10ca2 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x140>
   10c84:	0135fd63          	bgeu	a1,s3,10c9e <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x13c>
   10c88:	00b90533          	add	a0,s2,a1
   10c8c:	00050503          	lb	a0,0(a0)
   10c90:	fc000613          	li	a2,-64
   10c94:	00c55763          	bge	a0,a2,10ca2 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x140>
   10c98:	4501                	li	a0,0
   10c9a:	e511                	bnez	a0,10ca6 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x144>
   10c9c:	a039                	j	10caa <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x148>
   10c9e:	ff359de3          	bne	a1,s3,10c98 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x136>
   10ca2:	854a                	mv	a0,s2
   10ca4:	c119                	beqz	a0,10caa <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x148>
   10ca6:	89ae                	mv	s3,a1
   10ca8:	892a                	mv	s2,a0
   10caa:	06028263          	beqz	t0,10d0e <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x1ac>
   10cae:	018a3403          	ld	s0,24(s4)
   10cb2:	02000513          	li	a0,32
   10cb6:	04a9f463          	bgeu	s3,a0,10cfe <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x19c>
   10cba:	4501                	li	a0,0
   10cbc:	00098e63          	beqz	s3,10cd8 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x176>
   10cc0:	85ce                	mv	a1,s3
   10cc2:	864a                	mv	a2,s2
   10cc4:	00060683          	lb	a3,0(a2) # 110000 <_ZN7userlib5panic5ERROR17hde6e59aa01650af9E+0xf8be0>
   10cc8:	0605                	addi	a2,a2,1
   10cca:	fc06a693          	slti	a3,a3,-64
   10cce:	0016c693          	xori	a3,a3,1
   10cd2:	15fd                	addi	a1,a1,-1
   10cd4:	9536                	add	a0,a0,a3
   10cd6:	f5fd                	bnez	a1,10cc4 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x162>
   10cd8:	02857b63          	bgeu	a0,s0,10d0e <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x1ac>
   10cdc:	038a4583          	lbu	a1,56(s4)
   10ce0:	468d                	li	a3,3
   10ce2:	4601                	li	a2,0
   10ce4:	00d58363          	beq	a1,a3,10cea <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x188>
   10ce8:	862e                	mv	a2,a1
   10cea:	00367593          	andi	a1,a2,3
   10cee:	40a40533          	sub	a0,s0,a0
   10cf2:	c1a1                	beqz	a1,10d32 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x1d0>
   10cf4:	4605                	li	a2,1
   10cf6:	04c59163          	bne	a1,a2,10d38 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x1d6>
   10cfa:	4a81                	li	s5,0
   10cfc:	a099                	j	10d42 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x1e0>
   10cfe:	854a                	mv	a0,s2
   10d00:	85ce                	mv	a1,s3
   10d02:	00000097          	auipc	ra,0x0
   10d06:	0c4080e7          	jalr	196(ra) # 10dc6 <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E>
   10d0a:	fc8569e3          	bltu	a0,s0,10cdc <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x17a>
   10d0e:	008a3583          	ld	a1,8(s4)
   10d12:	000a3503          	ld	a0,0(s4)
   10d16:	6d9c                	ld	a5,24(a1)
   10d18:	85ca                	mv	a1,s2
   10d1a:	864e                	mv	a2,s3
   10d1c:	60a6                	ld	ra,72(sp)
   10d1e:	6406                	ld	s0,64(sp)
   10d20:	74e2                	ld	s1,56(sp)
   10d22:	7942                	ld	s2,48(sp)
   10d24:	79a2                	ld	s3,40(sp)
   10d26:	7a02                	ld	s4,32(sp)
   10d28:	6ae2                	ld	s5,24(sp)
   10d2a:	6b42                	ld	s6,16(sp)
   10d2c:	6ba2                	ld	s7,8(sp)
   10d2e:	6161                	addi	sp,sp,80
   10d30:	8782                	jr	a5
   10d32:	8aaa                	mv	s5,a0
   10d34:	852e                	mv	a0,a1
   10d36:	a031                	j	10d42 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x1e0>
   10d38:	00150593          	addi	a1,a0,1
   10d3c:	8105                	srli	a0,a0,0x1
   10d3e:	0015da93          	srli	s5,a1,0x1
   10d42:	000a3b03          	ld	s6,0(s4)
   10d46:	008a3b83          	ld	s7,8(s4)
   10d4a:	034a2483          	lw	s1,52(s4)
   10d4e:	00150413          	addi	s0,a0,1
   10d52:	147d                	addi	s0,s0,-1
   10d54:	c809                	beqz	s0,10d66 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x204>
   10d56:	020bb603          	ld	a2,32(s7)
   10d5a:	855a                	mv	a0,s6
   10d5c:	85a6                	mv	a1,s1
   10d5e:	9602                	jalr	a2
   10d60:	d96d                	beqz	a0,10d52 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x1f0>
   10d62:	4a05                	li	s4,1
   10d64:	a82d                	j	10d9e <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x23c>
   10d66:	00110537          	lui	a0,0x110
   10d6a:	4a05                	li	s4,1
   10d6c:	02a48963          	beq	s1,a0,10d9e <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x23c>
   10d70:	018bb683          	ld	a3,24(s7)
   10d74:	855a                	mv	a0,s6
   10d76:	85ca                	mv	a1,s2
   10d78:	864e                	mv	a2,s3
   10d7a:	9682                	jalr	a3
   10d7c:	e10d                	bnez	a0,10d9e <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x23c>
   10d7e:	4401                	li	s0,0
   10d80:	008a8c63          	beq	s5,s0,10d98 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x236>
   10d84:	020bb603          	ld	a2,32(s7)
   10d88:	0405                	addi	s0,s0,1
   10d8a:	855a                	mv	a0,s6
   10d8c:	85a6                	mv	a1,s1
   10d8e:	9602                	jalr	a2
   10d90:	d965                	beqz	a0,10d80 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x21e>
   10d92:	fff40513          	addi	a0,s0,-1
   10d96:	a011                	j	10d9a <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E+0x238>
   10d98:	8556                	mv	a0,s5
   10d9a:	01553a33          	sltu	s4,a0,s5
   10d9e:	8552                	mv	a0,s4
   10da0:	60a6                	ld	ra,72(sp)
   10da2:	6406                	ld	s0,64(sp)
   10da4:	74e2                	ld	s1,56(sp)
   10da6:	7942                	ld	s2,48(sp)
   10da8:	79a2                	ld	s3,40(sp)
   10daa:	7a02                	ld	s4,32(sp)
   10dac:	6ae2                	ld	s5,24(sp)
   10dae:	6b42                	ld	s6,16(sp)
   10db0:	6ba2                	ld	s7,8(sp)
   10db2:	6161                	addi	sp,sp,80
   10db4:	8082                	ret

0000000000010db6 <_ZN42_$LT$str$u20$as$u20$core..fmt..Display$GT$3fmt17h5eb5edd471b47f2bE>:
   10db6:	86ae                	mv	a3,a1
   10db8:	85aa                	mv	a1,a0
   10dba:	8532                	mv	a0,a2
   10dbc:	8636                	mv	a2,a3
   10dbe:	00000317          	auipc	t1,0x0
   10dc2:	da430067          	jr	-604(t1) # 10b62 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E>

0000000000010dc6 <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E>:
   10dc6:	862a                	mv	a2,a0
   10dc8:	051d                	addi	a0,a0,7
   10dca:	ff857713          	andi	a4,a0,-8
   10dce:	40c708b3          	sub	a7,a4,a2
   10dd2:	0115ec63          	bltu	a1,a7,10dea <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E+0x24>
   10dd6:	41158833          	sub	a6,a1,a7
   10dda:	00883513          	sltiu	a0,a6,8
   10dde:	0098b793          	sltiu	a5,a7,9
   10de2:	0017c793          	xori	a5,a5,1
   10de6:	8d5d                	or	a0,a0,a5
   10de8:	cd11                	beqz	a0,10e04 <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E+0x3e>
   10dea:	4501                	li	a0,0
   10dec:	c999                	beqz	a1,10e02 <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E+0x3c>
   10dee:	00060683          	lb	a3,0(a2)
   10df2:	0605                	addi	a2,a2,1
   10df4:	fc06a693          	slti	a3,a3,-64
   10df8:	0016c693          	xori	a3,a3,1
   10dfc:	15fd                	addi	a1,a1,-1
   10dfe:	9536                	add	a0,a0,a3
   10e00:	f5fd                	bnez	a1,10dee <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E+0x28>
   10e02:	8082                	ret
   10e04:	00787593          	andi	a1,a6,7
   10e08:	4781                	li	a5,0
   10e0a:	00c70f63          	beq	a4,a2,10e28 <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E+0x62>
   10e0e:	40e60733          	sub	a4,a2,a4
   10e12:	8532                	mv	a0,a2
   10e14:	00050683          	lb	a3,0(a0) # 110000 <_ZN7userlib5panic5ERROR17hde6e59aa01650af9E+0xf8be0>
   10e18:	0505                	addi	a0,a0,1
   10e1a:	fc06a693          	slti	a3,a3,-64
   10e1e:	0016c693          	xori	a3,a3,1
   10e22:	0705                	addi	a4,a4,1
   10e24:	97b6                	add	a5,a5,a3
   10e26:	f77d                	bnez	a4,10e14 <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E+0x4e>
   10e28:	011602b3          	add	t0,a2,a7
   10e2c:	4601                	li	a2,0
   10e2e:	cd99                	beqz	a1,10e4c <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E+0x86>
   10e30:	ff887513          	andi	a0,a6,-8
   10e34:	00a286b3          	add	a3,t0,a0
   10e38:	00068503          	lb	a0,0(a3)
   10e3c:	0685                	addi	a3,a3,1
   10e3e:	fc052513          	slti	a0,a0,-64
   10e42:	00154513          	xori	a0,a0,1
   10e46:	15fd                	addi	a1,a1,-1
   10e48:	962a                	add	a2,a2,a0
   10e4a:	f5fd                	bnez	a1,10e38 <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E+0x72>
   10e4c:	00385713          	srli	a4,a6,0x3

0000000000010e50 <.LBB291_27>:
   10e50:	00002517          	auipc	a0,0x2
   10e54:	3b050513          	addi	a0,a0,944 # 13200 <.LCPI291_0>
   10e58:	00053f03          	ld	t5,0(a0)

0000000000010e5c <.LBB291_28>:
   10e5c:	00002517          	auipc	a0,0x2
   10e60:	3ac50513          	addi	a0,a0,940 # 13208 <.LCPI291_1>
   10e64:	00053883          	ld	a7,0(a0)
   10e68:	10001537          	lui	a0,0x10001
   10e6c:	0512                	slli	a0,a0,0x4
   10e6e:	0505                	addi	a0,a0,1
   10e70:	0542                	slli	a0,a0,0x10
   10e72:	00150813          	addi	a6,a0,1 # 10001001 <_ZN7userlib5panic5ERROR17hde6e59aa01650af9E+0xffe9be1>
   10e76:	00f60533          	add	a0,a2,a5
   10e7a:	a025                	j	10ea2 <.LBB291_28+0x46>
   10e7c:	003e1613          	slli	a2,t3,0x3
   10e80:	00c302b3          	add	t0,t1,a2
   10e84:	41c38733          	sub	a4,t2,t3
   10e88:	003e7613          	andi	a2,t3,3
   10e8c:	0115f6b3          	and	a3,a1,a7
   10e90:	81a1                	srli	a1,a1,0x8
   10e92:	0115f5b3          	and	a1,a1,a7
   10e96:	95b6                	add	a1,a1,a3
   10e98:	030585b3          	mul	a1,a1,a6
   10e9c:	91c1                	srli	a1,a1,0x30
   10e9e:	952e                	add	a0,a0,a1
   10ea0:	e241                	bnez	a2,10f20 <.LBB291_28+0xc4>
   10ea2:	d325                	beqz	a4,10e02 <_ZN4core3str5count14do_count_chars17h81c026632c8bbdf7E+0x3c>
   10ea4:	83ba                	mv	t2,a4
   10ea6:	8316                	mv	t1,t0
   10ea8:	0c000593          	li	a1,192
   10eac:	8e3a                	mv	t3,a4
   10eae:	00b76463          	bltu	a4,a1,10eb6 <.LBB291_28+0x5a>
   10eb2:	0c000e13          	li	t3,192
   10eb6:	0fce7593          	andi	a1,t3,252
   10eba:	00359613          	slli	a2,a1,0x3
   10ebe:	00c30eb3          	add	t4,t1,a2
   10ec2:	ddcd                	beqz	a1,10e7c <.LBB291_28+0x20>
   10ec4:	4581                	li	a1,0
   10ec6:	861a                	mv	a2,t1
   10ec8:	da55                	beqz	a2,10e7c <.LBB291_28+0x20>
   10eca:	6218                	ld	a4,0(a2)
   10ecc:	fff74793          	not	a5,a4
   10ed0:	839d                	srli	a5,a5,0x7
   10ed2:	8319                	srli	a4,a4,0x6
   10ed4:	6614                	ld	a3,8(a2)
   10ed6:	8f5d                	or	a4,a4,a5
   10ed8:	01e77733          	and	a4,a4,t5
   10edc:	95ba                	add	a1,a1,a4
   10ede:	fff6c713          	not	a4,a3
   10ee2:	831d                	srli	a4,a4,0x7
   10ee4:	8299                	srli	a3,a3,0x6
   10ee6:	6a1c                	ld	a5,16(a2)
   10ee8:	8ed9                	or	a3,a3,a4
   10eea:	01e6f6b3          	and	a3,a3,t5
   10eee:	95b6                	add	a1,a1,a3
   10ef0:	fff7c693          	not	a3,a5
   10ef4:	829d                	srli	a3,a3,0x7
   10ef6:	0067d713          	srli	a4,a5,0x6
   10efa:	6e1c                	ld	a5,24(a2)
   10efc:	8ed9                	or	a3,a3,a4
   10efe:	01e6f6b3          	and	a3,a3,t5
   10f02:	95b6                	add	a1,a1,a3
   10f04:	fff7c693          	not	a3,a5
   10f08:	829d                	srli	a3,a3,0x7
   10f0a:	0067d713          	srli	a4,a5,0x6
   10f0e:	8ed9                	or	a3,a3,a4
   10f10:	01e6f6b3          	and	a3,a3,t5
   10f14:	02060613          	addi	a2,a2,32
   10f18:	95b6                	add	a1,a1,a3
   10f1a:	fbd617e3          	bne	a2,t4,10ec8 <.LBB291_28+0x6c>
   10f1e:	bfb9                	j	10e7c <.LBB291_28+0x20>
   10f20:	02030a63          	beqz	t1,10f54 <.LBB291_28+0xf8>
   10f24:	0c000593          	li	a1,192
   10f28:	00b3e463          	bltu	t2,a1,10f30 <.LBB291_28+0xd4>
   10f2c:	0c000393          	li	t2,192
   10f30:	4581                	li	a1,0
   10f32:	0033f613          	andi	a2,t2,3
   10f36:	060e                	slli	a2,a2,0x3
   10f38:	000eb683          	ld	a3,0(t4)
   10f3c:	0ea1                	addi	t4,t4,8
   10f3e:	fff6c713          	not	a4,a3
   10f42:	831d                	srli	a4,a4,0x7
   10f44:	8299                	srli	a3,a3,0x6
   10f46:	8ed9                	or	a3,a3,a4
   10f48:	01e6f6b3          	and	a3,a3,t5
   10f4c:	1661                	addi	a2,a2,-8
   10f4e:	95b6                	add	a1,a1,a3
   10f50:	f665                	bnez	a2,10f38 <.LBB291_28+0xdc>
   10f52:	a011                	j	10f56 <.LBB291_28+0xfa>
   10f54:	4581                	li	a1,0
   10f56:	0115f633          	and	a2,a1,a7
   10f5a:	81a1                	srli	a1,a1,0x8
   10f5c:	0115f5b3          	and	a1,a1,a7
   10f60:	95b2                	add	a1,a1,a2
   10f62:	030585b3          	mul	a1,a1,a6
   10f66:	91c1                	srli	a1,a1,0x30
   10f68:	952e                	add	a0,a0,a1
   10f6a:	8082                	ret

0000000000010f6c <_ZN4core3fmt3num3imp7fmt_u6417h673673462c6acfe6E>:
   10f6c:	7139                	addi	sp,sp,-64
   10f6e:	fc06                	sd	ra,56(sp)
   10f70:	f822                	sd	s0,48(sp)
   10f72:	f426                	sd	s1,40(sp)
   10f74:	8832                	mv	a6,a2
   10f76:	00455693          	srli	a3,a0,0x4
   10f7a:	02700713          	li	a4,39
   10f7e:	27100793          	li	a5,625

0000000000010f82 <.LBB571_10>:
   10f82:	00001e97          	auipc	t4,0x1
   10f86:	43ee8e93          	addi	t4,t4,1086 # 123c0 <.Lanon.821f7da411c20e9d3ff3aab887ae7593.287>
   10f8a:	02f6f363          	bgeu	a3,a5,10fb0 <.LBB571_10+0x2e>
   10f8e:	06300693          	li	a3,99
   10f92:	0aa6e963          	bltu	a3,a0,11044 <.LBB571_11+0x92>
   10f96:	4629                	li	a2,10
   10f98:	0ec57763          	bgeu	a0,a2,11086 <.LBB571_11+0xd4>
   10f9c:	fff70693          	addi	a3,a4,-1
   10fa0:	00110613          	addi	a2,sp,1
   10fa4:	9636                	add	a2,a2,a3
   10fa6:	0305051b          	addiw	a0,a0,48
   10faa:	00a60023          	sb	a0,0(a2)
   10fae:	a8dd                	j	110a4 <.LBB571_11+0xf2>
   10fb0:	4701                	li	a4,0

0000000000010fb2 <.LBB571_11>:
   10fb2:	00002697          	auipc	a3,0x2
   10fb6:	2be68693          	addi	a3,a3,702 # 13270 <.LCPI571_0>
   10fba:	0006b883          	ld	a7,0(a3)
   10fbe:	6689                	lui	a3,0x2
   10fc0:	7106839b          	addiw	t2,a3,1808
   10fc4:	6685                	lui	a3,0x1
   10fc6:	47b68e1b          	addiw	t3,a3,1147
   10fca:	06400293          	li	t0,100
   10fce:	00110313          	addi	t1,sp,1
   10fd2:	05f5e6b7          	lui	a3,0x5f5e
   10fd6:	0ff68f1b          	addiw	t5,a3,255
   10fda:	87aa                	mv	a5,a0
   10fdc:	03153533          	mulhu	a0,a0,a7
   10fe0:	812d                	srli	a0,a0,0xb
   10fe2:	0275063b          	mulw	a2,a0,t2
   10fe6:	40c7863b          	subw	a2,a5,a2
   10fea:	03061693          	slli	a3,a2,0x30
   10fee:	92c9                	srli	a3,a3,0x32
   10ff0:	03c686b3          	mul	a3,a3,t3
   10ff4:	0116df93          	srli	t6,a3,0x11
   10ff8:	82c1                	srli	a3,a3,0x10
   10ffa:	7fe6f413          	andi	s0,a3,2046
   10ffe:	025f86bb          	mulw	a3,t6,t0
   11002:	9e15                	subw	a2,a2,a3
   11004:	1646                	slli	a2,a2,0x31
   11006:	9241                	srli	a2,a2,0x30
   11008:	008e86b3          	add	a3,t4,s0
   1100c:	00e30433          	add	s0,t1,a4
   11010:	0006cf83          	lbu	t6,0(a3) # 5f5e000 <_ZN7userlib5panic5ERROR17hde6e59aa01650af9E+0x5f46be0>
   11014:	00168683          	lb	a3,1(a3)
   11018:	9676                	add	a2,a2,t4
   1101a:	00160483          	lb	s1,1(a2)
   1101e:	00064603          	lbu	a2,0(a2)
   11022:	02d40223          	sb	a3,36(s0)
   11026:	03f401a3          	sb	t6,35(s0)
   1102a:	02940323          	sb	s1,38(s0)
   1102e:	02c402a3          	sb	a2,37(s0)
   11032:	1771                	addi	a4,a4,-4
   11034:	faff63e3          	bltu	t5,a5,10fda <.LBB571_11+0x28>
   11038:	02770713          	addi	a4,a4,39
   1103c:	06300693          	li	a3,99
   11040:	f4a6fbe3          	bgeu	a3,a0,10f96 <.LBB571_10+0x14>
   11044:	03051613          	slli	a2,a0,0x30
   11048:	9249                	srli	a2,a2,0x32
   1104a:	6685                	lui	a3,0x1
   1104c:	47b6869b          	addiw	a3,a3,1147
   11050:	02d60633          	mul	a2,a2,a3
   11054:	8245                	srli	a2,a2,0x11
   11056:	06400693          	li	a3,100
   1105a:	02d606bb          	mulw	a3,a2,a3
   1105e:	9d15                	subw	a0,a0,a3
   11060:	1546                	slli	a0,a0,0x31
   11062:	9141                	srli	a0,a0,0x30
   11064:	1779                	addi	a4,a4,-2
   11066:	9576                	add	a0,a0,t4
   11068:	00150683          	lb	a3,1(a0)
   1106c:	00054503          	lbu	a0,0(a0)
   11070:	00110793          	addi	a5,sp,1
   11074:	97ba                	add	a5,a5,a4
   11076:	00d780a3          	sb	a3,1(a5)
   1107a:	00a78023          	sb	a0,0(a5)
   1107e:	8532                	mv	a0,a2
   11080:	4629                	li	a2,10
   11082:	f0c56de3          	bltu	a0,a2,10f9c <.LBB571_10+0x1a>
   11086:	0506                	slli	a0,a0,0x1
   11088:	ffe70693          	addi	a3,a4,-2
   1108c:	9576                	add	a0,a0,t4
   1108e:	00150603          	lb	a2,1(a0)
   11092:	00054503          	lbu	a0,0(a0)
   11096:	00110713          	addi	a4,sp,1
   1109a:	9736                	add	a4,a4,a3
   1109c:	00c700a3          	sb	a2,1(a4)
   110a0:	00a70023          	sb	a0,0(a4)
   110a4:	00110513          	addi	a0,sp,1
   110a8:	00d50733          	add	a4,a0,a3
   110ac:	02700513          	li	a0,39
   110b0:	40d507b3          	sub	a5,a0,a3

00000000000110b4 <.LBB571_12>:
   110b4:	00001617          	auipc	a2,0x1
   110b8:	26c60613          	addi	a2,a2,620 # 12320 <.Lanon.821f7da411c20e9d3ff3aab887ae7593.2>
   110bc:	8542                	mv	a0,a6
   110be:	4681                	li	a3,0
   110c0:	00000097          	auipc	ra,0x0
   110c4:	82e080e7          	jalr	-2002(ra) # 108ee <_ZN4core3fmt9Formatter12pad_integral17hbc2b0641268f7191E>
   110c8:	70e2                	ld	ra,56(sp)
   110ca:	7442                	ld	s0,48(sp)
   110cc:	74a2                	ld	s1,40(sp)
   110ce:	6121                	addi	sp,sp,64
   110d0:	8082                	ret

00000000000110d2 <_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i32$GT$3fmt17h9e7237c084e472baE>:
   110d2:	00056503          	lwu	a0,0(a0)
   110d6:	862e                	mv	a2,a1
   110d8:	0005069b          	sext.w	a3,a0
   110dc:	0006a593          	slti	a1,a3,0
   110e0:	0015c593          	xori	a1,a1,1
   110e4:	0006d463          	bgez	a3,110ec <_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i32$GT$3fmt17h9e7237c084e472baE+0x1a>
   110e8:	40d00533          	neg	a0,a3
   110ec:	00000317          	auipc	t1,0x0
   110f0:	e8030067          	jr	-384(t1) # 10f6c <_ZN4core3fmt3num3imp7fmt_u6417h673673462c6acfe6E>

00000000000110f4 <_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17hef9fe5e8b139d194E>:
   110f4:	00056503          	lwu	a0,0(a0)
   110f8:	862e                	mv	a2,a1
   110fa:	4585                	li	a1,1
   110fc:	00000317          	auipc	t1,0x0
   11100:	e7030067          	jr	-400(t1) # 10f6c <_ZN4core3fmt3num3imp7fmt_u6417h673673462c6acfe6E>

0000000000011104 <_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h790955b0a0300c8dE>:
   11104:	6110                	ld	a2,0(a0)
   11106:	43f65513          	srai	a0,a2,0x3f
   1110a:	00a646b3          	xor	a3,a2,a0
   1110e:	40a68533          	sub	a0,a3,a0
   11112:	fff64613          	not	a2,a2
   11116:	927d                	srli	a2,a2,0x3f
   11118:	86ae                	mv	a3,a1
   1111a:	85b2                	mv	a1,a2
   1111c:	8636                	mv	a2,a3
   1111e:	00000317          	auipc	t1,0x0
   11122:	e4e30067          	jr	-434(t1) # 10f6c <_ZN4core3fmt3num3imp7fmt_u6417h673673462c6acfe6E>

0000000000011126 <_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u64$GT$3fmt17h6fab6fe087fa630eE>:
   11126:	6108                	ld	a0,0(a0)
   11128:	862e                	mv	a2,a1
   1112a:	4585                	li	a1,1
   1112c:	00000317          	auipc	t1,0x0
   11130:	e4030067          	jr	-448(t1) # 10f6c <_ZN4core3fmt3num3imp7fmt_u6417h673673462c6acfe6E>

0000000000011134 <_ZN53_$LT$core..fmt..Error$u20$as$u20$core..fmt..Debug$GT$3fmt17h242baf87e8ca9f0bE>:
   11134:	6590                	ld	a2,8(a1)
   11136:	6188                	ld	a0,0(a1)
   11138:	6e1c                	ld	a5,24(a2)

000000000001113a <.LBB603_1>:
   1113a:	00001597          	auipc	a1,0x1
   1113e:	34e58593          	addi	a1,a1,846 # 12488 <.Lanon.821f7da411c20e9d3ff3aab887ae7593.627>
   11142:	4615                	li	a2,5
   11144:	8782                	jr	a5

0000000000011146 <_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h45ca0030bea6599bE>:
   11146:	6510                	ld	a2,8(a0)
   11148:	6108                	ld	a0,0(a0)
   1114a:	6e1c                	ld	a5,24(a2)
   1114c:	8782                	jr	a5

000000000001114e <_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17h4575078792fa2417E>:
   1114e:	6114                	ld	a3,0(a0)
   11150:	6510                	ld	a2,8(a0)
   11152:	852e                	mv	a0,a1
   11154:	85b6                	mv	a1,a3
   11156:	00000317          	auipc	t1,0x0
   1115a:	a0c30067          	jr	-1524(t1) # 10b62 <_ZN4core3fmt9Formatter3pad17h3e213f500e7762a2E>
