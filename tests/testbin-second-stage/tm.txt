
time-test:     file format elf64-littleriscv


Disassembly of section .text:

0000000000010120 <exit>:
   10120:	fe010113          	addi	sp,sp,-32
   10124:	00a13423          	sd	a0,8(sp)
   10128:	00113c23          	sd	ra,24(sp)
   1012c:	550000ef          	jal	ra,1067c <__funcs_on_exit>
   10130:	550000ef          	jal	ra,10680 <__libc_exit_fini>
   10134:	4c8030ef          	jal	ra,135fc <__stdio_exit>
   10138:	00813503          	ld	a0,8(sp)
   1013c:	07d020ef          	jal	ra,129b8 <_Exit>

0000000000010140 <_start>:
   10140:	00008197          	auipc	gp,0x8
   10144:	6c018193          	addi	gp,gp,1728 # 18800 <__global_pointer$>
   10148:	00010513          	mv	a0,sp
   1014c:	00000593          	li	a1,0
   10150:	ff017113          	andi	sp,sp,-16
   10154:	0040006f          	j	10158 <_start_c>

0000000000010158 <_start_c>:
   10158:	00052583          	lw	a1,0(a0)
   1015c:	00010737          	lui	a4,0x10
   10160:	000106b7          	lui	a3,0x10
   10164:	00850613          	addi	a2,a0,8
   10168:	00010537          	lui	a0,0x10
   1016c:	00000793          	li	a5,0
   10170:	67c70713          	addi	a4,a4,1660 # 1067c <__funcs_on_exit>
   10174:	3d068693          	addi	a3,a3,976 # 103d0 <_init>
   10178:	2fc50513          	addi	a0,a0,764 # 102fc <main>
   1017c:	4a80006f          	j	10624 <__libc_start_main>

0000000000010180 <deregister_tm_clones>:
   10180:	8e818713          	addi	a4,gp,-1816 # 180e8 <__TMC_END__>
   10184:	8e818793          	addi	a5,gp,-1816 # 180e8 <__TMC_END__>
   10188:	00e78a63          	beq	a5,a4,1019c <deregister_tm_clones+0x1c>
   1018c:	00000793          	li	a5,0
   10190:	00078663          	beqz	a5,1019c <deregister_tm_clones+0x1c>
   10194:	8e818513          	addi	a0,gp,-1816 # 180e8 <__TMC_END__>
   10198:	00078067          	jr	a5
   1019c:	00008067          	ret

00000000000101a0 <register_tm_clones>:
   101a0:	8e818593          	addi	a1,gp,-1816 # 180e8 <__TMC_END__>
   101a4:	8e818793          	addi	a5,gp,-1816 # 180e8 <__TMC_END__>
   101a8:	40f585b3          	sub	a1,a1,a5
   101ac:	4035d593          	srai	a1,a1,0x3
   101b0:	00200793          	li	a5,2
   101b4:	02f5c5b3          	div	a1,a1,a5
   101b8:	00058a63          	beqz	a1,101cc <register_tm_clones+0x2c>
   101bc:	00000793          	li	a5,0
   101c0:	00078663          	beqz	a5,101cc <register_tm_clones+0x2c>
   101c4:	8e818513          	addi	a0,gp,-1816 # 180e8 <__TMC_END__>
   101c8:	00078067          	jr	a5
   101cc:	00008067          	ret

00000000000101d0 <__do_global_dtors_aux>:
   101d0:	9881c703          	lbu	a4,-1656(gp) # 18188 <completed.1>
   101d4:	04071463          	bnez	a4,1021c <__do_global_dtors_aux+0x4c>
   101d8:	ff010113          	addi	sp,sp,-16
   101dc:	00813023          	sd	s0,0(sp)
   101e0:	00113423          	sd	ra,8(sp)
   101e4:	00078413          	mv	s0,a5
   101e8:	f99ff0ef          	jal	ra,10180 <deregister_tm_clones>
   101ec:	00000793          	li	a5,0
   101f0:	00078a63          	beqz	a5,10204 <__do_global_dtors_aux+0x34>
   101f4:	00017537          	lui	a0,0x17
   101f8:	92050513          	addi	a0,a0,-1760 # 16920 <__FRAME_END__>
   101fc:	00000097          	auipc	ra,0x0
   10200:	000000e7          	jalr	zero # 0 <exit-0x10120>
   10204:	00100793          	li	a5,1
   10208:	00813083          	ld	ra,8(sp)
   1020c:	98f18423          	sb	a5,-1656(gp) # 18188 <completed.1>
   10210:	00013403          	ld	s0,0(sp)
   10214:	01010113          	addi	sp,sp,16
   10218:	00008067          	ret
   1021c:	00008067          	ret

0000000000010220 <frame_dummy>:
   10220:	00000793          	li	a5,0
   10224:	02078463          	beqz	a5,1024c <frame_dummy+0x2c>
   10228:	00017537          	lui	a0,0x17
   1022c:	ff010113          	addi	sp,sp,-16
   10230:	99018593          	addi	a1,gp,-1648 # 18190 <object.0>
   10234:	92050513          	addi	a0,a0,-1760 # 16920 <__FRAME_END__>
   10238:	00113423          	sd	ra,8(sp)
   1023c:	00000097          	auipc	ra,0x0
   10240:	000000e7          	jalr	zero # 0 <exit-0x10120>
   10244:	00813083          	ld	ra,8(sp)
   10248:	01010113          	addi	sp,sp,16
   1024c:	f55ff06f          	j	101a0 <register_tm_clones>

0000000000010250 <now_ns>:
   10250:	fe010113          	addi	sp,sp,-32
   10254:	00113c23          	sd	ra,24(sp)
   10258:	00813823          	sd	s0,16(sp)
   1025c:	02010413          	addi	s0,sp,32
   10260:	fe043023          	sd	zero,-32(s0)
   10264:	fe043423          	sd	zero,-24(s0)
   10268:	fe040793          	addi	a5,s0,-32
   1026c:	00078593          	mv	a1,a5
   10270:	00000513          	li	a0,0
   10274:	378020ef          	jal	ra,125ec <__clock_gettime>
   10278:	fe043783          	ld	a5,-32(s0)
   1027c:	00078713          	mv	a4,a5
   10280:	3b9ad7b7          	lui	a5,0x3b9ad
   10284:	a0078793          	addi	a5,a5,-1536 # 3b9aca00 <__global_pointer$+0x3b994200>
   10288:	02f707b3          	mul	a5,a4,a5
   1028c:	fe843703          	ld	a4,-24(s0)
   10290:	00e787b3          	add	a5,a5,a4
   10294:	00078513          	mv	a0,a5
   10298:	01813083          	ld	ra,24(sp)
   1029c:	01013403          	ld	s0,16(sp)
   102a0:	02010113          	addi	sp,sp,32
   102a4:	00008067          	ret

00000000000102a8 <iter>:
   102a8:	fd010113          	addi	sp,sp,-48
   102ac:	02813423          	sd	s0,40(sp)
   102b0:	03010413          	addi	s0,sp,48
   102b4:	fca43c23          	sd	a0,-40(s0)
   102b8:	fe043023          	sd	zero,-32(s0)
   102bc:	fe043423          	sd	zero,-24(s0)
   102c0:	01c0006f          	j	102dc <iter+0x34>
   102c4:	fe043783          	ld	a5,-32(s0)
   102c8:	00178793          	addi	a5,a5,1
   102cc:	fef43023          	sd	a5,-32(s0)
   102d0:	fe843783          	ld	a5,-24(s0)
   102d4:	00178793          	addi	a5,a5,1
   102d8:	fef43423          	sd	a5,-24(s0)
   102dc:	fe843703          	ld	a4,-24(s0)
   102e0:	fd843783          	ld	a5,-40(s0)
   102e4:	fef760e3          	bltu	a4,a5,102c4 <iter+0x1c>
   102e8:	00000013          	nop
   102ec:	00078513          	mv	a0,a5
   102f0:	02813403          	ld	s0,40(sp)
   102f4:	03010113          	addi	sp,sp,48
   102f8:	00008067          	ret

00000000000102fc <main>:
   102fc:	fb010113          	addi	sp,sp,-80
   10300:	04113423          	sd	ra,72(sp)
   10304:	04813023          	sd	s0,64(sp)
   10308:	05010413          	addi	s0,sp,80
   1030c:	f45ff0ef          	jal	ra,10250 <now_ns>
   10310:	fea43423          	sd	a0,-24(s0)
   10314:	000f47b7          	lui	a5,0xf4
   10318:	24078513          	addi	a0,a5,576 # f4240 <__global_pointer$+0xdba40>
   1031c:	f8dff0ef          	jal	ra,102a8 <iter>
   10320:	f31ff0ef          	jal	ra,10250 <now_ns>
   10324:	fea43023          	sd	a0,-32(s0)
   10328:	fe043703          	ld	a4,-32(s0)
   1032c:	fe843783          	ld	a5,-24(s0)
   10330:	40f707b3          	sub	a5,a4,a5
   10334:	1dcd6737          	lui	a4,0x1dcd6
   10338:	50070713          	addi	a4,a4,1280 # 1dcd6500 <__global_pointer$+0x1dcbdd00>
   1033c:	02f757b3          	divu	a5,a4,a5
   10340:	fcf43c23          	sd	a5,-40(s0)
   10344:	fd843703          	ld	a4,-40(s0)
   10348:	000f47b7          	lui	a5,0xf4
   1034c:	24078793          	addi	a5,a5,576 # f4240 <__global_pointer$+0xdba40>
   10350:	02f707b3          	mul	a5,a4,a5
   10354:	fcf43823          	sd	a5,-48(s0)
   10358:	fd043503          	ld	a0,-48(s0)
   1035c:	f4dff0ef          	jal	ra,102a8 <iter>
   10360:	ef1ff0ef          	jal	ra,10250 <now_ns>
   10364:	fca43423          	sd	a0,-56(s0)
   10368:	fc843703          	ld	a4,-56(s0)
   1036c:	fe043783          	ld	a5,-32(s0)
   10370:	40f707b3          	sub	a5,a4,a5
   10374:	fcf43023          	sd	a5,-64(s0)
   10378:	fc043703          	ld	a4,-64(s0)
   1037c:	000f47b7          	lui	a5,0xf4
   10380:	24078793          	addi	a5,a5,576 # f4240 <__global_pointer$+0xdba40>
   10384:	02f757b3          	divu	a5,a4,a5
   10388:	faf43c23          	sd	a5,-72(s0)
   1038c:	fd043783          	ld	a5,-48(s0)
   10390:	d237f753          	fcvt.d.lu	fa4,a5
   10394:	fc043783          	ld	a5,-64(s0)
   10398:	d237f7d3          	fcvt.d.lu	fa5,a5
   1039c:	1af777d3          	fdiv.d	fa5,fa4,fa5
   103a0:	faf43827          	fsd	fa5,-80(s0)
   103a4:	fb843603          	ld	a2,-72(s0)
   103a8:	fb043583          	ld	a1,-80(s0)
   103ac:	000167b7          	lui	a5,0x16
   103b0:	c0078513          	addi	a0,a5,-1024 # 15c00 <__clzdi2+0x44>
   103b4:	314000ef          	jal	ra,106c8 <printf>
   103b8:	00000793          	li	a5,0
   103bc:	00078513          	mv	a0,a5
   103c0:	04813083          	ld	ra,72(sp)
   103c4:	04013403          	ld	s0,64(sp)
   103c8:	05010113          	addi	sp,sp,80
   103cc:	00008067          	ret

00000000000103d0 <_init>:
   103d0:	00008067          	ret

00000000000103d4 <__init_ssp>:
   103d4:	00008067          	ret

00000000000103d8 <__init_libc>:
   103d8:	e8010113          	addi	sp,sp,-384
   103dc:	17213023          	sd	s2,352(sp)
   103e0:	03010913          	addi	s2,sp,48
   103e4:	16813823          	sd	s0,368(sp)
   103e8:	16913423          	sd	s1,360(sp)
   103ec:	00058413          	mv	s0,a1
   103f0:	00050493          	mv	s1,a0
   103f4:	13000613          	li	a2,304
   103f8:	00000593          	li	a1,0
   103fc:	00090513          	mv	a0,s2
   10400:	16113c23          	sd	ra,376(sp)
   10404:	0ac020ef          	jal	ra,124b0 <memset>
   10408:	9691b023          	sd	s1,-1696(gp) # 18160 <__environ>
   1040c:	00000513          	li	a0,0
   10410:	00351793          	slli	a5,a0,0x3
   10414:	00f487b3          	add	a5,s1,a5
   10418:	0007b783          	ld	a5,0(a5)
   1041c:	00150513          	addi	a0,a0,1
   10420:	fe0798e3          	bnez	a5,10410 <__init_libc+0x38>
   10424:	00351513          	slli	a0,a0,0x3
   10428:	00a48533          	add	a0,s1,a0
   1042c:	9c018793          	addi	a5,gp,-1600 # 181c0 <__libc>
   10430:	00a7b823          	sd	a0,16(a5)
   10434:	9c018493          	addi	s1,gp,-1600 # 181c0 <__libc>
   10438:	02500713          	li	a4,37
   1043c:	00053783          	ld	a5,0(a0)
   10440:	0e079e63          	bnez	a5,1053c <__init_libc+0x164>
   10444:	0b013703          	ld	a4,176(sp)
   10448:	94e1b423          	sd	a4,-1720(gp) # 18148 <__hwcap>
   1044c:	13013783          	ld	a5,304(sp)
   10450:	00078463          	beqz	a5,10458 <__init_libc+0x80>
   10454:	94f1b023          	sd	a5,-1728(gp) # 18140 <__sysinfo>
   10458:	06013783          	ld	a5,96(sp)
   1045c:	02f4bc23          	sd	a5,56(s1)
   10460:	00041a63          	bnez	s0,10474 <__init_libc+0x9c>
   10464:	12813403          	ld	s0,296(sp)
   10468:	00041663          	bnez	s0,10474 <__init_libc+0x9c>
   1046c:	00016437          	lui	s0,0x16
   10470:	c3840413          	addi	s0,s0,-968 # 15c38 <__clzdi2+0x7c>
   10474:	9481bc23          	sd	s0,-1704(gp) # 18158 <__progname_full>
   10478:	9481b823          	sd	s0,-1712(gp) # 18150 <__progname>
   1047c:	02f00693          	li	a3,47
   10480:	00140413          	addi	s0,s0,1
   10484:	fff44703          	lbu	a4,-1(s0)
   10488:	0c071a63          	bnez	a4,1055c <__init_libc+0x184>
   1048c:	00090513          	mv	a0,s2
   10490:	328020ef          	jal	ra,127b8 <__init_tls>
   10494:	0f813503          	ld	a0,248(sp)
   10498:	f3dff0ef          	jal	ra,103d4 <__init_ssp>
   1049c:	08813703          	ld	a4,136(sp)
   104a0:	09013783          	ld	a5,144(sp)
   104a4:	00f71c63          	bne	a4,a5,104bc <__init_libc+0xe4>
   104a8:	09813703          	ld	a4,152(sp)
   104ac:	0a013783          	ld	a5,160(sp)
   104b0:	00f71663          	bne	a4,a5,104bc <__init_libc+0xe4>
   104b4:	0e813783          	ld	a5,232(sp)
   104b8:	0c078263          	beqz	a5,1057c <__init_libc+0x1a4>
   104bc:	02013023          	sd	zero,32(sp)
   104c0:	00100793          	li	a5,1
   104c4:	02013423          	sd	zero,40(sp)
   104c8:	02f12023          	sw	a5,32(sp)
   104cc:	00200793          	li	a5,2
   104d0:	00013c23          	sd	zero,24(sp)
   104d4:	02f12423          	sw	a5,40(sp)
   104d8:	00013423          	sd	zero,8(sp)
   104dc:	00013823          	sd	zero,16(sp)
   104e0:	04900893          	li	a7,73
   104e4:	01810513          	addi	a0,sp,24
   104e8:	00300593          	li	a1,3
   104ec:	00810613          	addi	a2,sp,8
   104f0:	00000693          	li	a3,0
   104f4:	00800713          	li	a4,8
   104f8:	00000073          	ecall
   104fc:	0005051b          	sext.w	a0,a0
   10500:	01810793          	addi	a5,sp,24
   10504:	02054863          	bltz	a0,10534 <__init_libc+0x15c>
   10508:	00008637          	lui	a2,0x8
   1050c:	000166b7          	lui	a3,0x16
   10510:	00260613          	addi	a2,a2,2 # 8002 <exit-0x811e>
   10514:	0067d703          	lhu	a4,6(a5)
   10518:	02077713          	andi	a4,a4,32
   1051c:	04070863          	beqz	a4,1056c <__init_libc+0x194>
   10520:	03800893          	li	a7,56
   10524:	f9c00513          	li	a0,-100
   10528:	c4068593          	addi	a1,a3,-960 # 15c40 <__clzdi2+0x84>
   1052c:	00000073          	ecall
   10530:	02055e63          	bgez	a0,1056c <__init_libc+0x194>
   10534:	00000023          	sb	zero,0(zero) # 0 <exit-0x10120>
   10538:	00100073          	ebreak
   1053c:	00f76c63          	bltu	a4,a5,10554 <__init_libc+0x17c>
   10540:	16010693          	addi	a3,sp,352
   10544:	00379793          	slli	a5,a5,0x3
   10548:	00f687b3          	add	a5,a3,a5
   1054c:	00853683          	ld	a3,8(a0)
   10550:	ecd7b823          	sd	a3,-304(a5)
   10554:	01050513          	addi	a0,a0,16
   10558:	ee5ff06f          	j	1043c <__init_libc+0x64>
   1055c:	00d71463          	bne	a4,a3,10564 <__init_libc+0x18c>
   10560:	9481b823          	sd	s0,-1712(gp) # 18150 <__progname>
   10564:	00140413          	addi	s0,s0,1
   10568:	f1dff06f          	j	10484 <__init_libc+0xac>
   1056c:	00878793          	addi	a5,a5,8
   10570:	faf912e3          	bne	s2,a5,10514 <__init_libc+0x13c>
   10574:	00100793          	li	a5,1
   10578:	00f4a423          	sw	a5,8(s1)
   1057c:	17813083          	ld	ra,376(sp)
   10580:	17013403          	ld	s0,368(sp)
   10584:	16813483          	ld	s1,360(sp)
   10588:	16013903          	ld	s2,352(sp)
   1058c:	18010113          	addi	sp,sp,384
   10590:	00008067          	ret

0000000000010594 <__libc_start_init>:
   10594:	fe010113          	addi	sp,sp,-32
   10598:	00813823          	sd	s0,16(sp)
   1059c:	00913423          	sd	s1,8(sp)
   105a0:	00018437          	lui	s0,0x18
   105a4:	000184b7          	lui	s1,0x18
   105a8:	00113c23          	sd	ra,24(sp)
   105ac:	ff040413          	addi	s0,s0,-16 # 17ff0 <__frame_dummy_init_array_entry>
   105b0:	e21ff0ef          	jal	ra,103d0 <_init>
   105b4:	ff848493          	addi	s1,s1,-8 # 17ff8 <__do_global_dtors_aux_fini_array_entry>
   105b8:	00946c63          	bltu	s0,s1,105d0 <__libc_start_init+0x3c>
   105bc:	01813083          	ld	ra,24(sp)
   105c0:	01013403          	ld	s0,16(sp)
   105c4:	00813483          	ld	s1,8(sp)
   105c8:	02010113          	addi	sp,sp,32
   105cc:	00008067          	ret
   105d0:	00043783          	ld	a5,0(s0)
   105d4:	00840413          	addi	s0,s0,8
   105d8:	000780e7          	jalr	a5
   105dc:	fddff06f          	j	105b8 <__libc_start_init+0x24>

00000000000105e0 <libc_start_main_stage2>:
   105e0:	fd010113          	addi	sp,sp,-48
   105e4:	02813023          	sd	s0,32(sp)
   105e8:	00158413          	addi	s0,a1,1
   105ec:	00341413          	slli	s0,s0,0x3
   105f0:	02113423          	sd	ra,40(sp)
   105f4:	00913c23          	sd	s1,24(sp)
   105f8:	01213823          	sd	s2,16(sp)
   105fc:	00058493          	mv	s1,a1
   10600:	00050913          	mv	s2,a0
   10604:	00860433          	add	s0,a2,s0
   10608:	00c13423          	sd	a2,8(sp)
   1060c:	f89ff0ef          	jal	ra,10594 <__libc_start_init>
   10610:	00813583          	ld	a1,8(sp)
   10614:	00040613          	mv	a2,s0
   10618:	00048513          	mv	a0,s1
   1061c:	000900e7          	jalr	s2
   10620:	b01ff0ef          	jal	ra,10120 <exit>

0000000000010624 <__libc_start_main>:
   10624:	fd010113          	addi	sp,sp,-48
   10628:	02813023          	sd	s0,32(sp)
   1062c:	00913c23          	sd	s1,24(sp)
   10630:	00058413          	mv	s0,a1
   10634:	00050493          	mv	s1,a0
   10638:	00158513          	addi	a0,a1,1
   1063c:	00063583          	ld	a1,0(a2)
   10640:	00351513          	slli	a0,a0,0x3
   10644:	00a60533          	add	a0,a2,a0
   10648:	02113423          	sd	ra,40(sp)
   1064c:	00c13423          	sd	a2,8(sp)
   10650:	d89ff0ef          	jal	ra,103d8 <__init_libc>
   10654:	000107b7          	lui	a5,0x10
   10658:	5e078793          	addi	a5,a5,1504 # 105e0 <libc_start_main_stage2>
   1065c:	00040593          	mv	a1,s0
   10660:	02013403          	ld	s0,32(sp)
   10664:	00813603          	ld	a2,8(sp)
   10668:	02813083          	ld	ra,40(sp)
   1066c:	00048513          	mv	a0,s1
   10670:	01813483          	ld	s1,24(sp)
   10674:	03010113          	addi	sp,sp,48
   10678:	00078067          	jr	a5

000000000001067c <__funcs_on_exit>:
   1067c:	00008067          	ret

0000000000010680 <__libc_exit_fini>:
   10680:	fe010113          	addi	sp,sp,-32
   10684:	00813823          	sd	s0,16(sp)
   10688:	00913423          	sd	s1,8(sp)
   1068c:	00018437          	lui	s0,0x18
   10690:	000184b7          	lui	s1,0x18
   10694:	00113c23          	sd	ra,24(sp)
   10698:	00040413          	mv	s0,s0
   1069c:	ff848493          	addi	s1,s1,-8 # 17ff8 <__do_global_dtors_aux_fini_array_entry>
   106a0:	0084ec63          	bltu	s1,s0,106b8 <__libc_exit_fini+0x38>
   106a4:	01013403          	ld	s0,16(sp)
   106a8:	01813083          	ld	ra,24(sp)
   106ac:	00813483          	ld	s1,8(sp)
   106b0:	02010113          	addi	sp,sp,32
   106b4:	fc9ff06f          	j	1067c <__funcs_on_exit>
   106b8:	ff843783          	ld	a5,-8(s0) # 17ff8 <__do_global_dtors_aux_fini_array_entry>
   106bc:	ff840413          	addi	s0,s0,-8
   106c0:	000780e7          	jalr	a5
   106c4:	fddff06f          	j	106a0 <__libc_exit_fini+0x20>

00000000000106c8 <printf>:
   106c8:	fa010113          	addi	sp,sp,-96
   106cc:	02b13423          	sd	a1,40(sp)
   106d0:	00050593          	mv	a1,a0
   106d4:	00018537          	lui	a0,0x18
   106d8:	02c13823          	sd	a2,48(sp)
   106dc:	00050513          	mv	a0,a0
   106e0:	02810613          	addi	a2,sp,40
   106e4:	00113c23          	sd	ra,24(sp)
   106e8:	02d13c23          	sd	a3,56(sp)
   106ec:	04e13023          	sd	a4,64(sp)
   106f0:	04f13423          	sd	a5,72(sp)
   106f4:	05013823          	sd	a6,80(sp)
   106f8:	05113c23          	sd	a7,88(sp)
   106fc:	00c13423          	sd	a2,8(sp)
   10700:	439010ef          	jal	ra,12338 <vfprintf>
   10704:	01813083          	ld	ra,24(sp)
   10708:	06010113          	addi	sp,sp,96
   1070c:	00008067          	ret

0000000000010710 <pop_arg>:
   10710:	ff75859b          	addiw	a1,a1,-9
   10714:	0005871b          	sext.w	a4,a1
   10718:	01100793          	li	a5,17
   1071c:	0ee7ea63          	bltu	a5,a4,10810 <pop_arg+0x100>
   10720:	02059593          	slli	a1,a1,0x20
   10724:	000167b7          	lui	a5,0x16
   10728:	c4c78793          	addi	a5,a5,-948 # 15c4c <__clzdi2+0x90>
   1072c:	01e5d593          	srli	a1,a1,0x1e
   10730:	00f585b3          	add	a1,a1,a5
   10734:	0005a703          	lw	a4,0(a1)
   10738:	ff010113          	addi	sp,sp,-16
   1073c:	00813023          	sd	s0,0(sp)
   10740:	00063783          	ld	a5,0(a2)
   10744:	00113423          	sd	ra,8(sp)
   10748:	00050413          	mv	s0,a0
   1074c:	00070067          	jr	a4
   10750:	00878713          	addi	a4,a5,8
   10754:	0007a783          	lw	a5,0(a5)
   10758:	00e63023          	sd	a4,0(a2)
   1075c:	00f43023          	sd	a5,0(s0)
   10760:	00813083          	ld	ra,8(sp)
   10764:	00013403          	ld	s0,0(sp)
   10768:	01010113          	addi	sp,sp,16
   1076c:	00008067          	ret
   10770:	00878713          	addi	a4,a5,8
   10774:	00e63023          	sd	a4,0(a2)
   10778:	0007e783          	lwu	a5,0(a5)
   1077c:	fe1ff06f          	j	1075c <pop_arg+0x4c>
   10780:	00878713          	addi	a4,a5,8
   10784:	00e63023          	sd	a4,0(a2)
   10788:	00079783          	lh	a5,0(a5)
   1078c:	fd1ff06f          	j	1075c <pop_arg+0x4c>
   10790:	00878713          	addi	a4,a5,8
   10794:	00e63023          	sd	a4,0(a2)
   10798:	0007d783          	lhu	a5,0(a5)
   1079c:	fc1ff06f          	j	1075c <pop_arg+0x4c>
   107a0:	00878713          	addi	a4,a5,8
   107a4:	00e63023          	sd	a4,0(a2)
   107a8:	00078783          	lb	a5,0(a5)
   107ac:	fb1ff06f          	j	1075c <pop_arg+0x4c>
   107b0:	00878713          	addi	a4,a5,8
   107b4:	00e63023          	sd	a4,0(a2)
   107b8:	0007c783          	lbu	a5,0(a5)
   107bc:	fa1ff06f          	j	1075c <pop_arg+0x4c>
   107c0:	00878713          	addi	a4,a5,8
   107c4:	00e63023          	sd	a4,0(a2)
   107c8:	0007b783          	ld	a5,0(a5)
   107cc:	f91ff06f          	j	1075c <pop_arg+0x4c>
   107d0:	0007b503          	ld	a0,0(a5)
   107d4:	00878713          	addi	a4,a5,8
   107d8:	00e63023          	sd	a4,0(a2)
   107dc:	2c4050ef          	jal	ra,15aa0 <__extenddftf2>
   107e0:	00a43023          	sd	a0,0(s0)
   107e4:	00b43423          	sd	a1,8(s0)
   107e8:	f79ff06f          	j	10760 <pop_arg+0x50>
   107ec:	00f78793          	addi	a5,a5,15
   107f0:	ff07f793          	andi	a5,a5,-16
   107f4:	01078713          	addi	a4,a5,16
   107f8:	00e63023          	sd	a4,0(a2)
   107fc:	0007b703          	ld	a4,0(a5)
   10800:	0087b783          	ld	a5,8(a5)
   10804:	00e53023          	sd	a4,0(a0) # 18000 <__stdout_FILE>
   10808:	00f53423          	sd	a5,8(a0)
   1080c:	f55ff06f          	j	10760 <pop_arg+0x50>
   10810:	00008067          	ret

0000000000010814 <fmt_u>:
   10814:	00050793          	mv	a5,a0
   10818:	00a00693          	li	a3,10
   1081c:	00058513          	mv	a0,a1
   10820:	00079463          	bnez	a5,10828 <fmt_u+0x14>
   10824:	00008067          	ret
   10828:	02d7f733          	remu	a4,a5,a3
   1082c:	fff50513          	addi	a0,a0,-1
   10830:	0307071b          	addiw	a4,a4,48
   10834:	02d7d7b3          	divu	a5,a5,a3
   10838:	00e50023          	sb	a4,0(a0)
   1083c:	fe5ff06f          	j	10820 <fmt_u+0xc>

0000000000010840 <getint>:
   10840:	0cccd637          	lui	a2,0xcccd
   10844:	800005b7          	lui	a1,0x80000
   10848:	00050693          	mv	a3,a0
   1084c:	00900313          	li	t1,9
   10850:	00000513          	li	a0,0
   10854:	ccc60613          	addi	a2,a2,-820 # ccccccc <__global_pointer$+0xccb44cc>
   10858:	ff600e13          	li	t3,-10
   1085c:	fff5c593          	not	a1,a1
   10860:	00a00e93          	li	t4,10
   10864:	0006b703          	ld	a4,0(a3)
   10868:	00074783          	lbu	a5,0(a4)
   1086c:	fd07889b          	addiw	a7,a5,-48
   10870:	00088793          	mv	a5,a7
   10874:	01137463          	bgeu	t1,a7,1087c <getint+0x3c>
   10878:	00008067          	ret
   1087c:	02a66263          	bltu	a2,a0,108a0 <getint+0x60>
   10880:	02ae083b          	mulw	a6,t3,a0
   10884:	00b8083b          	addw	a6,a6,a1
   10888:	01184c63          	blt	a6,a7,108a0 <getint+0x60>
   1088c:	02ae853b          	mulw	a0,t4,a0
   10890:	00a7853b          	addw	a0,a5,a0
   10894:	00170713          	addi	a4,a4,1
   10898:	00e6b023          	sd	a4,0(a3)
   1089c:	fc9ff06f          	j	10864 <getint+0x24>
   108a0:	fff00513          	li	a0,-1
   108a4:	ff1ff06f          	j	10894 <getint+0x54>

00000000000108a8 <out>:
   108a8:	00050793          	mv	a5,a0
   108ac:	0007a703          	lw	a4,0(a5)
   108b0:	00058513          	mv	a0,a1
   108b4:	00060593          	mv	a1,a2
   108b8:	02077713          	andi	a4,a4,32
   108bc:	00071663          	bnez	a4,108c8 <out+0x20>
   108c0:	00078613          	mv	a2,a5
   108c4:	4e00206f          	j	12da4 <__fwritex>
   108c8:	00008067          	ret

00000000000108cc <pad>:
   108cc:	000127b7          	lui	a5,0x12
   108d0:	00f77733          	and	a4,a4,a5
   108d4:	08071e63          	bnez	a4,10970 <pad+0xa4>
   108d8:	08c6dc63          	bge	a3,a2,10970 <pad+0xa4>
   108dc:	ed010113          	addi	sp,sp,-304
   108e0:	12813023          	sd	s0,288(sp)
   108e4:	10913c23          	sd	s1,280(sp)
   108e8:	11213823          	sd	s2,272(sp)
   108ec:	40d604bb          	subw	s1,a2,a3
   108f0:	12113423          	sd	ra,296(sp)
   108f4:	11313423          	sd	s3,264(sp)
   108f8:	10000793          	li	a5,256
   108fc:	00050913          	mv	s2,a0
   10900:	00048413          	mv	s0,s1
   10904:	0004861b          	sext.w	a2,s1
   10908:	0097d463          	bge	a5,s1,10910 <pad+0x44>
   1090c:	10000613          	li	a2,256
   10910:	00010513          	mv	a0,sp
   10914:	39d010ef          	jal	ra,124b0 <memset>
   10918:	0ff00993          	li	s3,255
   1091c:	0299ce63          	blt	s3,s1,10958 <pad+0x8c>
   10920:	0084561b          	srliw	a2,s0,0x8
   10924:	f0000793          	li	a5,-256
   10928:	02f6063b          	mulw	a2,a2,a5
   1092c:	00010593          	mv	a1,sp
   10930:	00090513          	mv	a0,s2
   10934:	0086063b          	addw	a2,a2,s0
   10938:	f71ff0ef          	jal	ra,108a8 <out>
   1093c:	12813083          	ld	ra,296(sp)
   10940:	12013403          	ld	s0,288(sp)
   10944:	11813483          	ld	s1,280(sp)
   10948:	11013903          	ld	s2,272(sp)
   1094c:	10813983          	ld	s3,264(sp)
   10950:	13010113          	addi	sp,sp,304
   10954:	00008067          	ret
   10958:	10000613          	li	a2,256
   1095c:	00010593          	mv	a1,sp
   10960:	00090513          	mv	a0,s2
   10964:	f45ff0ef          	jal	ra,108a8 <out>
   10968:	f004849b          	addiw	s1,s1,-256
   1096c:	fb1ff06f          	j	1091c <pad+0x50>
   10970:	00008067          	ret

0000000000010974 <fmt_fp>:
   10974:	fffff337          	lui	t1,0xfffff
   10978:	81010113          	addi	sp,sp,-2032
   1097c:	a3030313          	addi	t1,t1,-1488 # ffffffffffffea30 <__global_pointer$+0xfffffffffffe6230>
   10980:	7e813023          	sd	s0,2016(sp)
   10984:	7c913c23          	sd	s1,2008(sp)
   10988:	7d313423          	sd	s3,1992(sp)
   1098c:	7d413023          	sd	s4,1984(sp)
   10990:	7b613823          	sd	s6,1968(sp)
   10994:	7b813023          	sd	s8,1952(sp)
   10998:	79913c23          	sd	s9,1944(sp)
   1099c:	7e113423          	sd	ra,2024(sp)
   109a0:	7d213823          	sd	s2,2000(sp)
   109a4:	7b513c23          	sd	s5,1976(sp)
   109a8:	7b713423          	sd	s7,1960(sp)
   109ac:	79a13823          	sd	s10,1936(sp)
   109b0:	79b13423          	sd	s11,1928(sp)
   109b4:	00070413          	mv	s0,a4
   109b8:	00610133          	add	sp,sp,t1
   109bc:	00002737          	lui	a4,0x2
   109c0:	00068993          	mv	s3,a3
   109c4:	d1070713          	addi	a4,a4,-752 # 1d10 <exit-0xe410>
   109c8:	04010693          	addi	a3,sp,64
   109cc:	00078a13          	mv	s4,a5
   109d0:	00d70733          	add	a4,a4,a3
   109d4:	ffffe7b7          	lui	a5,0xffffe
   109d8:	00f707b3          	add	a5,a4,a5
   109dc:	00050493          	mv	s1,a0
   109e0:	00058c93          	mv	s9,a1
   109e4:	00058513          	mv	a0,a1
   109e8:	00060593          	mv	a1,a2
   109ec:	00060c13          	mv	s8,a2
   109f0:	00080b13          	mv	s6,a6
   109f4:	00f13023          	sd	a5,0(sp)
   109f8:	2e07ae23          	sw	zero,764(a5) # ffffffffffffe2fc <__global_pointer$+0xfffffffffffe5afc>
   109fc:	068020ef          	jal	ra,12a64 <__signbitl>
   10a00:	10050e63          	beqz	a0,10b1c <fmt_fp+0x1a8>
   10a04:	fff00793          	li	a5,-1
   10a08:	03f79793          	slli	a5,a5,0x3f
   10a0c:	00016937          	lui	s2,0x16
   10a10:	00fc4c33          	xor	s8,s8,a5
   10a14:	00100a93          	li	s5,1
   10a18:	c9890913          	addi	s2,s2,-872 # 15c98 <__clzdi2+0xdc>
   10a1c:	000c8513          	mv	a0,s9
   10a20:	000c0593          	mv	a1,s8
   10a24:	7fd010ef          	jal	ra,12a20 <__fpclassifyl>
   10a28:	00100793          	li	a5,1
   10a2c:	14a7c863          	blt	a5,a0,10b7c <fmt_fp+0x208>
   10a30:	020b7b13          	andi	s6,s6,32
   10a34:	120b1863          	bnez	s6,10b64 <fmt_fp+0x1f0>
   10a38:	00016bb7          	lui	s7,0x16
   10a3c:	cb8b8b93          	addi	s7,s7,-840 # 15cb8 <__clzdi2+0xfc>
   10a40:	000c8613          	mv	a2,s9
   10a44:	000c0693          	mv	a3,s8
   10a48:	000c8513          	mv	a0,s9
   10a4c:	000c0593          	mv	a1,s8
   10a50:	195030ef          	jal	ra,143e4 <__eqtf2>
   10a54:	00050863          	beqz	a0,10a64 <fmt_fp+0xf0>
   10a58:	100b1c63          	bnez	s6,10b70 <fmt_fp+0x1fc>
   10a5c:	00016bb7          	lui	s7,0x16
   10a60:	cc8b8b93          	addi	s7,s7,-824 # 15cc8 <__clzdi2+0x10c>
   10a64:	ffff0737          	lui	a4,0xffff0
   10a68:	003a8b1b          	addiw	s6,s5,3
   10a6c:	fff70713          	addi	a4,a4,-1 # fffffffffffeffff <__global_pointer$+0xfffffffffffd77ff>
   10a70:	00ea7733          	and	a4,s4,a4
   10a74:	000b0693          	mv	a3,s6
   10a78:	00098613          	mv	a2,s3
   10a7c:	02000593          	li	a1,32
   10a80:	00048513          	mv	a0,s1
   10a84:	e49ff0ef          	jal	ra,108cc <pad>
   10a88:	000a8613          	mv	a2,s5
   10a8c:	00090593          	mv	a1,s2
   10a90:	00048513          	mv	a0,s1
   10a94:	e15ff0ef          	jal	ra,108a8 <out>
   10a98:	00300613          	li	a2,3
   10a9c:	000b8593          	mv	a1,s7
   10aa0:	00048513          	mv	a0,s1
   10aa4:	e05ff0ef          	jal	ra,108a8 <out>
   10aa8:	00002737          	lui	a4,0x2
   10aac:	00ea4733          	xor	a4,s4,a4
   10ab0:	000b0693          	mv	a3,s6
   10ab4:	00098613          	mv	a2,s3
   10ab8:	02000593          	li	a1,32
   10abc:	00048513          	mv	a0,s1
   10ac0:	000b0413          	mv	s0,s6
   10ac4:	e09ff0ef          	jal	ra,108cc <pad>
   10ac8:	013b5463          	bge	s6,s3,10ad0 <fmt_fp+0x15c>
   10acc:	00098413          	mv	s0,s3
   10ad0:	0004051b          	sext.w	a0,s0
   10ad4:	00001337          	lui	t1,0x1
   10ad8:	5d030313          	addi	t1,t1,1488 # 15d0 <exit-0xeb50>
   10adc:	00610133          	add	sp,sp,t1
   10ae0:	7e813083          	ld	ra,2024(sp)
   10ae4:	7e013403          	ld	s0,2016(sp)
   10ae8:	7d813483          	ld	s1,2008(sp)
   10aec:	7d013903          	ld	s2,2000(sp)
   10af0:	7c813983          	ld	s3,1992(sp)
   10af4:	7c013a03          	ld	s4,1984(sp)
   10af8:	7b813a83          	ld	s5,1976(sp)
   10afc:	7b013b03          	ld	s6,1968(sp)
   10b00:	7a813b83          	ld	s7,1960(sp)
   10b04:	7a013c03          	ld	s8,1952(sp)
   10b08:	79813c83          	ld	s9,1944(sp)
   10b0c:	79013d03          	ld	s10,1936(sp)
   10b10:	78813d83          	ld	s11,1928(sp)
   10b14:	7f010113          	addi	sp,sp,2032
   10b18:	00008067          	ret
   10b1c:	00ba5713          	srli	a4,s4,0xb
   10b20:	00177713          	andi	a4,a4,1
   10b24:	000a079b          	sext.w	a5,s4
   10b28:	00071e63          	bnez	a4,10b44 <fmt_fp+0x1d0>
   10b2c:	0017f793          	andi	a5,a5,1
   10b30:	02079263          	bnez	a5,10b54 <fmt_fp+0x1e0>
   10b34:	00016937          	lui	s2,0x16
   10b38:	00050a93          	mv	s5,a0
   10b3c:	c9990913          	addi	s2,s2,-871 # 15c99 <__clzdi2+0xdd>
   10b40:	eddff06f          	j	10a1c <fmt_fp+0xa8>
   10b44:	00016937          	lui	s2,0x16
   10b48:	00100a93          	li	s5,1
   10b4c:	c9b90913          	addi	s2,s2,-869 # 15c9b <__clzdi2+0xdf>
   10b50:	ecdff06f          	j	10a1c <fmt_fp+0xa8>
   10b54:	00016937          	lui	s2,0x16
   10b58:	00100a93          	li	s5,1
   10b5c:	c9e90913          	addi	s2,s2,-866 # 15c9e <__clzdi2+0xe2>
   10b60:	ebdff06f          	j	10a1c <fmt_fp+0xa8>
   10b64:	00016bb7          	lui	s7,0x16
   10b68:	cb0b8b93          	addi	s7,s7,-848 # 15cb0 <__clzdi2+0xf4>
   10b6c:	ed5ff06f          	j	10a40 <fmt_fp+0xcc>
   10b70:	00016bb7          	lui	s7,0x16
   10b74:	cc0b8b93          	addi	s7,s7,-832 # 15cc0 <__clzdi2+0x104>
   10b78:	eedff06f          	j	10a64 <fmt_fp+0xf0>
   10b7c:	00002bb7          	lui	s7,0x2
   10b80:	04010713          	addi	a4,sp,64
   10b84:	ffffed37          	lui	s10,0xffffe
   10b88:	d10b8793          	addi	a5,s7,-752 # 1d10 <exit-0xe410>
   10b8c:	00e787b3          	add	a5,a5,a4
   10b90:	2fcd0613          	addi	a2,s10,764 # ffffffffffffe2fc <__global_pointer$+0xfffffffffffe5afc>
   10b94:	00c78633          	add	a2,a5,a2
   10b98:	000c8513          	mv	a0,s9
   10b9c:	000c0593          	mv	a1,s8
   10ba0:	6cd010ef          	jal	ra,12a6c <frexpl>
   10ba4:	00050613          	mv	a2,a0
   10ba8:	00058693          	mv	a3,a1
   10bac:	541020ef          	jal	ra,138ec <__addtf3>
   10bb0:	00000613          	li	a2,0
   10bb4:	00000693          	li	a3,0
   10bb8:	00050c93          	mv	s9,a0
   10bbc:	00058d93          	mv	s11,a1
   10bc0:	00a13023          	sd	a0,0(sp)
   10bc4:	00058c13          	mv	s8,a1
   10bc8:	01d030ef          	jal	ra,143e4 <__eqtf2>
   10bcc:	000c8713          	mv	a4,s9
   10bd0:	02050263          	beqz	a0,10bf4 <fmt_fp+0x280>
   10bd4:	04010693          	addi	a3,sp,64
   10bd8:	d10b8793          	addi	a5,s7,-752
   10bdc:	00d787b3          	add	a5,a5,a3
   10be0:	01a787b3          	add	a5,a5,s10
   10be4:	2fc7a683          	lw	a3,764(a5)
   10be8:	00f13023          	sd	a5,0(sp)
   10bec:	fff6869b          	addiw	a3,a3,-1
   10bf0:	2ed7ae23          	sw	a3,764(a5)
   10bf4:	020b6793          	ori	a5,s6,32
   10bf8:	00f13023          	sd	a5,0(sp)
   10bfc:	06100693          	li	a3,97
   10c00:	3cd79a63          	bne	a5,a3,10fd4 <fmt_fp+0x660>
   10c04:	020b7793          	andi	a5,s6,32
   10c08:	00f13c23          	sd	a5,24(sp)
   10c0c:	00078463          	beqz	a5,10c14 <fmt_fp+0x2a0>
   10c10:	00990913          	addi	s2,s2,9
   10c14:	002a879b          	addiw	a5,s5,2
   10c18:	00078a93          	mv	s5,a5
   10c1c:	00f13023          	sd	a5,0(sp)
   10c20:	0004079b          	sext.w	a5,s0
   10c24:	00f13823          	sd	a5,16(sp)
   10c28:	01a00693          	li	a3,26
   10c2c:	00016d37          	lui	s10,0x16
   10c30:	0686e263          	bltu	a3,s0,10c94 <fmt_fp+0x320>
   10c34:	df8d3b83          	ld	s7,-520(s10) # 15df8 <__clzdi2+0x23c>
   10c38:	01b00c13          	li	s8,27
   10c3c:	408c0c3b          	subw	s8,s8,s0
   10c40:	00000c93          	li	s9,0
   10c44:	fff00813          	li	a6,-1
   10c48:	000b8693          	mv	a3,s7
   10c4c:	fffc0c1b          	addiw	s8,s8,-1
   10c50:	210c1863          	bne	s8,a6,10e60 <fmt_fp+0x4ec>
   10c54:	00094603          	lbu	a2,0(s2)
   10c58:	02d00693          	li	a3,45
   10c5c:	22d61a63          	bne	a2,a3,10e90 <fmt_fp+0x51c>
   10c60:	03fc1c13          	slli	s8,s8,0x3f
   10c64:	000c8613          	mv	a2,s9
   10c68:	000b8693          	mv	a3,s7
   10c6c:	00070513          	mv	a0,a4
   10c70:	018dc5b3          	xor	a1,s11,s8
   10c74:	0b8040ef          	jal	ra,14d2c <__subtf3>
   10c78:	00050613          	mv	a2,a0
   10c7c:	00058693          	mv	a3,a1
   10c80:	000c8513          	mv	a0,s9
   10c84:	000b8593          	mv	a1,s7
   10c88:	465020ef          	jal	ra,138ec <__addtf3>
   10c8c:	00050c93          	mv	s9,a0
   10c90:	0185cc33          	xor	s8,a1,s8
   10c94:	000026b7          	lui	a3,0x2
   10c98:	ffffe7b7          	lui	a5,0xffffe
   10c9c:	04010613          	addi	a2,sp,64
   10ca0:	d1068713          	addi	a4,a3,-752 # 1d10 <exit-0xe410>
   10ca4:	30078b93          	addi	s7,a5,768 # ffffffffffffe300 <__global_pointer$+0xfffffffffffe5b00>
   10ca8:	00c70733          	add	a4,a4,a2
   10cac:	01770bb3          	add	s7,a4,s7
   10cb0:	d1068713          	addi	a4,a3,-752
   10cb4:	00c70733          	add	a4,a4,a2
   10cb8:	00f707b3          	add	a5,a4,a5
   10cbc:	00f13423          	sd	a5,8(sp)
   10cc0:	2fc7a783          	lw	a5,764(a5)
   10cc4:	00cb8d93          	addi	s11,s7,12
   10cc8:	000d8593          	mv	a1,s11
   10ccc:	41f7d51b          	sraiw	a0,a5,0x1f
   10cd0:	00f547b3          	xor	a5,a0,a5
   10cd4:	40a7853b          	subw	a0,a5,a0
   10cd8:	b3dff0ef          	jal	ra,10814 <fmt_u>
   10cdc:	01b51a63          	bne	a0,s11,10cf0 <fmt_fp+0x37c>
   10ce0:	00813703          	ld	a4,8(sp)
   10ce4:	03000793          	li	a5,48
   10ce8:	00bb8513          	addi	a0,s7,11
   10cec:	30f705a3          	sb	a5,779(a4) # 230b <exit-0xde15>
   10cf0:	00002737          	lui	a4,0x2
   10cf4:	d1070713          	addi	a4,a4,-752 # 1d10 <exit-0xe410>
   10cf8:	04010693          	addi	a3,sp,64
   10cfc:	00d70733          	add	a4,a4,a3
   10d00:	ffffe7b7          	lui	a5,0xffffe
   10d04:	00f707b3          	add	a5,a4,a5
   10d08:	2fc7a703          	lw	a4,764(a5) # ffffffffffffe2fc <__global_pointer$+0xfffffffffffe5afc>
   10d0c:	00f13423          	sd	a5,8(sp)
   10d10:	02d00793          	li	a5,45
   10d14:	00074463          	bltz	a4,10d1c <fmt_fp+0x3a8>
   10d18:	02b00793          	li	a5,43
   10d1c:	00002737          	lui	a4,0x2
   10d20:	fef50fa3          	sb	a5,-1(a0)
   10d24:	d1070713          	addi	a4,a4,-752 # 1d10 <exit-0xe410>
   10d28:	ffffe7b7          	lui	a5,0xffffe
   10d2c:	04010693          	addi	a3,sp,64
   10d30:	31078793          	addi	a5,a5,784 # ffffffffffffe310 <__global_pointer$+0xfffffffffffe5b10>
   10d34:	00d70733          	add	a4,a4,a3
   10d38:	00f70db3          	add	s11,a4,a5
   10d3c:	00016737          	lui	a4,0x16
   10d40:	04070793          	addi	a5,a4,64 # 16040 <xdigits>
   10d44:	00f13423          	sd	a5,8(sp)
   10d48:	df8d3783          	ld	a5,-520(s10)
   10d4c:	00fb0b1b          	addiw	s6,s6,15
   10d50:	ffe50b93          	addi	s7,a0,-2
   10d54:	ff650f23          	sb	s6,-2(a0)
   10d58:	02f13423          	sd	a5,40(sp)
   10d5c:	03b13023          	sd	s11,32(sp)
   10d60:	008a7d13          	andi	s10,s4,8
   10d64:	000c0593          	mv	a1,s8
   10d68:	000c8513          	mv	a0,s9
   10d6c:	2c1040ef          	jal	ra,1582c <__fixtfsi>
   10d70:	00813783          	ld	a5,8(sp)
   10d74:	0005051b          	sext.w	a0,a0
   10d78:	001d8b13          	addi	s6,s11,1
   10d7c:	00a78733          	add	a4,a5,a0
   10d80:	00074703          	lbu	a4,0(a4)
   10d84:	01813783          	ld	a5,24(sp)
   10d88:	00e7e733          	or	a4,a5,a4
   10d8c:	00ed8023          	sb	a4,0(s11)
   10d90:	425040ef          	jal	ra,159b4 <__floatsitf>
   10d94:	00050613          	mv	a2,a0
   10d98:	00058693          	mv	a3,a1
   10d9c:	000c8513          	mv	a0,s9
   10da0:	000c0593          	mv	a1,s8
   10da4:	789030ef          	jal	ra,14d2c <__subtf3>
   10da8:	02813683          	ld	a3,40(sp)
   10dac:	00000613          	li	a2,0
   10db0:	704030ef          	jal	ra,144b4 <__multf3>
   10db4:	02013783          	ld	a5,32(sp)
   10db8:	00050813          	mv	a6,a0
   10dbc:	00058713          	mv	a4,a1
   10dc0:	40fb06b3          	sub	a3,s6,a5
   10dc4:	00100793          	li	a5,1
   10dc8:	00050c93          	mv	s9,a0
   10dcc:	00058c13          	mv	s8,a1
   10dd0:	02f69c63          	bne	a3,a5,10e08 <fmt_fp+0x494>
   10dd4:	00000613          	li	a2,0
   10dd8:	00000693          	li	a3,0
   10ddc:	02b13c23          	sd	a1,56(sp)
   10de0:	02a13823          	sd	a0,48(sp)
   10de4:	600030ef          	jal	ra,143e4 <__eqtf2>
   10de8:	000c8813          	mv	a6,s9
   10dec:	000c0713          	mv	a4,s8
   10df0:	00051663          	bnez	a0,10dfc <fmt_fp+0x488>
   10df4:	00804463          	bgtz	s0,10dfc <fmt_fp+0x488>
   10df8:	020d0463          	beqz	s10,10e20 <fmt_fp+0x4ac>
   10dfc:	02e00793          	li	a5,46
   10e00:	002d8b13          	addi	s6,s11,2
   10e04:	00fd80a3          	sb	a5,1(s11)
   10e08:	00000613          	li	a2,0
   10e0c:	00000693          	li	a3,0
   10e10:	00080513          	mv	a0,a6
   10e14:	00070593          	mv	a1,a4
   10e18:	5cc030ef          	jal	ra,143e4 <__eqtf2>
   10e1c:	0a051063          	bnez	a0,10ebc <fmt_fp+0x548>
   10e20:	000027b7          	lui	a5,0x2
   10e24:	04010693          	addi	a3,sp,64
   10e28:	d1078793          	addi	a5,a5,-752 # 1d10 <exit-0xe410>
   10e2c:	00d787b3          	add	a5,a5,a3
   10e30:	ffffe737          	lui	a4,0xffffe
   10e34:	00e78c33          	add	s8,a5,a4
   10e38:	00013683          	ld	a3,0(sp)
   10e3c:	30cc0c13          	addi	s8,s8,780
   10e40:	800007b7          	lui	a5,0x80000
   10e44:	417c0c33          	sub	s8,s8,s7
   10e48:	ffd7c793          	xori	a5,a5,-3
   10e4c:	418787b3          	sub	a5,a5,s8
   10e50:	40d787b3          	sub	a5,a5,a3
   10e54:	0687d863          	bge	a5,s0,10ec4 <fmt_fp+0x550>
   10e58:	fff00513          	li	a0,-1
   10e5c:	c79ff06f          	j	10ad4 <fmt_fp+0x160>
   10e60:	000c8513          	mv	a0,s9
   10e64:	000b8593          	mv	a1,s7
   10e68:	00000613          	li	a2,0
   10e6c:	00e13423          	sd	a4,8(sp)
   10e70:	644030ef          	jal	ra,144b4 <__multf3>
   10e74:	040037b7          	lui	a5,0x4003
   10e78:	00813703          	ld	a4,8(sp)
   10e7c:	00050c93          	mv	s9,a0
   10e80:	00058b93          	mv	s7,a1
   10e84:	fff00813          	li	a6,-1
   10e88:	02479693          	slli	a3,a5,0x24
   10e8c:	dc1ff06f          	j	10c4c <fmt_fp+0x2d8>
   10e90:	00070613          	mv	a2,a4
   10e94:	000d8693          	mv	a3,s11
   10e98:	000c8513          	mv	a0,s9
   10e9c:	000b8593          	mv	a1,s7
   10ea0:	24d020ef          	jal	ra,138ec <__addtf3>
   10ea4:	000c8613          	mv	a2,s9
   10ea8:	000b8693          	mv	a3,s7
   10eac:	681030ef          	jal	ra,14d2c <__subtf3>
   10eb0:	00050c93          	mv	s9,a0
   10eb4:	00058c13          	mv	s8,a1
   10eb8:	dddff06f          	j	10c94 <fmt_fp+0x320>
   10ebc:	000b0d93          	mv	s11,s6
   10ec0:	ea5ff06f          	j	10d64 <fmt_fp+0x3f0>
   10ec4:	000027b7          	lui	a5,0x2
   10ec8:	d1078793          	addi	a5,a5,-752 # 1d10 <exit-0xe410>
   10ecc:	04010693          	addi	a3,sp,64
   10ed0:	00d787b3          	add	a5,a5,a3
   10ed4:	31070713          	addi	a4,a4,784 # ffffffffffffe310 <__global_pointer$+0xfffffffffffe5b10>
   10ed8:	00e78733          	add	a4,a5,a4
   10edc:	40eb0b33          	sub	s6,s6,a4
   10ee0:	000c079b          	sext.w	a5,s8
   10ee4:	016c0dbb          	addw	s11,s8,s6
   10ee8:	0e040263          	beqz	s0,10fcc <fmt_fp+0x658>
   10eec:	fffb0713          	addi	a4,s6,-1
   10ef0:	0ce44e63          	blt	s0,a4,10fcc <fmt_fp+0x658>
   10ef4:	01013703          	ld	a4,16(sp)
   10ef8:	0027041b          	addiw	s0,a4,2
   10efc:	00f4043b          	addw	s0,s0,a5
   10f00:	01540cbb          	addw	s9,s0,s5
   10f04:	000a0713          	mv	a4,s4
   10f08:	000c8693          	mv	a3,s9
   10f0c:	00098613          	mv	a2,s3
   10f10:	02000593          	li	a1,32
   10f14:	00048513          	mv	a0,s1
   10f18:	9b5ff0ef          	jal	ra,108cc <pad>
   10f1c:	00013603          	ld	a2,0(sp)
   10f20:	00090593          	mv	a1,s2
   10f24:	00048513          	mv	a0,s1
   10f28:	981ff0ef          	jal	ra,108a8 <out>
   10f2c:	00010737          	lui	a4,0x10
   10f30:	000c8693          	mv	a3,s9
   10f34:	00ea4733          	xor	a4,s4,a4
   10f38:	00098613          	mv	a2,s3
   10f3c:	03000593          	li	a1,48
   10f40:	00048513          	mv	a0,s1
   10f44:	989ff0ef          	jal	ra,108cc <pad>
   10f48:	000027b7          	lui	a5,0x2
   10f4c:	04010713          	addi	a4,sp,64
   10f50:	ffffe5b7          	lui	a1,0xffffe
   10f54:	d1078793          	addi	a5,a5,-752 # 1d10 <exit-0xe410>
   10f58:	00e787b3          	add	a5,a5,a4
   10f5c:	31058593          	addi	a1,a1,784 # ffffffffffffe310 <__global_pointer$+0xfffffffffffe5b10>
   10f60:	00b785b3          	add	a1,a5,a1
   10f64:	000b0613          	mv	a2,s6
   10f68:	00048513          	mv	a0,s1
   10f6c:	93dff0ef          	jal	ra,108a8 <out>
   10f70:	00000713          	li	a4,0
   10f74:	00000693          	li	a3,0
   10f78:	41b4063b          	subw	a2,s0,s11
   10f7c:	03000593          	li	a1,48
   10f80:	00048513          	mv	a0,s1
   10f84:	949ff0ef          	jal	ra,108cc <pad>
   10f88:	000c0613          	mv	a2,s8
   10f8c:	000b8593          	mv	a1,s7
   10f90:	00048513          	mv	a0,s1
   10f94:	915ff0ef          	jal	ra,108a8 <out>
   10f98:	000a091b          	sext.w	s2,s4
   10f9c:	00002737          	lui	a4,0x2
   10fa0:	00e94733          	xor	a4,s2,a4
   10fa4:	000c8693          	mv	a3,s9
   10fa8:	00098613          	mv	a2,s3
   10fac:	02000593          	li	a1,32
   10fb0:	00048513          	mv	a0,s1
   10fb4:	000c8a93          	mv	s5,s9
   10fb8:	915ff0ef          	jal	ra,108cc <pad>
   10fbc:	013cd463          	bge	s9,s3,10fc4 <fmt_fp+0x650>
   10fc0:	00098a93          	mv	s5,s3
   10fc4:	000a851b          	sext.w	a0,s5
   10fc8:	b0dff06f          	j	10ad4 <fmt_fp+0x160>
   10fcc:	000d841b          	sext.w	s0,s11
   10fd0:	f31ff06f          	j	10f00 <fmt_fp+0x58c>
   10fd4:	00045463          	bgez	s0,10fdc <fmt_fp+0x668>
   10fd8:	00600413          	li	s0,6
   10fdc:	00000613          	li	a2,0
   10fe0:	00000693          	li	a3,0
   10fe4:	00070513          	mv	a0,a4
   10fe8:	000d8593          	mv	a1,s11
   10fec:	00e13823          	sd	a4,16(sp)
   10ff0:	3f4030ef          	jal	ra,143e4 <__eqtf2>
   10ff4:	04050a63          	beqz	a0,11048 <fmt_fp+0x6d4>
   10ff8:	01013703          	ld	a4,16(sp)
   10ffc:	000166b7          	lui	a3,0x16
   11000:	e586b683          	ld	a3,-424(a3) # 15e58 <__clzdi2+0x29c>
   11004:	00070513          	mv	a0,a4
   11008:	00000613          	li	a2,0
   1100c:	000d8593          	mv	a1,s11
   11010:	4a4030ef          	jal	ra,144b4 <__multf3>
   11014:	00002737          	lui	a4,0x2
   11018:	d1070713          	addi	a4,a4,-752 # 1d10 <exit-0xe410>
   1101c:	04010693          	addi	a3,sp,64
   11020:	00d70733          	add	a4,a4,a3
   11024:	ffffe7b7          	lui	a5,0xffffe
   11028:	00f707b3          	add	a5,a4,a5
   1102c:	00f13823          	sd	a5,16(sp)
   11030:	2fc7a783          	lw	a5,764(a5) # ffffffffffffe2fc <__global_pointer$+0xfffffffffffe5afc>
   11034:	01013703          	ld	a4,16(sp)
   11038:	00050c93          	mv	s9,a0
   1103c:	fe47879b          	addiw	a5,a5,-28
   11040:	00058c13          	mv	s8,a1
   11044:	2ef72e23          	sw	a5,764(a4)
   11048:	04c12303          	lw	t1,76(sp)
   1104c:	00002737          	lui	a4,0x2
   11050:	b4870693          	addi	a3,a4,-1208 # 1b48 <exit-0xe5d8>
   11054:	04010613          	addi	a2,sp,64
   11058:	ffffe7b7          	lui	a5,0xffffe
   1105c:	00d608b3          	add	a7,a2,a3
   11060:	00035a63          	bgez	t1,11074 <fmt_fp+0x700>
   11064:	d1070713          	addi	a4,a4,-752
   11068:	33878793          	addi	a5,a5,824 # ffffffffffffe338 <__global_pointer$+0xfffffffffffe5b38>
   1106c:	00c70733          	add	a4,a4,a2
   11070:	00f708b3          	add	a7,a4,a5
   11074:	00016737          	lui	a4,0x16
   11078:	e6873d83          	ld	s11,-408(a4) # 15e68 <__clzdi2+0x2ac>
   1107c:	00088d13          	mv	s10,a7
   11080:	000c0593          	mv	a1,s8
   11084:	000c8513          	mv	a0,s9
   11088:	01113c23          	sd	a7,24(sp)
   1108c:	00613823          	sd	t1,16(sp)
   11090:	071040ef          	jal	ra,15900 <__fixunstfsi>
   11094:	00ad2023          	sw	a0,0(s10)
   11098:	0005051b          	sext.w	a0,a0
   1109c:	1a1040ef          	jal	ra,15a3c <__floatunsitf>
   110a0:	00050613          	mv	a2,a0
   110a4:	00058693          	mv	a3,a1
   110a8:	000c8513          	mv	a0,s9
   110ac:	000c0593          	mv	a1,s8
   110b0:	47d030ef          	jal	ra,14d2c <__subtf3>
   110b4:	00000613          	li	a2,0
   110b8:	000d8693          	mv	a3,s11
   110bc:	3f8030ef          	jal	ra,144b4 <__multf3>
   110c0:	00000613          	li	a2,0
   110c4:	00000693          	li	a3,0
   110c8:	00050c93          	mv	s9,a0
   110cc:	00058c13          	mv	s8,a1
   110d0:	314030ef          	jal	ra,143e4 <__eqtf2>
   110d4:	01013303          	ld	t1,16(sp)
   110d8:	01813883          	ld	a7,24(sp)
   110dc:	004d0d13          	addi	s10,s10,4
   110e0:	fa0510e3          	bnez	a0,11080 <fmt_fp+0x70c>
   110e4:	3b9ad537          	lui	a0,0x3b9ad
   110e8:	00088b93          	mv	s7,a7
   110ec:	00000713          	li	a4,0
   110f0:	01d00e13          	li	t3,29
   110f4:	a0050513          	addi	a0,a0,-1536 # 3b9aca00 <__global_pointer$+0x3b994200>
   110f8:	1a604c63          	bgtz	t1,112b0 <fmt_fp+0x93c>
   110fc:	02070263          	beqz	a4,11120 <fmt_fp+0x7ac>
   11100:	000027b7          	lui	a5,0x2
   11104:	d1078793          	addi	a5,a5,-752 # 1d10 <exit-0xe410>
   11108:	04010693          	addi	a3,sp,64
   1110c:	ffffe737          	lui	a4,0xffffe
   11110:	00d787b3          	add	a5,a5,a3
   11114:	00e787b3          	add	a5,a5,a4
   11118:	00f13823          	sd	a5,16(sp)
   1111c:	2e67ae23          	sw	t1,764(a5)
   11120:	00900693          	li	a3,9
   11124:	02d4071b          	addiw	a4,s0,45
   11128:	02d7573b          	divuw	a4,a4,a3
   1112c:	000027b7          	lui	a5,0x2
   11130:	04010613          	addi	a2,sp,64
   11134:	d1078793          	addi	a5,a5,-752 # 1d10 <exit-0xe410>
   11138:	ffffe6b7          	lui	a3,0xffffe
   1113c:	00c787b3          	add	a5,a5,a2
   11140:	00d787b3          	add	a5,a5,a3
   11144:	2fc7a683          	lw	a3,764(a5)
   11148:	3b9ade37          	lui	t3,0x3b9ad
   1114c:	00f13823          	sd	a5,16(sp)
   11150:	00000613          	li	a2,0
   11154:	ff700f13          	li	t5,-9
   11158:	00100f93          	li	t6,1
   1115c:	a00e0e1b          	addiw	t3,t3,-1536
   11160:	06600293          	li	t0,102
   11164:	0017071b          	addiw	a4,a4,1
   11168:	02071713          	slli	a4,a4,0x20
   1116c:	02075713          	srli	a4,a4,0x20
   11170:	00271e93          	slli	t4,a4,0x2
   11174:	1a06c663          	bltz	a3,11320 <fmt_fp+0x9ac>
   11178:	02060263          	beqz	a2,1119c <fmt_fp+0x828>
   1117c:	000027b7          	lui	a5,0x2
   11180:	d1078793          	addi	a5,a5,-752 # 1d10 <exit-0xe410>
   11184:	04010613          	addi	a2,sp,64
   11188:	ffffe737          	lui	a4,0xffffe
   1118c:	00c787b3          	add	a5,a5,a2
   11190:	00e787b3          	add	a5,a5,a4
   11194:	00f13823          	sd	a5,16(sp)
   11198:	2ed7ae23          	sw	a3,764(a5)
   1119c:	00000313          	li	t1,0
   111a0:	03abf263          	bgeu	s7,s10,111c4 <fmt_fp+0x850>
   111a4:	41788333          	sub	t1,a7,s7
   111a8:	000ba683          	lw	a3,0(s7)
   111ac:	40235713          	srai	a4,t1,0x2
   111b0:	00900313          	li	t1,9
   111b4:	02e3033b          	mulw	t1,t1,a4
   111b8:	00a00613          	li	a2,10
   111bc:	00a00713          	li	a4,10
   111c0:	1ee6f463          	bgeu	a3,a4,113a8 <fmt_fp+0xa34>
   111c4:	00013783          	ld	a5,0(sp)
   111c8:	06700613          	li	a2,103
   111cc:	00000693          	li	a3,0
   111d0:	f9a78713          	addi	a4,a5,-102
   111d4:	00e03733          	snez	a4,a4
   111d8:	0267073b          	mulw	a4,a4,t1
   111dc:	40e4073b          	subw	a4,s0,a4
   111e0:	00c79463          	bne	a5,a2,111e8 <fmt_fp+0x874>
   111e4:	008036b3          	snez	a3,s0
   111e8:	40d7063b          	subw	a2,a4,a3
   111ec:	411d0733          	sub	a4,s10,a7
   111f0:	40275713          	srai	a4,a4,0x2
   111f4:	fff70713          	addi	a4,a4,-1 # ffffffffffffdfff <__global_pointer$+0xfffffffffffe57ff>
   111f8:	00900593          	li	a1,9
   111fc:	02b70733          	mul	a4,a4,a1
   11200:	24e65663          	bge	a2,a4,1144c <fmt_fp+0xad8>
   11204:	00024737          	lui	a4,0x24
   11208:	00c7073b          	addw	a4,a4,a2
   1120c:	00900613          	li	a2,9
   11210:	02c74c3b          	divw	s8,a4,a2
   11214:	ffff06b7          	lui	a3,0xffff0
   11218:	00468693          	addi	a3,a3,4 # ffffffffffff0004 <__global_pointer$+0xfffffffffffd7804>
   1121c:	00a00593          	li	a1,10
   11220:	02c7673b          	remw	a4,a4,a2
   11224:	002c1c13          	slli	s8,s8,0x2
   11228:	00dc0c33          	add	s8,s8,a3
   1122c:	01888c33          	add	s8,a7,s8
   11230:	00a00693          	li	a3,10
   11234:	00900613          	li	a2,9
   11238:	0017071b          	addiw	a4,a4,1
   1123c:	16c71c63          	bne	a4,a2,113b4 <fmt_fp+0xa40>
   11240:	000c2d83          	lw	s11,0(s8)
   11244:	00068e1b          	sext.w	t3,a3
   11248:	02ddf63b          	remuw	a2,s11,a3
   1124c:	00060593          	mv	a1,a2
   11250:	00061663          	bnez	a2,1125c <fmt_fp+0x8e8>
   11254:	004c0713          	addi	a4,s8,4
   11258:	1eed0463          	beq	s10,a4,11440 <fmt_fp+0xacc>
   1125c:	03cdd73b          	divuw	a4,s11,t3
   11260:	00177713          	andi	a4,a4,1
   11264:	02071063          	bnez	a4,11284 <fmt_fp+0x910>
   11268:	3b9ad737          	lui	a4,0x3b9ad
   1126c:	a0070713          	addi	a4,a4,-1536 # 3b9aca00 <__global_pointer$+0x3b994200>
   11270:	14e69863          	bne	a3,a4,113c0 <fmt_fp+0xa4c>
   11274:	158bf663          	bgeu	s7,s8,113c0 <fmt_fp+0xa4c>
   11278:	ffcc2703          	lw	a4,-4(s8)
   1127c:	00177713          	andi	a4,a4,1
   11280:	14070063          	beqz	a4,113c0 <fmt_fp+0xa4c>
   11284:	00016537          	lui	a0,0x16
   11288:	e1853c83          	ld	s9,-488(a0) # 15e18 <__clzdi2+0x25c>
   1128c:	00100713          	li	a4,1
   11290:	4016d693          	srai	a3,a3,0x1
   11294:	12d66e63          	bltu	a2,a3,113d0 <fmt_fp+0xa5c>
   11298:	00c69663          	bne	a3,a2,112a4 <fmt_fp+0x930>
   1129c:	004c0693          	addi	a3,s8,4
   112a0:	38dd0e63          	beq	s10,a3,1163c <fmt_fp+0xcc8>
   112a4:	000166b7          	lui	a3,0x16
   112a8:	e286b683          	ld	a3,-472(a3) # 15e28 <__clzdi2+0x26c>
   112ac:	12c0006f          	j	113d8 <fmt_fp+0xa64>
   112b0:	0003061b          	sext.w	a2,t1
   112b4:	006e5463          	bge	t3,t1,112bc <fmt_fp+0x948>
   112b8:	01d00613          	li	a2,29
   112bc:	ffcd0593          	addi	a1,s10,-4
   112c0:	00000713          	li	a4,0
   112c4:	0375f463          	bgeu	a1,s7,112ec <fmt_fp+0x978>
   112c8:	00070663          	beqz	a4,112d4 <fmt_fp+0x960>
   112cc:	feebae23          	sw	a4,-4(s7)
   112d0:	ffcb8b93          	addi	s7,s7,-4
   112d4:	01abf663          	bgeu	s7,s10,112e0 <fmt_fp+0x96c>
   112d8:	ffcd2703          	lw	a4,-4(s10)
   112dc:	02070e63          	beqz	a4,11318 <fmt_fp+0x9a4>
   112e0:	40c3033b          	subw	t1,t1,a2
   112e4:	00100713          	li	a4,1
   112e8:	e11ff06f          	j	110f8 <fmt_fp+0x784>
   112ec:	0005e683          	lwu	a3,0(a1)
   112f0:	02071713          	slli	a4,a4,0x20
   112f4:	02075713          	srli	a4,a4,0x20
   112f8:	00c696b3          	sll	a3,a3,a2
   112fc:	00e68733          	add	a4,a3,a4
   11300:	02a776b3          	remu	a3,a4,a0
   11304:	ffc58593          	addi	a1,a1,-4
   11308:	02a75733          	divu	a4,a4,a0
   1130c:	00d5a223          	sw	a3,4(a1)
   11310:	0007071b          	sext.w	a4,a4
   11314:	fb1ff06f          	j	112c4 <fmt_fp+0x950>
   11318:	ffcd0d13          	addi	s10,s10,-4
   1131c:	fb9ff06f          	j	112d4 <fmt_fp+0x960>
   11320:	00900593          	li	a1,9
   11324:	01e6c463          	blt	a3,t5,1132c <fmt_fp+0x9b8>
   11328:	40d005bb          	negw	a1,a3
   1132c:	00bf93bb          	sllw	t2,t6,a1
   11330:	fff3839b          	addiw	t2,t2,-1
   11334:	40be5c3b          	sraw	s8,t3,a1
   11338:	000b8613          	mv	a2,s7
   1133c:	00000513          	li	a0,0
   11340:	05a66463          	bltu	a2,s10,11388 <fmt_fp+0xa14>
   11344:	000ba603          	lw	a2,0(s7)
   11348:	00061463          	bnez	a2,11350 <fmt_fp+0x9dc>
   1134c:	004b8b93          	addi	s7,s7,4
   11350:	00050663          	beqz	a0,1135c <fmt_fp+0x9e8>
   11354:	00ad2023          	sw	a0,0(s10)
   11358:	004d0d13          	addi	s10,s10,4
   1135c:	00013783          	ld	a5,0(sp)
   11360:	00088613          	mv	a2,a7
   11364:	00578463          	beq	a5,t0,1136c <fmt_fp+0x9f8>
   11368:	000b8613          	mv	a2,s7
   1136c:	40cd0533          	sub	a0,s10,a2
   11370:	40255513          	srai	a0,a0,0x2
   11374:	00a75463          	bge	a4,a0,1137c <fmt_fp+0xa08>
   11378:	01d60d33          	add	s10,a2,t4
   1137c:	00b686bb          	addw	a3,a3,a1
   11380:	00100613          	li	a2,1
   11384:	df1ff06f          	j	11174 <fmt_fp+0x800>
   11388:	00062303          	lw	t1,0(a2)
   1138c:	00460613          	addi	a2,a2,4
   11390:	00b35cbb          	srlw	s9,t1,a1
   11394:	00ac853b          	addw	a0,s9,a0
   11398:	fea62e23          	sw	a0,-4(a2)
   1139c:	00737533          	and	a0,t1,t2
   113a0:	0385053b          	mulw	a0,a0,s8
   113a4:	f9dff06f          	j	11340 <fmt_fp+0x9cc>
   113a8:	02e6073b          	mulw	a4,a2,a4
   113ac:	0013031b          	addiw	t1,t1,1
   113b0:	e11ff06f          	j	111c0 <fmt_fp+0x84c>
   113b4:	02d586bb          	mulw	a3,a1,a3
   113b8:	0017071b          	addiw	a4,a4,1
   113bc:	e81ff06f          	j	1123c <fmt_fp+0x8c8>
   113c0:	00016537          	lui	a0,0x16
   113c4:	e0853c83          	ld	s9,-504(a0) # 15e08 <__clzdi2+0x24c>
   113c8:	00000713          	li	a4,0
   113cc:	ec5ff06f          	j	11290 <fmt_fp+0x91c>
   113d0:	000166b7          	lui	a3,0x16
   113d4:	e486b683          	ld	a3,-440(a3) # 15e48 <__clzdi2+0x28c>
   113d8:	020a8063          	beqz	s5,113f8 <fmt_fp+0xa84>
   113dc:	00094503          	lbu	a0,0(s2)
   113e0:	02d00613          	li	a2,45
   113e4:	00c51a63          	bne	a0,a2,113f8 <fmt_fp+0xa84>
   113e8:	fff00613          	li	a2,-1
   113ec:	03f61613          	slli	a2,a2,0x3f
   113f0:	00ccccb3          	xor	s9,s9,a2
   113f4:	00c6c6b3          	xor	a3,a3,a2
   113f8:	00000613          	li	a2,0
   113fc:	00070513          	mv	a0,a4
   11400:	40bd8dbb          	subw	s11,s11,a1
   11404:	000c8593          	mv	a1,s9
   11408:	03113423          	sd	a7,40(sp)
   1140c:	01c13c23          	sd	t3,24(sp)
   11410:	02613023          	sd	t1,32(sp)
   11414:	00e13823          	sd	a4,16(sp)
   11418:	4d4020ef          	jal	ra,138ec <__addtf3>
   1141c:	01013703          	ld	a4,16(sp)
   11420:	000c8693          	mv	a3,s9
   11424:	00070613          	mv	a2,a4
   11428:	7bd020ef          	jal	ra,143e4 <__eqtf2>
   1142c:	01813e03          	ld	t3,24(sp)
   11430:	02813883          	ld	a7,40(sp)
   11434:	20051a63          	bnez	a0,11648 <fmt_fp+0xcd4>
   11438:	02013303          	ld	t1,32(sp)
   1143c:	01bc2023          	sw	s11,0(s8)
   11440:	004c0c13          	addi	s8,s8,4
   11444:	01ac7463          	bgeu	s8,s10,1144c <fmt_fp+0xad8>
   11448:	000c0d13          	mv	s10,s8
   1144c:	01abf663          	bgeu	s7,s10,11458 <fmt_fp+0xae4>
   11450:	ffcd2703          	lw	a4,-4(s10)
   11454:	24070e63          	beqz	a4,116b0 <fmt_fp+0xd3c>
   11458:	00013783          	ld	a5,0(sp)
   1145c:	06700713          	li	a4,103
   11460:	06e79a63          	bne	a5,a4,114d4 <fmt_fp+0xb60>
   11464:	00041463          	bnez	s0,1146c <fmt_fp+0xaf8>
   11468:	00100413          	li	s0,1
   1146c:	24835663          	bge	t1,s0,116b8 <fmt_fp+0xd44>
   11470:	ffc00713          	li	a4,-4
   11474:	24e34263          	blt	t1,a4,116b8 <fmt_fp+0xd44>
   11478:	0013071b          	addiw	a4,t1,1
   1147c:	fffb0b1b          	addiw	s6,s6,-1
   11480:	40e4043b          	subw	s0,s0,a4
   11484:	008a7713          	andi	a4,s4,8
   11488:	04071663          	bnez	a4,114d4 <fmt_fp+0xb60>
   1148c:	00900693          	li	a3,9
   11490:	01abf663          	bgeu	s7,s10,1149c <fmt_fp+0xb28>
   11494:	ffcd2603          	lw	a2,-4(s10)
   11498:	24061063          	bnez	a2,116d8 <fmt_fp+0xd64>
   1149c:	411d0733          	sub	a4,s10,a7
   114a0:	40275713          	srai	a4,a4,0x2
   114a4:	00900613          	li	a2,9
   114a8:	fff70713          	addi	a4,a4,-1
   114ac:	02c70733          	mul	a4,a4,a2
   114b0:	06600593          	li	a1,102
   114b4:	020b6613          	ori	a2,s6,32
   114b8:	22b61863          	bne	a2,a1,116e8 <fmt_fp+0xd74>
   114bc:	40d70733          	sub	a4,a4,a3
   114c0:	00075463          	bgez	a4,114c8 <fmt_fp+0xb54>
   114c4:	00000713          	li	a4,0
   114c8:	00e45463          	bge	s0,a4,114d0 <fmt_fp+0xb5c>
   114cc:	00040713          	mv	a4,s0
   114d0:	0007041b          	sext.w	s0,a4
   114d4:	80000737          	lui	a4,0x80000
   114d8:	ffd74713          	xori	a4,a4,-3
   114dc:	96874ee3          	blt	a4,s0,10e58 <fmt_fp+0x4e4>
   114e0:	008a7c93          	andi	s9,s4,8
   114e4:	01946c33          	or	s8,s0,s9
   114e8:	018036b3          	snez	a3,s8
   114ec:	0014071b          	addiw	a4,s0,1
   114f0:	00e6853b          	addw	a0,a3,a4
   114f4:	000a079b          	sext.w	a5,s4
   114f8:	800006b7          	lui	a3,0x80000
   114fc:	fff6c693          	not	a3,a3
   11500:	00f13023          	sd	a5,0(sp)
   11504:	020b6d93          	ori	s11,s6,32
   11508:	06600613          	li	a2,102
   1150c:	40a686bb          	subw	a3,a3,a0
   11510:	1ecd9063          	bne	s11,a2,116f0 <fmt_fp+0xd7c>
   11514:	9466c2e3          	blt	a3,t1,10e58 <fmt_fp+0x4e4>
   11518:	00605463          	blez	t1,11520 <fmt_fp+0xbac>
   1151c:	0065053b          	addw	a0,a0,t1
   11520:	80000737          	lui	a4,0x80000
   11524:	280a9663          	bnez	s5,117b0 <fmt_fp+0xe3c>
   11528:	fff74713          	not	a4,a4
   1152c:	01113c23          	sd	a7,24(sp)
   11530:	0007071b          	sext.w	a4,a4
   11534:	92a742e3          	blt	a4,a0,10e58 <fmt_fp+0x4e4>
   11538:	01550b3b          	addw	s6,a0,s5
   1153c:	000a0713          	mv	a4,s4
   11540:	000b0693          	mv	a3,s6
   11544:	00098613          	mv	a2,s3
   11548:	02000593          	li	a1,32
   1154c:	00048513          	mv	a0,s1
   11550:	01612823          	sw	s6,16(sp)
   11554:	b78ff0ef          	jal	ra,108cc <pad>
   11558:	000a8613          	mv	a2,s5
   1155c:	00090593          	mv	a1,s2
   11560:	00048513          	mv	a0,s1
   11564:	b44ff0ef          	jal	ra,108a8 <out>
   11568:	00013783          	ld	a5,0(sp)
   1156c:	00010737          	lui	a4,0x10
   11570:	000b0693          	mv	a3,s6
   11574:	00e7c733          	xor	a4,a5,a4
   11578:	00098613          	mv	a2,s3
   1157c:	03000593          	li	a1,48
   11580:	00048513          	mv	a0,s1
   11584:	b48ff0ef          	jal	ra,108cc <pad>
   11588:	06600713          	li	a4,102
   1158c:	01813883          	ld	a7,24(sp)
   11590:	32ed9c63          	bne	s11,a4,118c8 <fmt_fp+0xf54>
   11594:	00088913          	mv	s2,a7
   11598:	011bf463          	bgeu	s7,a7,115a0 <fmt_fp+0xc2c>
   1159c:	000b8913          	mv	s2,s7
   115a0:	00002737          	lui	a4,0x2
   115a4:	ffffea37          	lui	s4,0xffffe
   115a8:	04010693          	addi	a3,sp,64
   115ac:	d1070793          	addi	a5,a4,-752 # 1d10 <exit-0xe410>
   115b0:	310a0b93          	addi	s7,s4,784 # ffffffffffffe310 <__global_pointer$+0xfffffffffffe5b10>
   115b4:	00d787b3          	add	a5,a5,a3
   115b8:	01778bb3          	add	s7,a5,s7
   115bc:	d1070793          	addi	a5,a4,-752
   115c0:	00d787b3          	add	a5,a5,a3
   115c4:	00090c93          	mv	s9,s2
   115c8:	009b8d93          	addi	s11,s7,9
   115cc:	03000a93          	li	s5,48
   115d0:	01478a33          	add	s4,a5,s4
   115d4:	1f98f263          	bgeu	a7,s9,117b8 <fmt_fp+0xe44>
   115d8:	00188693          	addi	a3,a7,1
   115dc:	ffd90713          	addi	a4,s2,-3
   115e0:	00000613          	li	a2,0
   115e4:	00e6e863          	bltu	a3,a4,115f4 <fmt_fp+0xc80>
   115e8:	00488893          	addi	a7,a7,4
   115ec:	412888b3          	sub	a7,a7,s2
   115f0:	ffc8f613          	andi	a2,a7,-4
   115f4:	00c90933          	add	s2,s2,a2
   115f8:	2c0c0463          	beqz	s8,118c0 <fmt_fp+0xf4c>
   115fc:	000165b7          	lui	a1,0x16
   11600:	00100613          	li	a2,1
   11604:	cd058593          	addi	a1,a1,-816 # 15cd0 <__clzdi2+0x114>
   11608:	00048513          	mv	a0,s1
   1160c:	a9cff0ef          	jal	ra,108a8 <out>
   11610:	000027b7          	lui	a5,0x2
   11614:	ffffea37          	lui	s4,0xffffe
   11618:	d1078793          	addi	a5,a5,-752 # 1d10 <exit-0xe410>
   1161c:	04010713          	addi	a4,sp,64
   11620:	310a0a13          	addi	s4,s4,784 # ffffffffffffe310 <__global_pointer$+0xfffffffffffe5b10>
   11624:	00e787b3          	add	a5,a5,a4
   11628:	01478a33          	add	s4,a5,s4
   1162c:	009a0a93          	addi	s5,s4,9
   11630:	03000b93          	li	s7,48
   11634:	00900c93          	li	s9,9
   11638:	2280006f          	j	11860 <fmt_fp+0xeec>
   1163c:	000166b7          	lui	a3,0x16
   11640:	e386b683          	ld	a3,-456(a3) # 15e38 <__clzdi2+0x27c>
   11644:	d95ff06f          	j	113d8 <fmt_fp+0xa64>
   11648:	01be0e3b          	addw	t3,t3,s11
   1164c:	3b9ad737          	lui	a4,0x3b9ad
   11650:	01cc2023          	sw	t3,0(s8)
   11654:	9ff70713          	addi	a4,a4,-1537 # 3b9ac9ff <__global_pointer$+0x3b9941ff>
   11658:	000c2683          	lw	a3,0(s8)
   1165c:	02d76863          	bltu	a4,a3,1168c <fmt_fp+0xd18>
   11660:	41788333          	sub	t1,a7,s7
   11664:	000ba683          	lw	a3,0(s7)
   11668:	40235713          	srai	a4,t1,0x2
   1166c:	00900313          	li	t1,9
   11670:	02e3033b          	mulw	t1,t1,a4
   11674:	00a00613          	li	a2,10
   11678:	00a00713          	li	a4,10
   1167c:	dce6e2e3          	bltu	a3,a4,11440 <fmt_fp+0xacc>
   11680:	0013031b          	addiw	t1,t1,1
   11684:	02e6073b          	mulw	a4,a2,a4
   11688:	ff5ff06f          	j	1167c <fmt_fp+0xd08>
   1168c:	ffcc0c13          	addi	s8,s8,-4
   11690:	000c2223          	sw	zero,4(s8)
   11694:	017c7663          	bgeu	s8,s7,116a0 <fmt_fp+0xd2c>
   11698:	fe0bae23          	sw	zero,-4(s7)
   1169c:	ffcb8b93          	addi	s7,s7,-4
   116a0:	000c2683          	lw	a3,0(s8)
   116a4:	0016869b          	addiw	a3,a3,1
   116a8:	00dc2023          	sw	a3,0(s8)
   116ac:	fadff06f          	j	11658 <fmt_fp+0xce4>
   116b0:	ffcd0d13          	addi	s10,s10,-4
   116b4:	d99ff06f          	j	1144c <fmt_fp+0xad8>
   116b8:	ffeb0b1b          	addiw	s6,s6,-2
   116bc:	fff4041b          	addiw	s0,s0,-1
   116c0:	dc5ff06f          	j	11484 <fmt_fp+0xb10>
   116c4:	02e5073b          	mulw	a4,a0,a4
   116c8:	0016869b          	addiw	a3,a3,1
   116cc:	02e675bb          	remuw	a1,a2,a4
   116d0:	fe058ae3          	beqz	a1,116c4 <fmt_fp+0xd50>
   116d4:	dc9ff06f          	j	1149c <fmt_fp+0xb28>
   116d8:	00000693          	li	a3,0
   116dc:	00a00713          	li	a4,10
   116e0:	00a00513          	li	a0,10
   116e4:	fe9ff06f          	j	116cc <fmt_fp+0xd58>
   116e8:	00e30733          	add	a4,t1,a4
   116ec:	dd1ff06f          	j	114bc <fmt_fp+0xb48>
   116f0:	00002737          	lui	a4,0x2
   116f4:	02d13023          	sd	a3,32(sp)
   116f8:	d1070713          	addi	a4,a4,-752 # 1d10 <exit-0xe410>
   116fc:	04010693          	addi	a3,sp,64
   11700:	00d70733          	add	a4,a4,a3
   11704:	ffffe7b7          	lui	a5,0xffffe
   11708:	41f3561b          	sraiw	a2,t1,0x1f
   1170c:	00f707b3          	add	a5,a4,a5
   11710:	30c78793          	addi	a5,a5,780 # ffffffffffffe30c <__global_pointer$+0xfffffffffffe5b0c>
   11714:	02a12423          	sw	a0,40(sp)
   11718:	00c34533          	xor	a0,t1,a2
   1171c:	00078593          	mv	a1,a5
   11720:	40c5053b          	subw	a0,a0,a2
   11724:	01113c23          	sd	a7,24(sp)
   11728:	00613823          	sd	t1,16(sp)
   1172c:	00f13423          	sd	a5,8(sp)
   11730:	8e4ff0ef          	jal	ra,10814 <fmt_u>
   11734:	00813783          	ld	a5,8(sp)
   11738:	01013303          	ld	t1,16(sp)
   1173c:	01813883          	ld	a7,24(sp)
   11740:	02013683          	ld	a3,32(sp)
   11744:	02812703          	lw	a4,40(sp)
   11748:	00100613          	li	a2,1
   1174c:	03000e13          	li	t3,48
   11750:	40a785b3          	sub	a1,a5,a0
   11754:	04b65863          	bge	a2,a1,117a4 <fmt_fp+0xe30>
   11758:	02d00613          	li	a2,45
   1175c:	00034463          	bltz	t1,11764 <fmt_fp+0xdf0>
   11760:	02b00613          	li	a2,43
   11764:	ffe50793          	addi	a5,a0,-2
   11768:	00f13423          	sd	a5,8(sp)
   1176c:	000027b7          	lui	a5,0x2
   11770:	fec50fa3          	sb	a2,-1(a0)
   11774:	d1078793          	addi	a5,a5,-752 # 1d10 <exit-0xe410>
   11778:	04010613          	addi	a2,sp,64
   1177c:	00c787b3          	add	a5,a5,a2
   11780:	ff650f23          	sb	s6,-2(a0)
   11784:	ffffe537          	lui	a0,0xffffe
   11788:	00a78533          	add	a0,a5,a0
   1178c:	00813783          	ld	a5,8(sp)
   11790:	30c50513          	addi	a0,a0,780 # ffffffffffffe30c <__global_pointer$+0xfffffffffffe5b0c>
   11794:	40f50533          	sub	a0,a0,a5
   11798:	eca6c063          	blt	a3,a0,10e58 <fmt_fp+0x4e4>
   1179c:	00a7053b          	addw	a0,a4,a0
   117a0:	d81ff06f          	j	11520 <fmt_fp+0xbac>
   117a4:	fff50513          	addi	a0,a0,-1
   117a8:	01c50023          	sb	t3,0(a0)
   117ac:	fa5ff06f          	j	11750 <fmt_fp+0xddc>
   117b0:	ffe74713          	xori	a4,a4,-2
   117b4:	d79ff06f          	j	1152c <fmt_fp+0xbb8>
   117b8:	000ce503          	lwu	a0,0(s9)
   117bc:	000d8593          	mv	a1,s11
   117c0:	01113423          	sd	a7,8(sp)
   117c4:	850ff0ef          	jal	ra,10814 <fmt_u>
   117c8:	00813883          	ld	a7,8(sp)
   117cc:	00050593          	mv	a1,a0
   117d0:	00050713          	mv	a4,a0
   117d4:	012c9e63          	bne	s9,s2,117f0 <fmt_fp+0xe7c>
   117d8:	03b51663          	bne	a0,s11,11804 <fmt_fp+0xe90>
   117dc:	315a0c23          	sb	s5,792(s4)
   117e0:	008b8593          	addi	a1,s7,8
   117e4:	0200006f          	j	11804 <fmt_fp+0xe90>
   117e8:	fff70713          	addi	a4,a4,-1
   117ec:	01570023          	sb	s5,0(a4)
   117f0:	feebece3          	bltu	s7,a4,117e8 <fmt_fp+0xe74>
   117f4:	00000713          	li	a4,0
   117f8:	0175e463          	bltu	a1,s7,11800 <fmt_fp+0xe8c>
   117fc:	40bb8733          	sub	a4,s7,a1
   11800:	00e585b3          	add	a1,a1,a4
   11804:	40bd8633          	sub	a2,s11,a1
   11808:	00048513          	mv	a0,s1
   1180c:	01113423          	sd	a7,8(sp)
   11810:	898ff0ef          	jal	ra,108a8 <out>
   11814:	00813883          	ld	a7,8(sp)
   11818:	004c8c93          	addi	s9,s9,4
   1181c:	db9ff06f          	j	115d4 <fmt_fp+0xc60>
   11820:	00096503          	lwu	a0,0(s2)
   11824:	000a8593          	mv	a1,s5
   11828:	fedfe0ef          	jal	ra,10814 <fmt_u>
   1182c:	00050713          	mv	a4,a0
   11830:	08ea6263          	bltu	s4,a4,118b4 <fmt_fp+0xf40>
   11834:	00000593          	li	a1,0
   11838:	01456463          	bltu	a0,s4,11840 <fmt_fp+0xecc>
   1183c:	40aa05b3          	sub	a1,s4,a0
   11840:	0004061b          	sext.w	a2,s0
   11844:	008cd463          	bge	s9,s0,1184c <fmt_fp+0xed8>
   11848:	00900613          	li	a2,9
   1184c:	00b505b3          	add	a1,a0,a1
   11850:	00048513          	mv	a0,s1
   11854:	854ff0ef          	jal	ra,108a8 <out>
   11858:	00490913          	addi	s2,s2,4
   1185c:	ff74041b          	addiw	s0,s0,-9
   11860:	01a97463          	bgeu	s2,s10,11868 <fmt_fp+0xef4>
   11864:	fa804ee3          	bgtz	s0,11820 <fmt_fp+0xeac>
   11868:	00000713          	li	a4,0
   1186c:	00900693          	li	a3,9
   11870:	0094061b          	addiw	a2,s0,9
   11874:	03000593          	li	a1,48
   11878:	00048513          	mv	a0,s1
   1187c:	850ff0ef          	jal	ra,108cc <pad>
   11880:	00013783          	ld	a5,0(sp)
   11884:	00002737          	lui	a4,0x2
   11888:	00048513          	mv	a0,s1
   1188c:	00e7c733          	xor	a4,a5,a4
   11890:	000b0693          	mv	a3,s6
   11894:	00098613          	mv	a2,s3
   11898:	02000593          	li	a1,32
   1189c:	830ff0ef          	jal	ra,108cc <pad>
   118a0:	01012503          	lw	a0,16(sp)
   118a4:	013b5463          	bge	s6,s3,118ac <fmt_fp+0xf38>
   118a8:	00098513          	mv	a0,s3
   118ac:	0005051b          	sext.w	a0,a0
   118b0:	a24ff06f          	j	10ad4 <fmt_fp+0x160>
   118b4:	fff70713          	addi	a4,a4,-1 # 1fff <exit-0xe121>
   118b8:	01770023          	sb	s7,0(a4)
   118bc:	f75ff06f          	j	11830 <fmt_fp+0xebc>
   118c0:	00000413          	li	s0,0
   118c4:	d4dff06f          	j	11610 <fmt_fp+0xc9c>
   118c8:	01abe463          	bltu	s7,s10,118d0 <fmt_fp+0xf5c>
   118cc:	004b8d13          	addi	s10,s7,4
   118d0:	000026b7          	lui	a3,0x2
   118d4:	ffffe737          	lui	a4,0xffffe
   118d8:	04010613          	addi	a2,sp,64
   118dc:	d1068793          	addi	a5,a3,-752 # 1d10 <exit-0xe410>
   118e0:	00c787b3          	add	a5,a5,a2
   118e4:	31070913          	addi	s2,a4,784 # ffffffffffffe310 <__global_pointer$+0xfffffffffffe5b10>
   118e8:	01278933          	add	s2,a5,s2
   118ec:	d1068793          	addi	a5,a3,-752
   118f0:	00c787b3          	add	a5,a5,a2
   118f4:	00e787b3          	add	a5,a5,a4
   118f8:	00f13c23          	sd	a5,24(sp)
   118fc:	00890793          	addi	a5,s2,8
   11900:	000b8a13          	mv	s4,s7
   11904:	00990a93          	addi	s5,s2,9
   11908:	02f13023          	sd	a5,32(sp)
   1190c:	01aa7463          	bgeu	s4,s10,11914 <fmt_fp+0xfa0>
   11910:	04045863          	bgez	s0,11960 <fmt_fp+0xfec>
   11914:	0124061b          	addiw	a2,s0,18
   11918:	00048513          	mv	a0,s1
   1191c:	00000713          	li	a4,0
   11920:	01200693          	li	a3,18
   11924:	03000593          	li	a1,48
   11928:	fa5fe0ef          	jal	ra,108cc <pad>
   1192c:	000027b7          	lui	a5,0x2
   11930:	04010713          	addi	a4,sp,64
   11934:	d1078793          	addi	a5,a5,-752 # 1d10 <exit-0xe410>
   11938:	00e787b3          	add	a5,a5,a4
   1193c:	ffffe637          	lui	a2,0xffffe
   11940:	00c78633          	add	a2,a5,a2
   11944:	00813783          	ld	a5,8(sp)
   11948:	30c60613          	addi	a2,a2,780 # ffffffffffffe30c <__global_pointer$+0xfffffffffffe5b0c>
   1194c:	00048513          	mv	a0,s1
   11950:	40f60633          	sub	a2,a2,a5
   11954:	00078593          	mv	a1,a5
   11958:	f51fe0ef          	jal	ra,108a8 <out>
   1195c:	f25ff06f          	j	11880 <fmt_fp+0xf0c>
   11960:	000a6503          	lwu	a0,0(s4)
   11964:	000a8593          	mv	a1,s5
   11968:	eadfe0ef          	jal	ra,10814 <fmt_u>
   1196c:	00050593          	mv	a1,a0
   11970:	01551a63          	bne	a0,s5,11984 <fmt_fp+0x1010>
   11974:	01813783          	ld	a5,24(sp)
   11978:	02013583          	ld	a1,32(sp)
   1197c:	03000713          	li	a4,48
   11980:	30e78c23          	sb	a4,792(a5)
   11984:	00040d9b          	sext.w	s11,s0
   11988:	00058713          	mv	a4,a1
   1198c:	057a1063          	bne	s4,s7,119cc <fmt_fp+0x1058>
   11990:	00100613          	li	a2,1
   11994:	00048513          	mv	a0,s1
   11998:	00158c13          	addi	s8,a1,1
   1199c:	f0dfe0ef          	jal	ra,108a8 <out>
   119a0:	01bce733          	or	a4,s9,s11
   119a4:	02070e63          	beqz	a4,119e0 <fmt_fp+0x106c>
   119a8:	000167b7          	lui	a5,0x16
   119ac:	00100613          	li	a2,1
   119b0:	cd078593          	addi	a1,a5,-816 # 15cd0 <__clzdi2+0x114>
   119b4:	00048513          	mv	a0,s1
   119b8:	ef1fe0ef          	jal	ra,108a8 <out>
   119bc:	0240006f          	j	119e0 <fmt_fp+0x106c>
   119c0:	fff70713          	addi	a4,a4,-1
   119c4:	03000793          	li	a5,48
   119c8:	00f70023          	sb	a5,0(a4)
   119cc:	fee96ae3          	bltu	s2,a4,119c0 <fmt_fp+0x104c>
   119d0:	00000693          	li	a3,0
   119d4:	0125e463          	bltu	a1,s2,119dc <fmt_fp+0x1068>
   119d8:	40b906b3          	sub	a3,s2,a1
   119dc:	00d58c33          	add	s8,a1,a3
   119e0:	418a88b3          	sub	a7,s5,s8
   119e4:	00040613          	mv	a2,s0
   119e8:	0088d463          	bge	a7,s0,119f0 <fmt_fp+0x107c>
   119ec:	00088613          	mv	a2,a7
   119f0:	000c0593          	mv	a1,s8
   119f4:	00048513          	mv	a0,s1
   119f8:	03113423          	sd	a7,40(sp)
   119fc:	eadfe0ef          	jal	ra,108a8 <out>
   11a00:	02813883          	ld	a7,40(sp)
   11a04:	004a0a13          	addi	s4,s4,4
   11a08:	411d843b          	subw	s0,s11,a7
   11a0c:	f01ff06f          	j	1190c <fmt_fp+0xf98>

0000000000011a10 <printf_core>:
   11a10:	ef010113          	addi	sp,sp,-272
   11a14:	000167b7          	lui	a5,0x16
   11a18:	e7078793          	addi	a5,a5,-400 # 15e70 <states>
   11a1c:	0ba13823          	sd	s10,176(sp)
   11a20:	00016d37          	lui	s10,0x16
   11a24:	0f313423          	sd	s3,232(sp)
   11a28:	00f13823          	sd	a5,16(sp)
   11a2c:	000169b7          	lui	s3,0x16
   11a30:	cf0d0793          	addi	a5,s10,-784 # 15cf0 <__clzdi2+0x134>
   11a34:	10813023          	sd	s0,256(sp)
   11a38:	0f213823          	sd	s2,240(sp)
   11a3c:	0f413023          	sd	s4,224(sp)
   11a40:	0d613823          	sd	s6,208(sp)
   11a44:	0d813023          	sd	s8,192(sp)
   11a48:	0b913c23          	sd	s9,184(sp)
   11a4c:	10113423          	sd	ra,264(sp)
   11a50:	0e913c23          	sd	s1,248(sp)
   11a54:	0d513c23          	sd	s5,216(sp)
   11a58:	0d713423          	sd	s7,200(sp)
   11a5c:	0bb13423          	sd	s11,168(sp)
   11a60:	00050413          	mv	s0,a0
   11a64:	00060a13          	mv	s4,a2
   11a68:	00d13023          	sd	a3,0(sp)
   11a6c:	00070b13          	mv	s6,a4
   11a70:	04b13023          	sd	a1,64(sp)
   11a74:	00000c93          	li	s9,0
   11a78:	00000913          	li	s2,0
   11a7c:	00000c13          	li	s8,0
   11a80:	cd898993          	addi	s3,s3,-808 # 15cd8 <__clzdi2+0x11c>
   11a84:	00f13c23          	sd	a5,24(sp)
   11a88:	04013b83          	ld	s7,64(sp)
   11a8c:	012c893b          	addw	s2,s9,s2
   11a90:	00090d93          	mv	s11,s2
   11a94:	000bc783          	lbu	a5,0(s7)
   11a98:	02078ee3          	beqz	a5,122d4 <printf_core+0x8c4>
   11a9c:	02500713          	li	a4,37
   11aa0:	04013303          	ld	t1,64(sp)
   11aa4:	00034783          	lbu	a5,0(t1)
   11aa8:	00078463          	beqz	a5,11ab0 <printf_core+0xa0>
   11aac:	00e79663          	bne	a5,a4,11ab8 <printf_core+0xa8>
   11ab0:	02500713          	li	a4,37
   11ab4:	01c0006f          	j	11ad0 <printf_core+0xc0>
   11ab8:	00130313          	addi	t1,t1,1
   11abc:	04613023          	sd	t1,64(sp)
   11ac0:	fe1ff06f          	j	11aa0 <printf_core+0x90>
   11ac4:	00278793          	addi	a5,a5,2
   11ac8:	00130313          	addi	t1,t1,1
   11acc:	04f13023          	sd	a5,64(sp)
   11ad0:	04013783          	ld	a5,64(sp)
   11ad4:	0007c683          	lbu	a3,0(a5)
   11ad8:	00e69663          	bne	a3,a4,11ae4 <printf_core+0xd4>
   11adc:	0017c683          	lbu	a3,1(a5)
   11ae0:	fee682e3          	beq	a3,a4,11ac4 <printf_core+0xb4>
   11ae4:	80000ab7          	lui	s5,0x80000
   11ae8:	fffaca93          	not	s5,s5
   11aec:	41ba87bb          	subw	a5,s5,s11
   11af0:	41730333          	sub	t1,t1,s7
   11af4:	00f13423          	sd	a5,8(sp)
   11af8:	7467ca63          	blt	a5,t1,1224c <printf_core+0x83c>
   11afc:	00030c9b          	sext.w	s9,t1
   11b00:	00040a63          	beqz	s0,11b14 <printf_core+0x104>
   11b04:	000c8613          	mv	a2,s9
   11b08:	000b8593          	mv	a1,s7
   11b0c:	00040513          	mv	a0,s0
   11b10:	d99fe0ef          	jal	ra,108a8 <out>
   11b14:	720c9863          	bnez	s9,12244 <printf_core+0x834>
   11b18:	04013783          	ld	a5,64(sp)
   11b1c:	00900713          	li	a4,9
   11b20:	0017c683          	lbu	a3,1(a5)
   11b24:	fd06869b          	addiw	a3,a3,-48
   11b28:	18d76663          	bltu	a4,a3,11cb4 <printf_core+0x2a4>
   11b2c:	0027c603          	lbu	a2,2(a5)
   11b30:	02400713          	li	a4,36
   11b34:	18e61063          	bne	a2,a4,11cb4 <printf_core+0x2a4>
   11b38:	00378793          	addi	a5,a5,3
   11b3c:	00100c13          	li	s8,1
   11b40:	00013537          	lui	a0,0x13
   11b44:	04f13023          	sd	a5,64(sp)
   11b48:	00000d13          	li	s10,0
   11b4c:	01f00893          	li	a7,31
   11b50:	8895051b          	addiw	a0,a0,-1911
   11b54:	00100e93          	li	t4,1
   11b58:	04013703          	ld	a4,64(sp)
   11b5c:	00074803          	lbu	a6,0(a4)
   11b60:	fe08061b          	addiw	a2,a6,-32
   11b64:	00c8e863          	bltu	a7,a2,11b74 <printf_core+0x164>
   11b68:	00c555bb          	srlw	a1,a0,a2
   11b6c:	0015f593          	andi	a1,a1,1
   11b70:	14059863          	bnez	a1,11cc0 <printf_core+0x2b0>
   11b74:	02a00613          	li	a2,42
   11b78:	18c81e63          	bne	a6,a2,11d14 <printf_core+0x304>
   11b7c:	00174603          	lbu	a2,1(a4)
   11b80:	00900593          	li	a1,9
   11b84:	fd06051b          	addiw	a0,a2,-48
   11b88:	14a5e863          	bltu	a1,a0,11cd8 <printf_core+0x2c8>
   11b8c:	00274503          	lbu	a0,2(a4)
   11b90:	02400593          	li	a1,36
   11b94:	14b51263          	bne	a0,a1,11cd8 <printf_core+0x2c8>
   11b98:	00261613          	slli	a2,a2,0x2
   11b9c:	00cb0633          	add	a2,s6,a2
   11ba0:	00a00593          	li	a1,10
   11ba4:	f4b62023          	sw	a1,-192(a2)
   11ba8:	00174603          	lbu	a2,1(a4)
   11bac:	00013783          	ld	a5,0(sp)
   11bb0:	00370713          	addi	a4,a4,3
   11bb4:	00461613          	slli	a2,a2,0x4
   11bb8:	00c78633          	add	a2,a5,a2
   11bbc:	d0062483          	lw	s1,-768(a2)
   11bc0:	04e13023          	sd	a4,64(sp)
   11bc4:	00100c13          	li	s8,1
   11bc8:	0004d863          	bgez	s1,11bd8 <printf_core+0x1c8>
   11bcc:	00002737          	lui	a4,0x2
   11bd0:	00ed6d33          	or	s10,s10,a4
   11bd4:	409004bb          	negw	s1,s1
   11bd8:	04013703          	ld	a4,64(sp)
   11bdc:	02e00613          	li	a2,46
   11be0:	00074583          	lbu	a1,0(a4) # 2000 <exit-0xe120>
   11be4:	18c59a63          	bne	a1,a2,11d78 <printf_core+0x368>
   11be8:	00174583          	lbu	a1,1(a4)
   11bec:	02a00613          	li	a2,42
   11bf0:	16c59263          	bne	a1,a2,11d54 <printf_core+0x344>
   11bf4:	00274603          	lbu	a2,2(a4)
   11bf8:	00900593          	li	a1,9
   11bfc:	fd06051b          	addiw	a0,a2,-48
   11c00:	12a5e863          	bltu	a1,a0,11d30 <printf_core+0x320>
   11c04:	00374503          	lbu	a0,3(a4)
   11c08:	02400593          	li	a1,36
   11c0c:	12b51263          	bne	a0,a1,11d30 <printf_core+0x320>
   11c10:	00261613          	slli	a2,a2,0x2
   11c14:	00cb0633          	add	a2,s6,a2
   11c18:	00a00593          	li	a1,10
   11c1c:	f4b62023          	sw	a1,-192(a2)
   11c20:	00274603          	lbu	a2,2(a4)
   11c24:	00013783          	ld	a5,0(sp)
   11c28:	00470713          	addi	a4,a4,4
   11c2c:	00461613          	slli	a2,a2,0x4
   11c30:	00c78633          	add	a2,a5,a2
   11c34:	d0062a83          	lw	s5,-768(a2)
   11c38:	fffacf13          	not	t5,s5
   11c3c:	04e13023          	sd	a4,64(sp)
   11c40:	01ff5f1b          	srliw	t5,t5,0x1f
   11c44:	00000713          	li	a4,0
   11c48:	03900e93          	li	t4,57
   11c4c:	03a00813          	li	a6,58
   11c50:	00700513          	li	a0,7
   11c54:	04013603          	ld	a2,64(sp)
   11c58:	00064583          	lbu	a1,0(a2)
   11c5c:	fbf5859b          	addiw	a1,a1,-65
   11c60:	06beee63          	bltu	t4,a1,11cdc <printf_core+0x2cc>
   11c64:	00160593          	addi	a1,a2,1
   11c68:	04b13023          	sd	a1,64(sp)
   11c6c:	00064603          	lbu	a2,0(a2)
   11c70:	01013783          	ld	a5,16(sp)
   11c74:	fbf6059b          	addiw	a1,a2,-65
   11c78:	02071613          	slli	a2,a4,0x20
   11c7c:	02065613          	srli	a2,a2,0x20
   11c80:	03060633          	mul	a2,a2,a6
   11c84:	00c78633          	add	a2,a5,a2
   11c88:	00b60633          	add	a2,a2,a1
   11c8c:	00064f83          	lbu	t6,0(a2)
   11c90:	ffff861b          	addiw	a2,t6,-1
   11c94:	000f859b          	sext.w	a1,t6
   11c98:	0ec57663          	bgeu	a0,a2,11d84 <printf_core+0x374>
   11c9c:	04058063          	beqz	a1,11cdc <printf_core+0x2cc>
   11ca0:	01b00613          	li	a2,27
   11ca4:	0ec59463          	bne	a1,a2,11d8c <printf_core+0x37c>
   11ca8:	0206da63          	bgez	a3,11cdc <printf_core+0x2cc>
   11cac:	16041a63          	bnez	s0,11e20 <printf_core+0x410>
   11cb0:	dd9ff06f          	j	11a88 <printf_core+0x78>
   11cb4:	00178793          	addi	a5,a5,1
   11cb8:	fff00693          	li	a3,-1
   11cbc:	e85ff06f          	j	11b40 <printf_core+0x130>
   11cc0:	00ce963b          	sllw	a2,t4,a2
   11cc4:	00cd67b3          	or	a5,s10,a2
   11cc8:	00170713          	addi	a4,a4,1
   11ccc:	00078d1b          	sext.w	s10,a5
   11cd0:	04e13023          	sd	a4,64(sp)
   11cd4:	e85ff06f          	j	11b58 <printf_core+0x148>
   11cd8:	000c0c63          	beqz	s8,11cf0 <printf_core+0x2e0>
   11cdc:	479000ef          	jal	ra,12954 <__errno_location>
   11ce0:	01600793          	li	a5,22
   11ce4:	00f52023          	sw	a5,0(a0) # 13000 <memcpy+0x14>
   11ce8:	fff00913          	li	s2,-1
   11cec:	0d80006f          	j	11dc4 <printf_core+0x3b4>
   11cf0:	00000493          	li	s1,0
   11cf4:	00040a63          	beqz	s0,11d08 <printf_core+0x2f8>
   11cf8:	000a3603          	ld	a2,0(s4)
   11cfc:	00062483          	lw	s1,0(a2)
   11d00:	00860593          	addi	a1,a2,8
   11d04:	00ba3023          	sd	a1,0(s4)
   11d08:	00170713          	addi	a4,a4,1
   11d0c:	04e13023          	sd	a4,64(sp)
   11d10:	eb9ff06f          	j	11bc8 <printf_core+0x1b8>
   11d14:	04010513          	addi	a0,sp,64
   11d18:	02d13023          	sd	a3,32(sp)
   11d1c:	b25fe0ef          	jal	ra,10840 <getint>
   11d20:	02013683          	ld	a3,32(sp)
   11d24:	00050493          	mv	s1,a0
   11d28:	ea0558e3          	bgez	a0,11bd8 <printf_core+0x1c8>
   11d2c:	5200006f          	j	1224c <printf_core+0x83c>
   11d30:	fa0c16e3          	bnez	s8,11cdc <printf_core+0x2cc>
   11d34:	00000a93          	li	s5,0
   11d38:	00040a63          	beqz	s0,11d4c <printf_core+0x33c>
   11d3c:	000a3603          	ld	a2,0(s4)
   11d40:	00062a83          	lw	s5,0(a2)
   11d44:	00860593          	addi	a1,a2,8
   11d48:	00ba3023          	sd	a1,0(s4)
   11d4c:	00270713          	addi	a4,a4,2
   11d50:	ee9ff06f          	j	11c38 <printf_core+0x228>
   11d54:	00170713          	addi	a4,a4,1
   11d58:	04010513          	addi	a0,sp,64
   11d5c:	02d13023          	sd	a3,32(sp)
   11d60:	04e13023          	sd	a4,64(sp)
   11d64:	addfe0ef          	jal	ra,10840 <getint>
   11d68:	02013683          	ld	a3,32(sp)
   11d6c:	00050a93          	mv	s5,a0
   11d70:	00100f13          	li	t5,1
   11d74:	ed1ff06f          	j	11c44 <printf_core+0x234>
   11d78:	00000f13          	li	t5,0
   11d7c:	fff00a93          	li	s5,-1
   11d80:	ec5ff06f          	j	11c44 <printf_core+0x234>
   11d84:	00058713          	mv	a4,a1
   11d88:	ecdff06f          	j	11c54 <printf_core+0x244>
   11d8c:	0206c863          	bltz	a3,11dbc <printf_core+0x3ac>
   11d90:	00013783          	ld	a5,0(sp)
   11d94:	00269613          	slli	a2,a3,0x2
   11d98:	00469693          	slli	a3,a3,0x4
   11d9c:	00cb0633          	add	a2,s6,a2
   11da0:	00d786b3          	add	a3,a5,a3
   11da4:	01f62023          	sw	t6,0(a2)
   11da8:	0006b603          	ld	a2,0(a3)
   11dac:	0086b683          	ld	a3,8(a3)
   11db0:	04c13823          	sd	a2,80(sp)
   11db4:	04d13c23          	sd	a3,88(sp)
   11db8:	ef5ff06f          	j	11cac <printf_core+0x29c>
   11dbc:	04041463          	bnez	s0,11e04 <printf_core+0x3f4>
   11dc0:	00000913          	li	s2,0
   11dc4:	10813083          	ld	ra,264(sp)
   11dc8:	10013403          	ld	s0,256(sp)
   11dcc:	0f813483          	ld	s1,248(sp)
   11dd0:	0e813983          	ld	s3,232(sp)
   11dd4:	0e013a03          	ld	s4,224(sp)
   11dd8:	0d813a83          	ld	s5,216(sp)
   11ddc:	0d013b03          	ld	s6,208(sp)
   11de0:	0c813b83          	ld	s7,200(sp)
   11de4:	0c013c03          	ld	s8,192(sp)
   11de8:	0b813c83          	ld	s9,184(sp)
   11dec:	0b013d03          	ld	s10,176(sp)
   11df0:	0a813d83          	ld	s11,168(sp)
   11df4:	00090513          	mv	a0,s2
   11df8:	0f013903          	ld	s2,240(sp)
   11dfc:	11010113          	addi	sp,sp,272
   11e00:	00008067          	ret
   11e04:	000a0613          	mv	a2,s4
   11e08:	05010513          	addi	a0,sp,80
   11e0c:	02e13423          	sd	a4,40(sp)
   11e10:	03e13023          	sd	t5,32(sp)
   11e14:	8fdfe0ef          	jal	ra,10710 <pop_arg>
   11e18:	02013f03          	ld	t5,32(sp)
   11e1c:	02813703          	ld	a4,40(sp)
   11e20:	04013683          	ld	a3,64(sp)
   11e24:	fff6c683          	lbu	a3,-1(a3)
   11e28:	0006881b          	sext.w	a6,a3
   11e2c:	00070a63          	beqz	a4,11e40 <printf_core+0x430>
   11e30:	00f6f693          	andi	a3,a3,15
   11e34:	00300613          	li	a2,3
   11e38:	00c69463          	bne	a3,a2,11e40 <printf_core+0x430>
   11e3c:	0df87813          	andi	a6,a6,223
   11e40:	00dd5693          	srli	a3,s10,0xd
   11e44:	0016f693          	andi	a3,a3,1
   11e48:	00068863          	beqz	a3,11e58 <printf_core+0x448>
   11e4c:	ffff06b7          	lui	a3,0xffff0
   11e50:	fff68693          	addi	a3,a3,-1 # fffffffffffeffff <__global_pointer$+0xfffffffffffd77ff>
   11e54:	00dd7d33          	and	s10,s10,a3
   11e58:	fbf8069b          	addiw	a3,a6,-65
   11e5c:	0006859b          	sext.w	a1,a3
   11e60:	03700613          	li	a2,55
   11e64:	44b66e63          	bltu	a2,a1,122c0 <printf_core+0x8b0>
   11e68:	01813783          	ld	a5,24(sp)
   11e6c:	02069693          	slli	a3,a3,0x20
   11e70:	01e6d693          	srli	a3,a3,0x1e
   11e74:	00f686b3          	add	a3,a3,a5
   11e78:	0006a683          	lw	a3,0(a3)
   11e7c:	00068067          	jr	a3
   11e80:	00700793          	li	a5,7
   11e84:	3ce7e063          	bltu	a5,a4,12244 <printf_core+0x834>
   11e88:	00271793          	slli	a5,a4,0x2
   11e8c:	00016737          	lui	a4,0x16
   11e90:	dd070713          	addi	a4,a4,-560 # 15dd0 <__clzdi2+0x214>
   11e94:	00e787b3          	add	a5,a5,a4
   11e98:	0007a783          	lw	a5,0(a5)
   11e9c:	00078067          	jr	a5
   11ea0:	05013783          	ld	a5,80(sp)
   11ea4:	01b7a023          	sw	s11,0(a5)
   11ea8:	be1ff06f          	j	11a88 <printf_core+0x78>
   11eac:	05013783          	ld	a5,80(sp)
   11eb0:	01279023          	sh	s2,0(a5)
   11eb4:	bd5ff06f          	j	11a88 <printf_core+0x78>
   11eb8:	05013783          	ld	a5,80(sp)
   11ebc:	01278023          	sb	s2,0(a5)
   11ec0:	bc9ff06f          	j	11a88 <printf_core+0x78>
   11ec4:	05013783          	ld	a5,80(sp)
   11ec8:	0127b023          	sd	s2,0(a5)
   11ecc:	bbdff06f          	j	11a88 <printf_core+0x78>
   11ed0:	000a889b          	sext.w	a7,s5
   11ed4:	01000693          	li	a3,16
   11ed8:	000a8713          	mv	a4,s5
   11edc:	00d8f463          	bgeu	a7,a3,11ee4 <printf_core+0x4d4>
   11ee0:	01000713          	li	a4,16
   11ee4:	00070a9b          	sext.w	s5,a4
   11ee8:	008d6d13          	ori	s10,s10,8
   11eec:	07800813          	li	a6,120
   11ef0:	05013583          	ld	a1,80(sp)
   11ef4:	000166b7          	lui	a3,0x16
   11ef8:	02087513          	andi	a0,a6,32
   11efc:	00058713          	mv	a4,a1
   11f00:	09f10b93          	addi	s7,sp,159
   11f04:	04068693          	addi	a3,a3,64 # 16040 <xdigits>
   11f08:	12071c63          	bnez	a4,12040 <printf_core+0x630>
   11f0c:	00098e93          	mv	t4,s3
   11f10:	00058c63          	beqz	a1,11f28 <printf_core+0x518>
   11f14:	008d7713          	andi	a4,s10,8
   11f18:	00070863          	beqz	a4,11f28 <printf_core+0x518>
   11f1c:	00485813          	srli	a6,a6,0x4
   11f20:	01098eb3          	add	t4,s3,a6
   11f24:	00200c93          	li	s9,2
   11f28:	000f0a63          	beqz	t5,11f3c <printf_core+0x52c>
   11f2c:	320ac063          	bltz	s5,1224c <printf_core+0x83c>
   11f30:	ffff0737          	lui	a4,0xffff0
   11f34:	fff70713          	addi	a4,a4,-1 # fffffffffffeffff <__global_pointer$+0xfffffffffffd77ff>
   11f38:	00ed7d33          	and	s10,s10,a4
   11f3c:	05013683          	ld	a3,80(sp)
   11f40:	09f10613          	addi	a2,sp,159
   11f44:	00069463          	bnez	a3,11f4c <printf_core+0x53c>
   11f48:	380a8263          	beqz	s5,122cc <printf_core+0x8bc>
   11f4c:	41760733          	sub	a4,a2,s7
   11f50:	0016b693          	seqz	a3,a3
   11f54:	00d70733          	add	a4,a4,a3
   11f58:	01575463          	bge	a4,s5,11f60 <printf_core+0x550>
   11f5c:	000a8713          	mv	a4,s5
   11f60:	00070a9b          	sext.w	s5,a4
   11f64:	41760db3          	sub	s11,a2,s7
   11f68:	01bad463          	bge	s5,s11,11f70 <printf_core+0x560>
   11f6c:	000d8a9b          	sext.w	s5,s11
   11f70:	80000737          	lui	a4,0x80000
   11f74:	fff74713          	not	a4,a4
   11f78:	4197073b          	subw	a4,a4,s9
   11f7c:	2d574863          	blt	a4,s5,1224c <printf_core+0x83c>
   11f80:	019a883b          	addw	a6,s5,s9
   11f84:	00080713          	mv	a4,a6
   11f88:	00985463          	bge	a6,s1,11f90 <printf_core+0x580>
   11f8c:	00048713          	mv	a4,s1
   11f90:	00813783          	ld	a5,8(sp)
   11f94:	03d13023          	sd	t4,32(sp)
   11f98:	0007049b          	sext.w	s1,a4
   11f9c:	2a97c863          	blt	a5,s1,1224c <printf_core+0x83c>
   11fa0:	00080693          	mv	a3,a6
   11fa4:	000d0713          	mv	a4,s10
   11fa8:	00048613          	mv	a2,s1
   11fac:	02000593          	li	a1,32
   11fb0:	00040513          	mv	a0,s0
   11fb4:	01013423          	sd	a6,8(sp)
   11fb8:	915fe0ef          	jal	ra,108cc <pad>
   11fbc:	02013e83          	ld	t4,32(sp)
   11fc0:	000c8613          	mv	a2,s9
   11fc4:	00040513          	mv	a0,s0
   11fc8:	000e8593          	mv	a1,t4
   11fcc:	8ddfe0ef          	jal	ra,108a8 <out>
   11fd0:	00813803          	ld	a6,8(sp)
   11fd4:	00010737          	lui	a4,0x10
   11fd8:	00ed4733          	xor	a4,s10,a4
   11fdc:	00080693          	mv	a3,a6
   11fe0:	00048613          	mv	a2,s1
   11fe4:	03000593          	li	a1,48
   11fe8:	00040513          	mv	a0,s0
   11fec:	8e1fe0ef          	jal	ra,108cc <pad>
   11ff0:	00000713          	li	a4,0
   11ff4:	000d869b          	sext.w	a3,s11
   11ff8:	000a8613          	mv	a2,s5
   11ffc:	03000593          	li	a1,48
   12000:	00040513          	mv	a0,s0
   12004:	8c9fe0ef          	jal	ra,108cc <pad>
   12008:	000d8613          	mv	a2,s11
   1200c:	000b8593          	mv	a1,s7
   12010:	00040513          	mv	a0,s0
   12014:	895fe0ef          	jal	ra,108a8 <out>
   12018:	00813803          	ld	a6,8(sp)
   1201c:	00002737          	lui	a4,0x2
   12020:	00ed4733          	xor	a4,s10,a4
   12024:	00080693          	mv	a3,a6
   12028:	00048613          	mv	a2,s1
   1202c:	02000593          	li	a1,32
   12030:	00040513          	mv	a0,s0
   12034:	899fe0ef          	jal	ra,108cc <pad>
   12038:	00048c93          	mv	s9,s1
   1203c:	a4dff06f          	j	11a88 <printf_core+0x78>
   12040:	00f77613          	andi	a2,a4,15
   12044:	00c68633          	add	a2,a3,a2
   12048:	00064603          	lbu	a2,0(a2)
   1204c:	fffb8b93          	addi	s7,s7,-1
   12050:	00475713          	srli	a4,a4,0x4
   12054:	00c56633          	or	a2,a0,a2
   12058:	00cb8023          	sb	a2,0(s7)
   1205c:	eadff06f          	j	11f08 <printf_core+0x4f8>
   12060:	05013683          	ld	a3,80(sp)
   12064:	09f10b93          	addi	s7,sp,159
   12068:	000b8713          	mv	a4,s7
   1206c:	02069063          	bnez	a3,1208c <printf_core+0x67c>
   12070:	008d7693          	andi	a3,s10,8
   12074:	00098e93          	mv	t4,s3
   12078:	ea0688e3          	beqz	a3,11f28 <printf_core+0x518>
   1207c:	41770733          	sub	a4,a4,s7
   12080:	eb5744e3          	blt	a4,s5,11f28 <printf_core+0x518>
   12084:	00170a9b          	addiw	s5,a4,1
   12088:	ea1ff06f          	j	11f28 <printf_core+0x518>
   1208c:	0076f613          	andi	a2,a3,7
   12090:	fffb8b93          	addi	s7,s7,-1
   12094:	0306061b          	addiw	a2,a2,48
   12098:	00cb8023          	sb	a2,0(s7)
   1209c:	0036d693          	srli	a3,a3,0x3
   120a0:	fcdff06f          	j	1206c <printf_core+0x65c>
   120a4:	05013703          	ld	a4,80(sp)
   120a8:	02075c63          	bgez	a4,120e0 <printf_core+0x6d0>
   120ac:	40e00733          	neg	a4,a4
   120b0:	04e13823          	sd	a4,80(sp)
   120b4:	00100c93          	li	s9,1
   120b8:	00098e93          	mv	t4,s3
   120bc:	05013503          	ld	a0,80(sp)
   120c0:	09f10593          	addi	a1,sp,159
   120c4:	03d13423          	sd	t4,40(sp)
   120c8:	03e13023          	sd	t5,32(sp)
   120cc:	f48fe0ef          	jal	ra,10814 <fmt_u>
   120d0:	02813e83          	ld	t4,40(sp)
   120d4:	02013f03          	ld	t5,32(sp)
   120d8:	00050b93          	mv	s7,a0
   120dc:	e4dff06f          	j	11f28 <printf_core+0x518>
   120e0:	00bd5713          	srli	a4,s10,0xb
   120e4:	00177713          	andi	a4,a4,1
   120e8:	00071e63          	bnez	a4,12104 <printf_core+0x6f4>
   120ec:	001d7713          	andi	a4,s10,1
   120f0:	fc0704e3          	beqz	a4,120b8 <printf_core+0x6a8>
   120f4:	00016eb7          	lui	t4,0x16
   120f8:	00100c93          	li	s9,1
   120fc:	cdae8e93          	addi	t4,t4,-806 # 15cda <__clzdi2+0x11e>
   12100:	fbdff06f          	j	120bc <printf_core+0x6ac>
   12104:	00016eb7          	lui	t4,0x16
   12108:	00100c93          	li	s9,1
   1210c:	cd9e8e93          	addi	t4,t4,-807 # 15cd9 <__clzdi2+0x11d>
   12110:	fadff06f          	j	120bc <printf_core+0x6ac>
   12114:	05013703          	ld	a4,80(sp)
   12118:	00098e93          	mv	t4,s3
   1211c:	00100a93          	li	s5,1
   12120:	08e10f23          	sb	a4,158(sp)
   12124:	ffff0737          	lui	a4,0xffff0
   12128:	fff70713          	addi	a4,a4,-1 # fffffffffffeffff <__global_pointer$+0xfffffffffffd77ff>
   1212c:	00ed7d33          	and	s10,s10,a4
   12130:	09f10613          	addi	a2,sp,159
   12134:	09e10b93          	addi	s7,sp,158
   12138:	e2dff06f          	j	11f64 <printf_core+0x554>
   1213c:	019000ef          	jal	ra,12954 <__errno_location>
   12140:	00052503          	lw	a0,0(a0)
   12144:	069000ef          	jal	ra,129ac <strerror>
   12148:	00050b93          	mv	s7,a0
   1214c:	000a8593          	mv	a1,s5
   12150:	000ad663          	bgez	s5,1215c <printf_core+0x74c>
   12154:	800005b7          	lui	a1,0x80000
   12158:	fff5c593          	not	a1,a1
   1215c:	000b8513          	mv	a0,s7
   12160:	448000ef          	jal	ra,125a8 <strnlen>
   12164:	00ab8633          	add	a2,s7,a0
   12168:	000ad663          	bgez	s5,12174 <printf_core+0x764>
   1216c:	00064703          	lbu	a4,0(a2)
   12170:	0c071e63          	bnez	a4,1224c <printf_core+0x83c>
   12174:	ffff0737          	lui	a4,0xffff0
   12178:	fff70713          	addi	a4,a4,-1 # fffffffffffeffff <__global_pointer$+0xfffffffffffd77ff>
   1217c:	00050a9b          	sext.w	s5,a0
   12180:	00ed7d33          	and	s10,s10,a4
   12184:	00098e93          	mv	t4,s3
   12188:	dddff06f          	j	11f64 <printf_core+0x554>
   1218c:	05013b83          	ld	s7,80(sp)
   12190:	fa0b9ee3          	bnez	s7,1214c <printf_core+0x73c>
   12194:	00016e37          	lui	t3,0x16
   12198:	ce8e0b93          	addi	s7,t3,-792 # 15ce8 <__clzdi2+0x12c>
   1219c:	fb1ff06f          	j	1214c <printf_core+0x73c>
   121a0:	05013703          	ld	a4,80(sp)
   121a4:	04012623          	sw	zero,76(sp)
   121a8:	fff00a93          	li	s5,-1
   121ac:	04e12423          	sw	a4,72(sp)
   121b0:	04810713          	addi	a4,sp,72
   121b4:	04e13823          	sd	a4,80(sp)
   121b8:	05013d83          	ld	s11,80(sp)
   121bc:	00000c93          	li	s9,0
   121c0:	035cf263          	bgeu	s9,s5,121e4 <printf_core+0x7d4>
   121c4:	000da583          	lw	a1,0(s11)
   121c8:	00058e63          	beqz	a1,121e4 <printf_core+0x7d4>
   121cc:	03810513          	addi	a0,sp,56
   121d0:	004d8d93          	addi	s11,s11,4
   121d4:	175000ef          	jal	ra,12b48 <wctomb>
   121d8:	b00548e3          	bltz	a0,11ce8 <printf_core+0x2d8>
   121dc:	419a8733          	sub	a4,s5,s9
   121e0:	06a77c63          	bgeu	a4,a0,12258 <printf_core+0x848>
   121e4:	80000737          	lui	a4,0x80000
   121e8:	fff74713          	not	a4,a4
   121ec:	07976063          	bltu	a4,s9,1224c <printf_core+0x83c>
   121f0:	000c8d9b          	sext.w	s11,s9
   121f4:	000d0713          	mv	a4,s10
   121f8:	000d8693          	mv	a3,s11
   121fc:	00048613          	mv	a2,s1
   12200:	02000593          	li	a1,32
   12204:	00040513          	mv	a0,s0
   12208:	ec4fe0ef          	jal	ra,108cc <pad>
   1220c:	05013b83          	ld	s7,80(sp)
   12210:	00000a93          	li	s5,0
   12214:	059ae663          	bltu	s5,s9,12260 <printf_core+0x850>
   12218:	00002737          	lui	a4,0x2
   1221c:	00ed4733          	xor	a4,s10,a4
   12220:	000d8693          	mv	a3,s11
   12224:	00048613          	mv	a2,s1
   12228:	02000593          	li	a1,32
   1222c:	00040513          	mv	a0,s0
   12230:	e9cfe0ef          	jal	ra,108cc <pad>
   12234:	00048313          	mv	t1,s1
   12238:	01b4d463          	bge	s1,s11,12240 <printf_core+0x830>
   1223c:	000d8313          	mv	t1,s11
   12240:	00030c9b          	sext.w	s9,t1
   12244:	00813783          	ld	a5,8(sp)
   12248:	8597d0e3          	bge	a5,s9,11a88 <printf_core+0x78>
   1224c:	708000ef          	jal	ra,12954 <__errno_location>
   12250:	04b00793          	li	a5,75
   12254:	a91ff06f          	j	11ce4 <printf_core+0x2d4>
   12258:	00ac8cb3          	add	s9,s9,a0
   1225c:	f65ff06f          	j	121c0 <printf_core+0x7b0>
   12260:	000ba583          	lw	a1,0(s7)
   12264:	fa058ae3          	beqz	a1,12218 <printf_core+0x808>
   12268:	03810513          	addi	a0,sp,56
   1226c:	0dd000ef          	jal	ra,12b48 <wctomb>
   12270:	00aa8ab3          	add	s5,s5,a0
   12274:	004b8b93          	addi	s7,s7,4
   12278:	00050613          	mv	a2,a0
   1227c:	f95ceee3          	bltu	s9,s5,12218 <printf_core+0x808>
   12280:	03810593          	addi	a1,sp,56
   12284:	00040513          	mv	a0,s0
   12288:	e20fe0ef          	jal	ra,108a8 <out>
   1228c:	f89ff06f          	j	12214 <printf_core+0x804>
   12290:	000f0463          	beqz	t5,12298 <printf_core+0x888>
   12294:	fa0acce3          	bltz	s5,1224c <printf_core+0x83c>
   12298:	05013583          	ld	a1,80(sp)
   1229c:	05813603          	ld	a2,88(sp)
   122a0:	000d0793          	mv	a5,s10
   122a4:	000a8713          	mv	a4,s5
   122a8:	00048693          	mv	a3,s1
   122ac:	00040513          	mv	a0,s0
   122b0:	ec4fe0ef          	jal	ra,10974 <fmt_fp>
   122b4:	00050c93          	mv	s9,a0
   122b8:	f80556e3          	bgez	a0,12244 <printf_core+0x834>
   122bc:	f91ff06f          	j	1224c <printf_core+0x83c>
   122c0:	00098e93          	mv	t4,s3
   122c4:	09f10613          	addi	a2,sp,159
   122c8:	c9dff06f          	j	11f64 <printf_core+0x554>
   122cc:	00060b93          	mv	s7,a2
   122d0:	c95ff06f          	j	11f64 <printf_core+0x554>
   122d4:	ae0418e3          	bnez	s0,11dc4 <printf_core+0x3b4>
   122d8:	ae0c04e3          	beqz	s8,11dc0 <printf_core+0x3b0>
   122dc:	00100413          	li	s0,1
   122e0:	00a00493          	li	s1,10
   122e4:	00241793          	slli	a5,s0,0x2
   122e8:	00fb07b3          	add	a5,s6,a5
   122ec:	0007a583          	lw	a1,0(a5)
   122f0:	02059263          	bnez	a1,12314 <printf_core+0x904>
   122f4:	00a00713          	li	a4,10
   122f8:	00241793          	slli	a5,s0,0x2
   122fc:	00fb07b3          	add	a5,s6,a5
   12300:	0007a783          	lw	a5,0(a5)
   12304:	9c079ce3          	bnez	a5,11cdc <printf_core+0x2cc>
   12308:	00140413          	addi	s0,s0,1
   1230c:	fee416e3          	bne	s0,a4,122f8 <printf_core+0x8e8>
   12310:	0200006f          	j	12330 <printf_core+0x920>
   12314:	00013783          	ld	a5,0(sp)
   12318:	00441513          	slli	a0,s0,0x4
   1231c:	000a0613          	mv	a2,s4
   12320:	00a78533          	add	a0,a5,a0
   12324:	00140413          	addi	s0,s0,1
   12328:	be8fe0ef          	jal	ra,10710 <pop_arg>
   1232c:	fa941ce3          	bne	s0,s1,122e4 <printf_core+0x8d4>
   12330:	00100913          	li	s2,1
   12334:	a91ff06f          	j	11dc4 <printf_core+0x3b4>

0000000000012338 <vfprintf>:
   12338:	ea010113          	addi	sp,sp,-352
   1233c:	14813823          	sd	s0,336(sp)
   12340:	00c13023          	sd	a2,0(sp)
   12344:	00050413          	mv	s0,a0
   12348:	00810713          	addi	a4,sp,8
   1234c:	08010693          	addi	a3,sp,128
   12350:	00010613          	mv	a2,sp
   12354:	00000513          	li	a0,0
   12358:	14913423          	sd	s1,328(sp)
   1235c:	13513423          	sd	s5,296(sp)
   12360:	14113c23          	sd	ra,344(sp)
   12364:	15213023          	sd	s2,320(sp)
   12368:	13313c23          	sd	s3,312(sp)
   1236c:	13413823          	sd	s4,304(sp)
   12370:	00058a93          	mv	s5,a1
   12374:	00013423          	sd	zero,8(sp)
   12378:	00013823          	sd	zero,16(sp)
   1237c:	00013c23          	sd	zero,24(sp)
   12380:	02013023          	sd	zero,32(sp)
   12384:	02013423          	sd	zero,40(sp)
   12388:	e88ff0ef          	jal	ra,11a10 <printf_core>
   1238c:	fff00493          	li	s1,-1
   12390:	0e054463          	bltz	a0,12478 <vfprintf+0x140>
   12394:	08c42783          	lw	a5,140(s0)
   12398:	00000993          	li	s3,0
   1239c:	0007879b          	sext.w	a5,a5
   123a0:	0007c863          	bltz	a5,123b0 <vfprintf+0x78>
   123a4:	00040513          	mv	a0,s0
   123a8:	7cc000ef          	jal	ra,12b74 <__lockfile>
   123ac:	00050993          	mv	s3,a0
   123b0:	00042783          	lw	a5,0(s0)
   123b4:	08842703          	lw	a4,136(s0)
   123b8:	0207fa13          	andi	s4,a5,32
   123bc:	00e04663          	bgtz	a4,123c8 <vfprintf+0x90>
   123c0:	fdf7f793          	andi	a5,a5,-33
   123c4:	00f42023          	sw	a5,0(s0)
   123c8:	06043783          	ld	a5,96(s0)
   123cc:	0c079a63          	bnez	a5,124a0 <vfprintf+0x168>
   123d0:	03010793          	addi	a5,sp,48
   123d4:	05843903          	ld	s2,88(s0)
   123d8:	04f43c23          	sd	a5,88(s0)
   123dc:	05000793          	li	a5,80
   123e0:	06f43023          	sd	a5,96(s0)
   123e4:	02043023          	sd	zero,32(s0)
   123e8:	02043c23          	sd	zero,56(s0)
   123ec:	02043423          	sd	zero,40(s0)
   123f0:	00040513          	mv	a0,s0
   123f4:	159000ef          	jal	ra,12d4c <__towrite>
   123f8:	fff00493          	li	s1,-1
   123fc:	02051063          	bnez	a0,1241c <vfprintf+0xe4>
   12400:	00810713          	addi	a4,sp,8
   12404:	08010693          	addi	a3,sp,128
   12408:	00010613          	mv	a2,sp
   1240c:	000a8593          	mv	a1,s5
   12410:	00040513          	mv	a0,s0
   12414:	dfcff0ef          	jal	ra,11a10 <printf_core>
   12418:	00050493          	mv	s1,a0
   1241c:	02090c63          	beqz	s2,12454 <vfprintf+0x11c>
   12420:	04843783          	ld	a5,72(s0)
   12424:	00000613          	li	a2,0
   12428:	00000593          	li	a1,0
   1242c:	00040513          	mv	a0,s0
   12430:	000780e7          	jalr	a5
   12434:	02843783          	ld	a5,40(s0)
   12438:	00079463          	bnez	a5,12440 <vfprintf+0x108>
   1243c:	fff00493          	li	s1,-1
   12440:	05243c23          	sd	s2,88(s0)
   12444:	06043023          	sd	zero,96(s0)
   12448:	02043023          	sd	zero,32(s0)
   1244c:	02043c23          	sd	zero,56(s0)
   12450:	02043423          	sd	zero,40(s0)
   12454:	00042783          	lw	a5,0(s0)
   12458:	0207f713          	andi	a4,a5,32
   1245c:	00070463          	beqz	a4,12464 <vfprintf+0x12c>
   12460:	fff00493          	li	s1,-1
   12464:	00fa6a33          	or	s4,s4,a5
   12468:	01442023          	sw	s4,0(s0)
   1246c:	00098663          	beqz	s3,12478 <vfprintf+0x140>
   12470:	00040513          	mv	a0,s0
   12474:	7d4000ef          	jal	ra,12c48 <__unlockfile>
   12478:	15813083          	ld	ra,344(sp)
   1247c:	15013403          	ld	s0,336(sp)
   12480:	14013903          	ld	s2,320(sp)
   12484:	13813983          	ld	s3,312(sp)
   12488:	13013a03          	ld	s4,304(sp)
   1248c:	12813a83          	ld	s5,296(sp)
   12490:	00048513          	mv	a0,s1
   12494:	14813483          	ld	s1,328(sp)
   12498:	16010113          	addi	sp,sp,352
   1249c:	00008067          	ret
   124a0:	02043903          	ld	s2,32(s0)
   124a4:	f40906e3          	beqz	s2,123f0 <vfprintf+0xb8>
   124a8:	00000913          	li	s2,0
   124ac:	f55ff06f          	j	12400 <vfprintf+0xc8>

00000000000124b0 <memset>:
   124b0:	0c060e63          	beqz	a2,1258c <memset+0xdc>
   124b4:	0ff5f793          	zext.b	a5,a1
   124b8:	00f50023          	sb	a5,0(a0)
   124bc:	00c50733          	add	a4,a0,a2
   124c0:	fef70fa3          	sb	a5,-1(a4) # 1fff <exit-0xe121>
   124c4:	00200693          	li	a3,2
   124c8:	0cc6f263          	bgeu	a3,a2,1258c <memset+0xdc>
   124cc:	00f500a3          	sb	a5,1(a0)
   124d0:	00f50123          	sb	a5,2(a0)
   124d4:	fef70f23          	sb	a5,-2(a4)
   124d8:	fef70ea3          	sb	a5,-3(a4)
   124dc:	00600693          	li	a3,6
   124e0:	0ac6f663          	bgeu	a3,a2,1258c <memset+0xdc>
   124e4:	00f501a3          	sb	a5,3(a0)
   124e8:	fef70e23          	sb	a5,-4(a4)
   124ec:	00800693          	li	a3,8
   124f0:	08c6fe63          	bgeu	a3,a2,1258c <memset+0xdc>
   124f4:	40a00733          	neg	a4,a0
   124f8:	00377713          	andi	a4,a4,3
   124fc:	00e507b3          	add	a5,a0,a4
   12500:	40e60633          	sub	a2,a2,a4
   12504:	01010737          	lui	a4,0x1010
   12508:	1017071b          	addiw	a4,a4,257
   1250c:	0ff5f593          	zext.b	a1,a1
   12510:	02e585bb          	mulw	a1,a1,a4
   12514:	ffc67613          	andi	a2,a2,-4
   12518:	00c78733          	add	a4,a5,a2
   1251c:	00b7a023          	sw	a1,0(a5)
   12520:	feb72e23          	sw	a1,-4(a4) # 100fffc <__global_pointer$+0xff77fc>
   12524:	06c6f463          	bgeu	a3,a2,1258c <memset+0xdc>
   12528:	00b7a223          	sw	a1,4(a5)
   1252c:	00b7a423          	sw	a1,8(a5)
   12530:	feb72a23          	sw	a1,-12(a4)
   12534:	feb72c23          	sw	a1,-8(a4)
   12538:	01800693          	li	a3,24
   1253c:	04c6f863          	bgeu	a3,a2,1258c <memset+0xdc>
   12540:	00b7a623          	sw	a1,12(a5)
   12544:	00b7a823          	sw	a1,16(a5)
   12548:	00b7aa23          	sw	a1,20(a5)
   1254c:	00b7ac23          	sw	a1,24(a5)
   12550:	feb72223          	sw	a1,-28(a4)
   12554:	feb72423          	sw	a1,-24(a4)
   12558:	feb72623          	sw	a1,-20(a4)
   1255c:	feb72823          	sw	a1,-16(a4)
   12560:	02059693          	slli	a3,a1,0x20
   12564:	0047f713          	andi	a4,a5,4
   12568:	01870713          	addi	a4,a4,24
   1256c:	0206d693          	srli	a3,a3,0x20
   12570:	02059593          	slli	a1,a1,0x20
   12574:	00e78733          	add	a4,a5,a4
   12578:	00c78633          	add	a2,a5,a2
   1257c:	00d5e5b3          	or	a1,a1,a3
   12580:	01f00793          	li	a5,31
   12584:	40e606b3          	sub	a3,a2,a4
   12588:	00d7e463          	bltu	a5,a3,12590 <memset+0xe0>
   1258c:	00008067          	ret
   12590:	00b73023          	sd	a1,0(a4)
   12594:	00b73423          	sd	a1,8(a4)
   12598:	00b73823          	sd	a1,16(a4)
   1259c:	00b73c23          	sd	a1,24(a4)
   125a0:	02070713          	addi	a4,a4,32
   125a4:	fe1ff06f          	j	12584 <memset+0xd4>

00000000000125a8 <strnlen>:
   125a8:	fe010113          	addi	sp,sp,-32
   125ac:	00813823          	sd	s0,16(sp)
   125b0:	00058613          	mv	a2,a1
   125b4:	00058413          	mv	s0,a1
   125b8:	00000593          	li	a1,0
   125bc:	00913423          	sd	s1,8(sp)
   125c0:	00113c23          	sd	ra,24(sp)
   125c4:	00050493          	mv	s1,a0
   125c8:	191000ef          	jal	ra,12f58 <memchr>
   125cc:	00050463          	beqz	a0,125d4 <strnlen+0x2c>
   125d0:	40950433          	sub	s0,a0,s1
   125d4:	01813083          	ld	ra,24(sp)
   125d8:	00040513          	mv	a0,s0
   125dc:	01013403          	ld	s0,16(sp)
   125e0:	00813483          	ld	s1,8(sp)
   125e4:	02010113          	addi	sp,sp,32
   125e8:	00008067          	ret

00000000000125ec <__clock_gettime>:
   125ec:	00050793          	mv	a5,a0
   125f0:	07100893          	li	a7,113
   125f4:	00000073          	ecall
   125f8:	fda00693          	li	a3,-38
   125fc:	0005051b          	sext.w	a0,a0
   12600:	02d51c63          	bne	a0,a3,12638 <__clock_gettime+0x4c>
   12604:	fea00513          	li	a0,-22
   12608:	02079863          	bnez	a5,12638 <__clock_gettime+0x4c>
   1260c:	00058713          	mv	a4,a1
   12610:	00058513          	mv	a0,a1
   12614:	0a900893          	li	a7,169
   12618:	00000593          	li	a1,0
   1261c:	00000073          	ecall
   12620:	00873603          	ld	a2,8(a4)
   12624:	3e800693          	li	a3,1000
   12628:	00078513          	mv	a0,a5
   1262c:	02c686bb          	mulw	a3,a3,a2
   12630:	00d73423          	sd	a3,8(a4)
   12634:	00008067          	ret
   12638:	ff010113          	addi	sp,sp,-16
   1263c:	00113423          	sd	ra,8(sp)
   12640:	390000ef          	jal	ra,129d0 <__syscall_ret>
   12644:	00813083          	ld	ra,8(sp)
   12648:	0005079b          	sext.w	a5,a0
   1264c:	00078513          	mv	a0,a5
   12650:	01010113          	addi	sp,sp,16
   12654:	00008067          	ret

0000000000012658 <__init_tp>:
   12658:	ff010113          	addi	sp,sp,-16
   1265c:	00813023          	sd	s0,0(sp)
   12660:	00113423          	sd	ra,8(sp)
   12664:	00050413          	mv	s0,a0
   12668:	00a53023          	sd	a0,0(a0)
   1266c:	0e050513          	addi	a0,a0,224
   12670:	5c1000ef          	jal	ra,13430 <__set_thread_area>
   12674:	04054e63          	bltz	a0,126d0 <__init_tp+0x78>
   12678:	00051663          	bnez	a0,12684 <__init_tp+0x2c>
   1267c:	00100713          	li	a4,1
   12680:	9ce1a023          	sw	a4,-1600(gp) # 181c0 <__libc>
   12684:	00100793          	li	a5,1
   12688:	04f42023          	sw	a5,64(s0)
   1268c:	06000893          	li	a7,96
   12690:	96818513          	addi	a0,gp,-1688 # 18168 <__thread_list_lock>
   12694:	00000073          	ecall
   12698:	a0018793          	addi	a5,gp,-1536 # 18200 <__libc+0x40>
   1269c:	0af43823          	sd	a5,176(s0)
   126a0:	09040793          	addi	a5,s0,144
   126a4:	08f43823          	sd	a5,144(s0)
   126a8:	9401b783          	ld	a5,-1728(gp) # 18140 <__sysinfo>
   126ac:	02a42c23          	sw	a0,56(s0)
   126b0:	00843823          	sd	s0,16(s0)
   126b4:	02f43023          	sd	a5,32(s0)
   126b8:	00843c23          	sd	s0,24(s0)
   126bc:	00000513          	li	a0,0
   126c0:	00813083          	ld	ra,8(sp)
   126c4:	00013403          	ld	s0,0(sp)
   126c8:	01010113          	addi	sp,sp,16
   126cc:	00008067          	ret
   126d0:	fff00513          	li	a0,-1
   126d4:	fedff06f          	j	126c0 <__init_tp+0x68>

00000000000126d8 <__copy_tls>:
   126d8:	fc010113          	addi	sp,sp,-64
   126dc:	9c018713          	addi	a4,gp,-1600 # 181c0 <__libc>
   126e0:	03073683          	ld	a3,48(a4)
   126e4:	02913423          	sd	s1,40(sp)
   126e8:	02073483          	ld	s1,32(a4)
   126ec:	02813823          	sd	s0,48(sp)
   126f0:	02873403          	ld	s0,40(a4)
   126f4:	00369693          	slli	a3,a3,0x3
   126f8:	ff848493          	addi	s1,s1,-8
   126fc:	40d484b3          	sub	s1,s1,a3
   12700:	f2000693          	li	a3,-224
   12704:	fff40413          	addi	s0,s0,-1
   12708:	40a686b3          	sub	a3,a3,a0
   1270c:	03213023          	sd	s2,32(sp)
   12710:	00d47433          	and	s0,s0,a3
   12714:	01873903          	ld	s2,24(a4)
   12718:	01513423          	sd	s5,8(sp)
   1271c:	009504b3          	add	s1,a0,s1
   12720:	00850433          	add	s0,a0,s0
   12724:	00001ab7          	lui	s5,0x1
   12728:	01313c23          	sd	s3,24(sp)
   1272c:	01413823          	sd	s4,16(sp)
   12730:	01613023          	sd	s6,0(sp)
   12734:	02113c23          	sd	ra,56(sp)
   12738:	0e040b13          	addi	s6,s0,224
   1273c:	00848a13          	addi	s4,s1,8
   12740:	9c018993          	addi	s3,gp,-1600 # 181c0 <__libc>
   12744:	800a8a93          	addi	s5,s5,-2048 # 800 <exit-0xf920>
   12748:	04091063          	bnez	s2,12788 <__copy_tls+0xb0>
   1274c:	0309b783          	ld	a5,48(s3)
   12750:	03813083          	ld	ra,56(sp)
   12754:	00040513          	mv	a0,s0
   12758:	00f4b023          	sd	a5,0(s1)
   1275c:	0c943c23          	sd	s1,216(s0)
   12760:	00943423          	sd	s1,8(s0)
   12764:	03013403          	ld	s0,48(sp)
   12768:	02813483          	ld	s1,40(sp)
   1276c:	02013903          	ld	s2,32(sp)
   12770:	01813983          	ld	s3,24(sp)
   12774:	01013a03          	ld	s4,16(sp)
   12778:	00813a83          	ld	s5,8(sp)
   1277c:	00013b03          	ld	s6,0(sp)
   12780:	04010113          	addi	sp,sp,64
   12784:	00008067          	ret
   12788:	02893783          	ld	a5,40(s2)
   1278c:	00893583          	ld	a1,8(s2)
   12790:	008a0a13          	addi	s4,s4,8
   12794:	00fb07b3          	add	a5,s6,a5
   12798:	015787b3          	add	a5,a5,s5
   1279c:	fefa3c23          	sd	a5,-8(s4)
   127a0:	02893503          	ld	a0,40(s2)
   127a4:	01093603          	ld	a2,16(s2)
   127a8:	00ab0533          	add	a0,s6,a0
   127ac:	041000ef          	jal	ra,12fec <memcpy>
   127b0:	00093903          	ld	s2,0(s2)
   127b4:	f95ff06f          	j	12748 <__copy_tls+0x70>

00000000000127b8 <__init_tls>:
   127b8:	01853f83          	ld	t6,24(a0)
   127bc:	02853e03          	ld	t3,40(a0)
   127c0:	93c1a303          	lw	t1,-1732(gp) # 1813c <__default_stacksize>
   127c4:	ff010113          	addi	sp,sp,-16
   127c8:	6474eeb7          	lui	t4,0x6474e
   127cc:	00113423          	sd	ra,8(sp)
   127d0:	00813023          	sd	s0,0(sp)
   127d4:	000f8793          	mv	a5,t6
   127d8:	00000f13          	li	t5,0
   127dc:	00000713          	li	a4,0
   127e0:	00000613          	li	a2,0
   127e4:	00600293          	li	t0,6
   127e8:	00200393          	li	t2,2
   127ec:	00000813          	li	a6,0
   127f0:	00700093          	li	ra,7
   127f4:	551e8e93          	addi	t4,t4,1361 # 6474e551 <__global_pointer$+0x64735d51>
   127f8:	00800437          	lui	s0,0x800
   127fc:	0c0e1c63          	bnez	t3,128d4 <__init_tls+0x11c>
   12800:	000f0463          	beqz	t5,12808 <__init_tls+0x50>
   12804:	9261ae23          	sw	t1,-1732(gp) # 1813c <__default_stacksize>
   12808:	fa018793          	addi	a5,gp,-96 # 187a0 <main_tls>
   1280c:	9c018693          	addi	a3,gp,-1600 # 181c0 <__libc>
   12810:	02060a63          	beqz	a2,12844 <__init_tls+0x8c>
   12814:	01063583          	ld	a1,16(a2)
   12818:	00f6bc23          	sd	a5,24(a3)
   1281c:	00e58733          	add	a4,a1,a4
   12820:	00e7b423          	sd	a4,8(a5)
   12824:	02063703          	ld	a4,32(a2)
   12828:	00e7b823          	sd	a4,16(a5)
   1282c:	02863703          	ld	a4,40(a2)
   12830:	00e7bc23          	sd	a4,24(a5)
   12834:	03063703          	ld	a4,48(a2)
   12838:	02e7b023          	sd	a4,32(a5)
   1283c:	00100713          	li	a4,1
   12840:	02e6b823          	sd	a4,48(a3)
   12844:	0187b803          	ld	a6,24(a5)
   12848:	0087b583          	ld	a1,8(a5)
   1284c:	0207b503          	ld	a0,32(a5)
   12850:	00b80733          	add	a4,a6,a1
   12854:	fff50613          	addi	a2,a0,-1
   12858:	40e00733          	neg	a4,a4
   1285c:	00c77733          	and	a4,a4,a2
   12860:	01070733          	add	a4,a4,a6
   12864:	00c5f633          	and	a2,a1,a2
   12868:	00e7bc23          	sd	a4,24(a5)
   1286c:	02c7b423          	sd	a2,40(a5)
   12870:	00700593          	li	a1,7
   12874:	00a5e663          	bltu	a1,a0,12880 <__init_tls+0xc8>
   12878:	00800593          	li	a1,8
   1287c:	02b7b023          	sd	a1,32(a5)
   12880:	0207b783          	ld	a5,32(a5)
   12884:	0f778593          	addi	a1,a5,247
   12888:	00c585b3          	add	a1,a1,a2
   1288c:	00e585b3          	add	a1,a1,a4
   12890:	ff85f593          	andi	a1,a1,-8
   12894:	02f6b423          	sd	a5,40(a3)
   12898:	02b6b023          	sd	a1,32(a3)
   1289c:	16800793          	li	a5,360
   128a0:	08b7fe63          	bgeu	a5,a1,1293c <__init_tls+0x184>
   128a4:	0de00893          	li	a7,222
   128a8:	00000513          	li	a0,0
   128ac:	00300613          	li	a2,3
   128b0:	02200693          	li	a3,34
   128b4:	fff00713          	li	a4,-1
   128b8:	00000793          	li	a5,0
   128bc:	00000073          	ecall
   128c0:	e19ff0ef          	jal	ra,126d8 <__copy_tls>
   128c4:	d95ff0ef          	jal	ra,12658 <__init_tp>
   128c8:	06055e63          	bgez	a0,12944 <__init_tls+0x18c>
   128cc:	00000023          	sb	zero,0(zero) # 0 <exit-0x10120>
   128d0:	00100073          	ebreak
   128d4:	0007a883          	lw	a7,0(a5)
   128d8:	00589e63          	bne	a7,t0,128f4 <__init_tls+0x13c>
   128dc:	0107b703          	ld	a4,16(a5)
   128e0:	40ef8733          	sub	a4,t6,a4
   128e4:	02053683          	ld	a3,32(a0)
   128e8:	fffe0e13          	addi	t3,t3,-1
   128ec:	00d787b3          	add	a5,a5,a3
   128f0:	f0dff06f          	j	127fc <__init_tls+0x44>
   128f4:	02788863          	beq	a7,t2,12924 <__init_tls+0x16c>
   128f8:	02188e63          	beq	a7,ra,12934 <__init_tls+0x17c>
   128fc:	ffd894e3          	bne	a7,t4,128e4 <__init_tls+0x12c>
   12900:	0287b683          	ld	a3,40(a5)
   12904:	02031893          	slli	a7,t1,0x20
   12908:	0208d893          	srli	a7,a7,0x20
   1290c:	fcd8fce3          	bgeu	a7,a3,128e4 <__init_tls+0x12c>
   12910:	00d47463          	bgeu	s0,a3,12918 <__init_tls+0x160>
   12914:	008006b7          	lui	a3,0x800
   12918:	0006831b          	sext.w	t1,a3
   1291c:	00100f13          	li	t5,1
   12920:	fc5ff06f          	j	128e4 <__init_tls+0x12c>
   12924:	fc0800e3          	beqz	a6,128e4 <__init_tls+0x12c>
   12928:	0107b703          	ld	a4,16(a5)
   1292c:	40e80733          	sub	a4,a6,a4
   12930:	fb5ff06f          	j	128e4 <__init_tls+0x12c>
   12934:	00078613          	mv	a2,a5
   12938:	fadff06f          	j	128e4 <__init_tls+0x12c>
   1293c:	e3818513          	addi	a0,gp,-456 # 18638 <builtin_tls>
   12940:	f81ff06f          	j	128c0 <__init_tls+0x108>
   12944:	00813083          	ld	ra,8(sp)
   12948:	00013403          	ld	s0,0(sp)
   1294c:	01010113          	addi	sp,sp,16
   12950:	00008067          	ret

0000000000012954 <__errno_location>:
   12954:	00020513          	mv	a0,tp
   12958:	f5c50513          	addi	a0,a0,-164
   1295c:	00008067          	ret

0000000000012960 <__strerror_l>:
   12960:	000166b7          	lui	a3,0x16
   12964:	00000793          	li	a5,0
   12968:	05068693          	addi	a3,a3,80 # 16050 <errid>
   1296c:	00f68633          	add	a2,a3,a5
   12970:	00064603          	lbu	a2,0(a2)
   12974:	0007871b          	sext.w	a4,a5
   12978:	00060663          	beqz	a2,12984 <__strerror_l+0x24>
   1297c:	00178793          	addi	a5,a5,1
   12980:	fea616e3          	bne	a2,a0,1296c <__strerror_l+0xc>
   12984:	00016537          	lui	a0,0x16
   12988:	0b050513          	addi	a0,a0,176 # 160b0 <errmsg>
   1298c:	0140006f          	j	129a0 <__strerror_l+0x40>
   12990:	00054783          	lbu	a5,0(a0)
   12994:	00150513          	addi	a0,a0,1
   12998:	fe079ce3          	bnez	a5,12990 <__strerror_l+0x30>
   1299c:	fff7071b          	addiw	a4,a4,-1
   129a0:	fe0718e3          	bnez	a4,12990 <__strerror_l+0x30>
   129a4:	0285b583          	ld	a1,40(a1) # ffffffff80000028 <__global_pointer$+0xffffffff7ffe7828>
   129a8:	0640006f          	j	12a0c <__lctrans>

00000000000129ac <strerror>:
   129ac:	00020793          	mv	a5,tp
   129b0:	fd07b583          	ld	a1,-48(a5)
   129b4:	fadff06f          	j	12960 <__strerror_l>

00000000000129b8 <_Exit>:
   129b8:	00050793          	mv	a5,a0
   129bc:	05e00893          	li	a7,94
   129c0:	00000073          	ecall
   129c4:	05d00893          	li	a7,93
   129c8:	00078513          	mv	a0,a5
   129cc:	ff5ff06f          	j	129c0 <_Exit+0x8>

00000000000129d0 <__syscall_ret>:
   129d0:	ff010113          	addi	sp,sp,-16
   129d4:	00813023          	sd	s0,0(sp)
   129d8:	00113423          	sd	ra,8(sp)
   129dc:	fffff7b7          	lui	a5,0xfffff
   129e0:	00050413          	mv	s0,a0
   129e4:	00a7fa63          	bgeu	a5,a0,129f8 <__syscall_ret+0x28>
   129e8:	f6dff0ef          	jal	ra,12954 <__errno_location>
   129ec:	4080043b          	negw	s0,s0
   129f0:	00852023          	sw	s0,0(a0)
   129f4:	fff00513          	li	a0,-1
   129f8:	00813083          	ld	ra,8(sp)
   129fc:	00013403          	ld	s0,0(sp)
   12a00:	01010113          	addi	sp,sp,16
   12a04:	00008067          	ret

0000000000012a08 <__lctrans_impl>:
   12a08:	00008067          	ret

0000000000012a0c <__lctrans>:
   12a0c:	ffdff06f          	j	12a08 <__lctrans_impl>

0000000000012a10 <__lctrans_cur>:
   12a10:	00020793          	mv	a5,tp
   12a14:	fd07b783          	ld	a5,-48(a5) # ffffffffffffefd0 <__global_pointer$+0xfffffffffffe67d0>
   12a18:	0287b583          	ld	a1,40(a5)
   12a1c:	fedff06f          	j	12a08 <__lctrans_impl>

0000000000012a20 <__fpclassifyl>:
   12a20:	00008737          	lui	a4,0x8
   12a24:	0305d693          	srli	a3,a1,0x30
   12a28:	fff70713          	addi	a4,a4,-1 # 7fff <exit-0x8121>
   12a2c:	01059593          	slli	a1,a1,0x10
   12a30:	00e6f6b3          	and	a3,a3,a4
   12a34:	00050793          	mv	a5,a0
   12a38:	0105d593          	srli	a1,a1,0x10
   12a3c:	00069a63          	bnez	a3,12a50 <__fpclassifyl+0x30>
   12a40:	00a5e5b3          	or	a1,a1,a0
   12a44:	00b03533          	snez	a0,a1
   12a48:	00250513          	addi	a0,a0,2
   12a4c:	00008067          	ret
   12a50:	00400513          	li	a0,4
   12a54:	00e69663          	bne	a3,a4,12a60 <__fpclassifyl+0x40>
   12a58:	00f5e5b3          	or	a1,a1,a5
   12a5c:	0015b513          	seqz	a0,a1
   12a60:	00008067          	ret

0000000000012a64 <__signbitl>:
   12a64:	03f5d513          	srli	a0,a1,0x3f
   12a68:	00008067          	ret

0000000000012a6c <frexpl>:
   12a6c:	00008737          	lui	a4,0x8
   12a70:	fe010113          	addi	sp,sp,-32
   12a74:	0305d793          	srli	a5,a1,0x30
   12a78:	fff70713          	addi	a4,a4,-1 # 7fff <exit-0x8121>
   12a7c:	00813823          	sd	s0,16(sp)
   12a80:	00913423          	sd	s1,8(sp)
   12a84:	01213023          	sd	s2,0(sp)
   12a88:	00113c23          	sd	ra,24(sp)
   12a8c:	00060913          	mv	s2,a2
   12a90:	00e7f633          	and	a2,a5,a4
   12a94:	00050493          	mv	s1,a0
   12a98:	00058413          	mv	s0,a1
   12a9c:	06061663          	bnez	a2,12b08 <frexpl+0x9c>
   12aa0:	00000693          	li	a3,0
   12aa4:	141010ef          	jal	ra,143e4 <__eqtf2>
   12aa8:	04050c63          	beqz	a0,12b00 <frexpl+0x94>
   12aac:	000167b7          	lui	a5,0x16
   12ab0:	7d87b683          	ld	a3,2008(a5) # 167d8 <errmsg+0x728>
   12ab4:	00000613          	li	a2,0
   12ab8:	00048513          	mv	a0,s1
   12abc:	00040593          	mv	a1,s0
   12ac0:	1f5010ef          	jal	ra,144b4 <__multf3>
   12ac4:	00090613          	mv	a2,s2
   12ac8:	fa5ff0ef          	jal	ra,12a6c <frexpl>
   12acc:	00092783          	lw	a5,0(s2)
   12ad0:	00050493          	mv	s1,a0
   12ad4:	00058413          	mv	s0,a1
   12ad8:	f887879b          	addiw	a5,a5,-120
   12adc:	00f92023          	sw	a5,0(s2)
   12ae0:	01813083          	ld	ra,24(sp)
   12ae4:	00040593          	mv	a1,s0
   12ae8:	01013403          	ld	s0,16(sp)
   12aec:	00013903          	ld	s2,0(sp)
   12af0:	00048513          	mv	a0,s1
   12af4:	00813483          	ld	s1,8(sp)
   12af8:	02010113          	addi	sp,sp,32
   12afc:	00008067          	ret
   12b00:	00092023          	sw	zero,0(s2)
   12b04:	fddff06f          	j	12ae0 <frexpl+0x74>
   12b08:	0006069b          	sext.w	a3,a2
   12b0c:	fce60ae3          	beq	a2,a4,12ae0 <frexpl+0x74>
   12b10:	ffffc737          	lui	a4,0xffffc
   12b14:	0027071b          	addiw	a4,a4,2
   12b18:	00e6873b          	addw	a4,a3,a4
   12b1c:	00e92023          	sw	a4,0(s2)
   12b20:	ffff8737          	lui	a4,0xffff8
   12b24:	00e7f7b3          	and	a5,a5,a4
   12b28:	00004737          	lui	a4,0x4
   12b2c:	ffe70713          	addi	a4,a4,-2 # 3ffe <exit-0xc122>
   12b30:	00e7e7b3          	or	a5,a5,a4
   12b34:	01059413          	slli	s0,a1,0x10
   12b38:	03079793          	slli	a5,a5,0x30
   12b3c:	01045413          	srli	s0,s0,0x10
   12b40:	00f46433          	or	s0,s0,a5
   12b44:	f9dff06f          	j	12ae0 <frexpl+0x74>

0000000000012b48 <wctomb>:
   12b48:	02050263          	beqz	a0,12b6c <wctomb+0x24>
   12b4c:	ff010113          	addi	sp,sp,-16
   12b50:	00000613          	li	a2,0
   12b54:	00113423          	sd	ra,8(sp)
   12b58:	0f1000ef          	jal	ra,13448 <wcrtomb>
   12b5c:	00813083          	ld	ra,8(sp)
   12b60:	0005051b          	sext.w	a0,a0
   12b64:	01010113          	addi	sp,sp,16
   12b68:	00008067          	ret
   12b6c:	00000513          	li	a0,0
   12b70:	00008067          	ret

0000000000012b74 <__lockfile>:
   12b74:	08c52783          	lw	a5,140(a0)
   12b78:	0007879b          	sext.w	a5,a5
   12b7c:	00020713          	mv	a4,tp
   12b80:	c00006b7          	lui	a3,0xc0000
   12b84:	f5872703          	lw	a4,-168(a4)
   12b88:	fff68693          	addi	a3,a3,-1 # ffffffffbfffffff <__global_pointer$+0xffffffffbffe77ff>
   12b8c:	00d7f7b3          	and	a5,a5,a3
   12b90:	0ae78863          	beq	a5,a4,12c40 <__lockfile+0xcc>
   12b94:	08c50793          	addi	a5,a0,140
   12b98:	00000613          	li	a2,0
   12b9c:	1607a6af          	lr.w.aqrl	a3,(a5)
   12ba0:	00c69663          	bne	a3,a2,12bac <__lockfile+0x38>
   12ba4:	1ee7a5af          	sc.w.aqrl	a1,a4,(a5)
   12ba8:	fe059ae3          	bnez	a1,12b9c <__lockfile+0x28>
   12bac:	0006869b          	sext.w	a3,a3
   12bb0:	02068863          	beqz	a3,12be0 <__lockfile+0x6c>
   12bb4:	40000837          	lui	a6,0x40000
   12bb8:	01076733          	or	a4,a4,a6
   12bbc:	00000313          	li	t1,0
   12bc0:	0007071b          	sext.w	a4,a4
   12bc4:	fda00e13          	li	t3,-38
   12bc8:	1607a6af          	lr.w.aqrl	a3,(a5)
   12bcc:	00669663          	bne	a3,t1,12bd8 <__lockfile+0x64>
   12bd0:	1ee7a62f          	sc.w.aqrl	a2,a4,(a5)
   12bd4:	fe061ae3          	bnez	a2,12bc8 <__lockfile+0x54>
   12bd8:	0006869b          	sext.w	a3,a3
   12bdc:	00069663          	bnez	a3,12be8 <__lockfile+0x74>
   12be0:	00100513          	li	a0,1
   12be4:	00008067          	ret
   12be8:	0106f5b3          	and	a1,a3,a6
   12bec:	0106e633          	or	a2,a3,a6
   12bf0:	0005859b          	sext.w	a1,a1
   12bf4:	0006061b          	sext.w	a2,a2
   12bf8:	02058663          	beqz	a1,12c24 <__lockfile+0xb0>
   12bfc:	06200893          	li	a7,98
   12c00:	00078513          	mv	a0,a5
   12c04:	08000593          	li	a1,128
   12c08:	00000693          	li	a3,0
   12c0c:	00000073          	ecall
   12c10:	fbc51ce3          	bne	a0,t3,12bc8 <__lockfile+0x54>
   12c14:	00078513          	mv	a0,a5
   12c18:	00000593          	li	a1,0
   12c1c:	00000073          	ecall
   12c20:	fa9ff06f          	j	12bc8 <__lockfile+0x54>
   12c24:	1607a5af          	lr.w.aqrl	a1,(a5)
   12c28:	00d59663          	bne	a1,a3,12c34 <__lockfile+0xc0>
   12c2c:	1ec7a52f          	sc.w.aqrl	a0,a2,(a5)
   12c30:	fe051ae3          	bnez	a0,12c24 <__lockfile+0xb0>
   12c34:	0005859b          	sext.w	a1,a1
   12c38:	f8d598e3          	bne	a1,a3,12bc8 <__lockfile+0x54>
   12c3c:	fc1ff06f          	j	12bfc <__lockfile+0x88>
   12c40:	00000513          	li	a0,0
   12c44:	00008067          	ret

0000000000012c48 <__unlockfile>:
   12c48:	08c50713          	addi	a4,a0,140
   12c4c:	00000613          	li	a2,0
   12c50:	08c52783          	lw	a5,140(a0)
   12c54:	0007879b          	sext.w	a5,a5
   12c58:	160726af          	lr.w.aqrl	a3,(a4)
   12c5c:	00f69663          	bne	a3,a5,12c68 <__unlockfile+0x20>
   12c60:	1ec725af          	sc.w.aqrl	a1,a2,(a4)
   12c64:	fe059ae3          	bnez	a1,12c58 <__unlockfile+0x10>
   12c68:	0006869b          	sext.w	a3,a3
   12c6c:	fed792e3          	bne	a5,a3,12c50 <__unlockfile+0x8>
   12c70:	01e7d793          	srli	a5,a5,0x1e
   12c74:	0017f793          	andi	a5,a5,1
   12c78:	02078663          	beqz	a5,12ca4 <__unlockfile+0x5c>
   12c7c:	06200893          	li	a7,98
   12c80:	00070513          	mv	a0,a4
   12c84:	08100593          	li	a1,129
   12c88:	00100613          	li	a2,1
   12c8c:	00000073          	ecall
   12c90:	fda00793          	li	a5,-38
   12c94:	00f51863          	bne	a0,a5,12ca4 <__unlockfile+0x5c>
   12c98:	00070513          	mv	a0,a4
   12c9c:	00100593          	li	a1,1
   12ca0:	00000073          	ecall
   12ca4:	00008067          	ret

0000000000012ca8 <__aio_close>:
   12ca8:	00008067          	ret

0000000000012cac <__stdio_close>:
   12cac:	07852503          	lw	a0,120(a0)
   12cb0:	ff010113          	addi	sp,sp,-16
   12cb4:	00113423          	sd	ra,8(sp)
   12cb8:	ff1ff0ef          	jal	ra,12ca8 <__aio_close>
   12cbc:	03900893          	li	a7,57
   12cc0:	00000073          	ecall
   12cc4:	d0dff0ef          	jal	ra,129d0 <__syscall_ret>
   12cc8:	00813083          	ld	ra,8(sp)
   12ccc:	0005051b          	sext.w	a0,a0
   12cd0:	01010113          	addi	sp,sp,16
   12cd4:	00008067          	ret

0000000000012cd8 <__stdio_seek>:
   12cd8:	07852503          	lw	a0,120(a0)
   12cdc:	7600006f          	j	1343c <__lseek>

0000000000012ce0 <__stdout_write>:
   12ce0:	fe010113          	addi	sp,sp,-32
   12ce4:	00060693          	mv	a3,a2
   12ce8:	00013637          	lui	a2,0x13
   12cec:	64860613          	addi	a2,a2,1608 # 13648 <__stdio_write>
   12cf0:	00113c23          	sd	ra,24(sp)
   12cf4:	04c53423          	sd	a2,72(a0)
   12cf8:	00052603          	lw	a2,0(a0)
   12cfc:	00050793          	mv	a5,a0
   12d00:	00058713          	mv	a4,a1
   12d04:	04067613          	andi	a2,a2,64
   12d08:	02061463          	bnez	a2,12d30 <__stdout_write+0x50>
   12d0c:	000055b7          	lui	a1,0x5
   12d10:	07852503          	lw	a0,120(a0)
   12d14:	01d00893          	li	a7,29
   12d18:	41358593          	addi	a1,a1,1043 # 5413 <exit-0xad0d>
   12d1c:	00810613          	addi	a2,sp,8
   12d20:	00000073          	ecall
   12d24:	00050663          	beqz	a0,12d30 <__stdout_write+0x50>
   12d28:	fff00613          	li	a2,-1
   12d2c:	08c7a823          	sw	a2,144(a5)
   12d30:	00068613          	mv	a2,a3
   12d34:	00070593          	mv	a1,a4
   12d38:	00078513          	mv	a0,a5
   12d3c:	10d000ef          	jal	ra,13648 <__stdio_write>
   12d40:	01813083          	ld	ra,24(sp)
   12d44:	02010113          	addi	sp,sp,32
   12d48:	00008067          	ret

0000000000012d4c <__towrite>:
   12d4c:	08852783          	lw	a5,136(a0)
   12d50:	fff7871b          	addiw	a4,a5,-1
   12d54:	00e7e7b3          	or	a5,a5,a4
   12d58:	08f52423          	sw	a5,136(a0)
   12d5c:	00052783          	lw	a5,0(a0)
   12d60:	0087f713          	andi	a4,a5,8
   12d64:	00070a63          	beqz	a4,12d78 <__towrite+0x2c>
   12d68:	0207e793          	ori	a5,a5,32
   12d6c:	00f52023          	sw	a5,0(a0)
   12d70:	fff00513          	li	a0,-1
   12d74:	00008067          	ret
   12d78:	05853783          	ld	a5,88(a0)
   12d7c:	06053703          	ld	a4,96(a0)
   12d80:	00053823          	sd	zero,16(a0)
   12d84:	02f53c23          	sd	a5,56(a0)
   12d88:	02f53423          	sd	a5,40(a0)
   12d8c:	00e787b3          	add	a5,a5,a4
   12d90:	00053423          	sd	zero,8(a0)
   12d94:	02f53023          	sd	a5,32(a0)
   12d98:	00000513          	li	a0,0
   12d9c:	00008067          	ret

0000000000012da0 <__towrite_needs_stdio_exit>:
   12da0:	05d0006f          	j	135fc <__stdio_exit>

0000000000012da4 <__fwritex>:
   12da4:	02063783          	ld	a5,32(a2)
   12da8:	fd010113          	addi	sp,sp,-48
   12dac:	02813023          	sd	s0,32(sp)
   12db0:	01213823          	sd	s2,16(sp)
   12db4:	01313423          	sd	s3,8(sp)
   12db8:	02113423          	sd	ra,40(sp)
   12dbc:	00913c23          	sd	s1,24(sp)
   12dc0:	00050993          	mv	s3,a0
   12dc4:	00058913          	mv	s2,a1
   12dc8:	00060413          	mv	s0,a2
   12dcc:	04078063          	beqz	a5,12e0c <__fwritex+0x68>
   12dd0:	02043783          	ld	a5,32(s0) # 800020 <__global_pointer$+0x7e7820>
   12dd4:	02843703          	ld	a4,40(s0)
   12dd8:	40e787b3          	sub	a5,a5,a4
   12ddc:	0527f263          	bgeu	a5,s2,12e20 <__fwritex+0x7c>
   12de0:	04843783          	ld	a5,72(s0)
   12de4:	00040513          	mv	a0,s0
   12de8:	02013403          	ld	s0,32(sp)
   12dec:	02813083          	ld	ra,40(sp)
   12df0:	01813483          	ld	s1,24(sp)
   12df4:	00090613          	mv	a2,s2
   12df8:	00098593          	mv	a1,s3
   12dfc:	01013903          	ld	s2,16(sp)
   12e00:	00813983          	ld	s3,8(sp)
   12e04:	03010113          	addi	sp,sp,48
   12e08:	00078067          	jr	a5
   12e0c:	00060513          	mv	a0,a2
   12e10:	f3dff0ef          	jal	ra,12d4c <__towrite>
   12e14:	fa050ee3          	beqz	a0,12dd0 <__fwritex+0x2c>
   12e18:	00000513          	li	a0,0
   12e1c:	0340006f          	j	12e50 <__fwritex+0xac>
   12e20:	09042783          	lw	a5,144(s0)
   12e24:	00090493          	mv	s1,s2
   12e28:	00a00713          	li	a4,10
   12e2c:	0407d263          	bgez	a5,12e70 <__fwritex+0xcc>
   12e30:	02843503          	ld	a0,40(s0)
   12e34:	00048613          	mv	a2,s1
   12e38:	00098593          	mv	a1,s3
   12e3c:	1b0000ef          	jal	ra,12fec <memcpy>
   12e40:	02843603          	ld	a2,40(s0)
   12e44:	00090513          	mv	a0,s2
   12e48:	009604b3          	add	s1,a2,s1
   12e4c:	02943423          	sd	s1,40(s0)
   12e50:	02813083          	ld	ra,40(sp)
   12e54:	02013403          	ld	s0,32(sp)
   12e58:	01813483          	ld	s1,24(sp)
   12e5c:	01013903          	ld	s2,16(sp)
   12e60:	00813983          	ld	s3,8(sp)
   12e64:	03010113          	addi	sp,sp,48
   12e68:	00008067          	ret
   12e6c:	00078493          	mv	s1,a5
   12e70:	02048c63          	beqz	s1,12ea8 <__fwritex+0x104>
   12e74:	fff48793          	addi	a5,s1,-1
   12e78:	00f986b3          	add	a3,s3,a5
   12e7c:	0006c683          	lbu	a3,0(a3)
   12e80:	fee696e3          	bne	a3,a4,12e6c <__fwritex+0xc8>
   12e84:	04843783          	ld	a5,72(s0)
   12e88:	00048613          	mv	a2,s1
   12e8c:	00098593          	mv	a1,s3
   12e90:	00040513          	mv	a0,s0
   12e94:	000780e7          	jalr	a5
   12e98:	fa956ce3          	bltu	a0,s1,12e50 <__fwritex+0xac>
   12e9c:	009989b3          	add	s3,s3,s1
   12ea0:	409904b3          	sub	s1,s2,s1
   12ea4:	f8dff06f          	j	12e30 <__fwritex+0x8c>
   12ea8:	00090493          	mv	s1,s2
   12eac:	f85ff06f          	j	12e30 <__fwritex+0x8c>

0000000000012eb0 <fwrite>:
   12eb0:	fc010113          	addi	sp,sp,-64
   12eb4:	02813823          	sd	s0,48(sp)
   12eb8:	02913423          	sd	s1,40(sp)
   12ebc:	03213023          	sd	s2,32(sp)
   12ec0:	01313c23          	sd	s3,24(sp)
   12ec4:	01513423          	sd	s5,8(sp)
   12ec8:	02113c23          	sd	ra,56(sp)
   12ecc:	01413823          	sd	s4,16(sp)
   12ed0:	02c58ab3          	mul	s5,a1,a2
   12ed4:	00050993          	mv	s3,a0
   12ed8:	00058913          	mv	s2,a1
   12edc:	00068493          	mv	s1,a3
   12ee0:	00000413          	li	s0,0
   12ee4:	00058463          	beqz	a1,12eec <fwrite+0x3c>
   12ee8:	00060413          	mv	s0,a2
   12eec:	08c4a783          	lw	a5,140(s1)
   12ef0:	00000a13          	li	s4,0
   12ef4:	0007879b          	sext.w	a5,a5
   12ef8:	0007c863          	bltz	a5,12f08 <fwrite+0x58>
   12efc:	00048513          	mv	a0,s1
   12f00:	c75ff0ef          	jal	ra,12b74 <__lockfile>
   12f04:	00050a13          	mv	s4,a0
   12f08:	00098513          	mv	a0,s3
   12f0c:	00048613          	mv	a2,s1
   12f10:	000a8593          	mv	a1,s5
   12f14:	e91ff0ef          	jal	ra,12da4 <__fwritex>
   12f18:	00050993          	mv	s3,a0
   12f1c:	000a0663          	beqz	s4,12f28 <fwrite+0x78>
   12f20:	00048513          	mv	a0,s1
   12f24:	d25ff0ef          	jal	ra,12c48 <__unlockfile>
   12f28:	013a8463          	beq	s5,s3,12f30 <fwrite+0x80>
   12f2c:	0329d433          	divu	s0,s3,s2
   12f30:	03813083          	ld	ra,56(sp)
   12f34:	00040513          	mv	a0,s0
   12f38:	03013403          	ld	s0,48(sp)
   12f3c:	02813483          	ld	s1,40(sp)
   12f40:	02013903          	ld	s2,32(sp)
   12f44:	01813983          	ld	s3,24(sp)
   12f48:	01013a03          	ld	s4,16(sp)
   12f4c:	00813a83          	ld	s5,8(sp)
   12f50:	04010113          	addi	sp,sp,64
   12f54:	00008067          	ret

0000000000012f58 <memchr>:
   12f58:	0ff5f593          	zext.b	a1,a1
   12f5c:	00757793          	andi	a5,a0,7
   12f60:	02078863          	beqz	a5,12f90 <memchr+0x38>
   12f64:	08060063          	beqz	a2,12fe4 <memchr+0x8c>
   12f68:	00054783          	lbu	a5,0(a0)
   12f6c:	00b79c63          	bne	a5,a1,12f84 <memchr+0x2c>
   12f70:	00c50633          	add	a2,a0,a2
   12f74:	06c50863          	beq	a0,a2,12fe4 <memchr+0x8c>
   12f78:	00054783          	lbu	a5,0(a0)
   12f7c:	06b79063          	bne	a5,a1,12fdc <memchr+0x84>
   12f80:	00008067          	ret
   12f84:	00150513          	addi	a0,a0,1
   12f88:	fff60613          	addi	a2,a2,-1
   12f8c:	fd1ff06f          	j	12f5c <memchr+0x4>
   12f90:	04060a63          	beqz	a2,12fe4 <memchr+0x8c>
   12f94:	00054783          	lbu	a5,0(a0)
   12f98:	fcb78ce3          	beq	a5,a1,12f70 <memchr+0x18>
   12f9c:	9081b683          	ld	a3,-1784(gp) # 18108 <__SDATA_BEGIN__>
   12fa0:	9101b883          	ld	a7,-1776(gp) # 18110 <__SDATA_BEGIN__+0x8>
   12fa4:	02d586b3          	mul	a3,a1,a3
   12fa8:	9181b803          	ld	a6,-1768(gp) # 18118 <__SDATA_BEGIN__+0x10>
   12fac:	00700313          	li	t1,7
   12fb0:	fcc370e3          	bgeu	t1,a2,12f70 <memchr+0x18>
   12fb4:	00053783          	ld	a5,0(a0)
   12fb8:	00f6c7b3          	xor	a5,a3,a5
   12fbc:	01178733          	add	a4,a5,a7
   12fc0:	fff7c793          	not	a5,a5
   12fc4:	00f777b3          	and	a5,a4,a5
   12fc8:	0107f7b3          	and	a5,a5,a6
   12fcc:	fa0792e3          	bnez	a5,12f70 <memchr+0x18>
   12fd0:	00850513          	addi	a0,a0,8
   12fd4:	ff860613          	addi	a2,a2,-8
   12fd8:	fd9ff06f          	j	12fb0 <memchr+0x58>
   12fdc:	00150513          	addi	a0,a0,1
   12fe0:	f95ff06f          	j	12f74 <memchr+0x1c>
   12fe4:	00000513          	li	a0,0
   12fe8:	00008067          	ret

0000000000012fec <memcpy>:
   12fec:	00050793          	mv	a5,a0
   12ff0:	0035f713          	andi	a4,a1,3
   12ff4:	00070463          	beqz	a4,12ffc <memcpy+0x10>
   12ff8:	02061063          	bnez	a2,13018 <memcpy+0x2c>
   12ffc:	0037f693          	andi	a3,a5,3
   13000:	0e069463          	bnez	a3,130e8 <memcpy+0xfc>
   13004:	00058713          	mv	a4,a1
   13008:	00078693          	mv	a3,a5
   1300c:	00c58333          	add	t1,a1,a2
   13010:	00f00813          	li	a6,15
   13014:	0440006f          	j	13058 <memcpy+0x6c>
   13018:	0005c703          	lbu	a4,0(a1)
   1301c:	00158593          	addi	a1,a1,1
   13020:	00178793          	addi	a5,a5,1
   13024:	fff60613          	addi	a2,a2,-1
   13028:	fee78fa3          	sb	a4,-1(a5)
   1302c:	fc5ff06f          	j	12ff0 <memcpy+0x4>
   13030:	00072883          	lw	a7,0(a4)
   13034:	01068693          	addi	a3,a3,16
   13038:	01070713          	addi	a4,a4,16
   1303c:	ff16a823          	sw	a7,-16(a3)
   13040:	ff472883          	lw	a7,-12(a4)
   13044:	ff16aa23          	sw	a7,-12(a3)
   13048:	ff872883          	lw	a7,-8(a4)
   1304c:	ff16ac23          	sw	a7,-8(a3)
   13050:	ffc72883          	lw	a7,-4(a4)
   13054:	ff16ae23          	sw	a7,-4(a3)
   13058:	40e308b3          	sub	a7,t1,a4
   1305c:	fd186ae3          	bltu	a6,a7,13030 <memcpy+0x44>
   13060:	00465713          	srli	a4,a2,0x4
   13064:	ff000693          	li	a3,-16
   13068:	02d706b3          	mul	a3,a4,a3
   1306c:	00471713          	slli	a4,a4,0x4
   13070:	00e787b3          	add	a5,a5,a4
   13074:	00e585b3          	add	a1,a1,a4
   13078:	00c686b3          	add	a3,a3,a2
   1307c:	00867613          	andi	a2,a2,8
   13080:	00060e63          	beqz	a2,1309c <memcpy+0xb0>
   13084:	0005a703          	lw	a4,0(a1)
   13088:	00878793          	addi	a5,a5,8
   1308c:	00858593          	addi	a1,a1,8
   13090:	fee7ac23          	sw	a4,-8(a5)
   13094:	ffc5a703          	lw	a4,-4(a1)
   13098:	fee7ae23          	sw	a4,-4(a5)
   1309c:	0046f713          	andi	a4,a3,4
   130a0:	00070a63          	beqz	a4,130b4 <memcpy+0xc8>
   130a4:	0005a703          	lw	a4,0(a1)
   130a8:	00478793          	addi	a5,a5,4
   130ac:	00458593          	addi	a1,a1,4
   130b0:	fee7ae23          	sw	a4,-4(a5)
   130b4:	0026f713          	andi	a4,a3,2
   130b8:	00070e63          	beqz	a4,130d4 <memcpy+0xe8>
   130bc:	0005c703          	lbu	a4,0(a1)
   130c0:	00278793          	addi	a5,a5,2
   130c4:	00258593          	addi	a1,a1,2
   130c8:	fee78f23          	sb	a4,-2(a5)
   130cc:	fff5c703          	lbu	a4,-1(a1)
   130d0:	fee78fa3          	sb	a4,-1(a5)
   130d4:	0016f693          	andi	a3,a3,1
   130d8:	20068e63          	beqz	a3,132f4 <memcpy+0x308>
   130dc:	0005c703          	lbu	a4,0(a1)
   130e0:	00e78023          	sb	a4,0(a5)
   130e4:	2100006f          	j	132f4 <memcpy+0x308>
   130e8:	01f00713          	li	a4,31
   130ec:	0cc77863          	bgeu	a4,a2,131bc <memcpy+0x1d0>
   130f0:	00200313          	li	t1,2
   130f4:	0005c703          	lbu	a4,0(a1)
   130f8:	0005a803          	lw	a6,0(a1)
   130fc:	00c788b3          	add	a7,a5,a2
   13100:	1e668c63          	beq	a3,t1,132f8 <memcpy+0x30c>
   13104:	00300313          	li	t1,3
   13108:	28668863          	beq	a3,t1,13398 <memcpy+0x3ac>
   1310c:	00e78023          	sb	a4,0(a5)
   13110:	0015c703          	lbu	a4,1(a1)
   13114:	00358e13          	addi	t3,a1,3
   13118:	00378313          	addi	t1,a5,3
   1311c:	00e780a3          	sb	a4,1(a5)
   13120:	0025c703          	lbu	a4,2(a1)
   13124:	01000e93          	li	t4,16
   13128:	00e78123          	sb	a4,2(a5)
   1312c:	000e0713          	mv	a4,t3
   13130:	00030793          	mv	a5,t1
   13134:	00172683          	lw	a3,1(a4)
   13138:	0188581b          	srliw	a6,a6,0x18
   1313c:	01078793          	addi	a5,a5,16
   13140:	0086959b          	slliw	a1,a3,0x8
   13144:	00b86833          	or	a6,a6,a1
   13148:	00572583          	lw	a1,5(a4)
   1314c:	ff07a823          	sw	a6,-16(a5)
   13150:	0186d69b          	srliw	a3,a3,0x18
   13154:	0085981b          	slliw	a6,a1,0x8
   13158:	0106e6b3          	or	a3,a3,a6
   1315c:	fed7aa23          	sw	a3,-12(a5)
   13160:	00972683          	lw	a3,9(a4)
   13164:	0185d59b          	srliw	a1,a1,0x18
   13168:	01070713          	addi	a4,a4,16
   1316c:	0086981b          	slliw	a6,a3,0x8
   13170:	0105e5b3          	or	a1,a1,a6
   13174:	ffd72803          	lw	a6,-3(a4)
   13178:	feb7ac23          	sw	a1,-8(a5)
   1317c:	0186d69b          	srliw	a3,a3,0x18
   13180:	0088159b          	slliw	a1,a6,0x8
   13184:	00b6e6b3          	or	a3,a3,a1
   13188:	fed7ae23          	sw	a3,-4(a5)
   1318c:	40f886b3          	sub	a3,a7,a5
   13190:	fadee2e3          	bltu	t4,a3,13134 <memcpy+0x148>
   13194:	fec60713          	addi	a4,a2,-20
   13198:	00475713          	srli	a4,a4,0x4
   1319c:	00170793          	addi	a5,a4,1
   131a0:	00479793          	slli	a5,a5,0x4
   131a4:	00fe05b3          	add	a1,t3,a5
   131a8:	fed60613          	addi	a2,a2,-19
   131ac:	00f307b3          	add	a5,t1,a5
   131b0:	ff000693          	li	a3,-16
   131b4:	02d70733          	mul	a4,a4,a3
   131b8:	00c70633          	add	a2,a4,a2
   131bc:	01067713          	andi	a4,a2,16
   131c0:	08070663          	beqz	a4,1324c <memcpy+0x260>
   131c4:	0005c703          	lbu	a4,0(a1)
   131c8:	01078793          	addi	a5,a5,16
   131cc:	01058593          	addi	a1,a1,16
   131d0:	fee78823          	sb	a4,-16(a5)
   131d4:	ff15c703          	lbu	a4,-15(a1)
   131d8:	fee788a3          	sb	a4,-15(a5)
   131dc:	ff25c703          	lbu	a4,-14(a1)
   131e0:	fee78923          	sb	a4,-14(a5)
   131e4:	ff35c703          	lbu	a4,-13(a1)
   131e8:	fee789a3          	sb	a4,-13(a5)
   131ec:	ff45c703          	lbu	a4,-12(a1)
   131f0:	fee78a23          	sb	a4,-12(a5)
   131f4:	ff55c703          	lbu	a4,-11(a1)
   131f8:	fee78aa3          	sb	a4,-11(a5)
   131fc:	ff65c703          	lbu	a4,-10(a1)
   13200:	fee78b23          	sb	a4,-10(a5)
   13204:	ff75c703          	lbu	a4,-9(a1)
   13208:	fee78ba3          	sb	a4,-9(a5)
   1320c:	ff85c703          	lbu	a4,-8(a1)
   13210:	fee78c23          	sb	a4,-8(a5)
   13214:	ff95c703          	lbu	a4,-7(a1)
   13218:	fee78ca3          	sb	a4,-7(a5)
   1321c:	ffa5c703          	lbu	a4,-6(a1)
   13220:	fee78d23          	sb	a4,-6(a5)
   13224:	ffb5c703          	lbu	a4,-5(a1)
   13228:	fee78da3          	sb	a4,-5(a5)
   1322c:	ffc5c703          	lbu	a4,-4(a1)
   13230:	fee78e23          	sb	a4,-4(a5)
   13234:	ffd5c703          	lbu	a4,-3(a1)
   13238:	fee78ea3          	sb	a4,-3(a5)
   1323c:	ffe5c703          	lbu	a4,-2(a1)
   13240:	fee78f23          	sb	a4,-2(a5)
   13244:	fff5c703          	lbu	a4,-1(a1)
   13248:	fee78fa3          	sb	a4,-1(a5)
   1324c:	00867713          	andi	a4,a2,8
   13250:	04070663          	beqz	a4,1329c <memcpy+0x2b0>
   13254:	0005c703          	lbu	a4,0(a1)
   13258:	00878793          	addi	a5,a5,8
   1325c:	00858593          	addi	a1,a1,8
   13260:	fee78c23          	sb	a4,-8(a5)
   13264:	ff95c703          	lbu	a4,-7(a1)
   13268:	fee78ca3          	sb	a4,-7(a5)
   1326c:	ffa5c703          	lbu	a4,-6(a1)
   13270:	fee78d23          	sb	a4,-6(a5)
   13274:	ffb5c703          	lbu	a4,-5(a1)
   13278:	fee78da3          	sb	a4,-5(a5)
   1327c:	ffc5c703          	lbu	a4,-4(a1)
   13280:	fee78e23          	sb	a4,-4(a5)
   13284:	ffd5c703          	lbu	a4,-3(a1)
   13288:	fee78ea3          	sb	a4,-3(a5)
   1328c:	ffe5c703          	lbu	a4,-2(a1)
   13290:	fee78f23          	sb	a4,-2(a5)
   13294:	fff5c703          	lbu	a4,-1(a1)
   13298:	fee78fa3          	sb	a4,-1(a5)
   1329c:	00467713          	andi	a4,a2,4
   132a0:	02070663          	beqz	a4,132cc <memcpy+0x2e0>
   132a4:	0005c703          	lbu	a4,0(a1)
   132a8:	00478793          	addi	a5,a5,4
   132ac:	00458593          	addi	a1,a1,4
   132b0:	fee78e23          	sb	a4,-4(a5)
   132b4:	ffd5c703          	lbu	a4,-3(a1)
   132b8:	fee78ea3          	sb	a4,-3(a5)
   132bc:	ffe5c703          	lbu	a4,-2(a1)
   132c0:	fee78f23          	sb	a4,-2(a5)
   132c4:	fff5c703          	lbu	a4,-1(a1)
   132c8:	fee78fa3          	sb	a4,-1(a5)
   132cc:	00267713          	andi	a4,a2,2
   132d0:	00070e63          	beqz	a4,132ec <memcpy+0x300>
   132d4:	0005c703          	lbu	a4,0(a1)
   132d8:	00278793          	addi	a5,a5,2
   132dc:	00258593          	addi	a1,a1,2
   132e0:	fee78f23          	sb	a4,-2(a5)
   132e4:	fff5c703          	lbu	a4,-1(a1)
   132e8:	fee78fa3          	sb	a4,-1(a5)
   132ec:	00167613          	andi	a2,a2,1
   132f0:	de0616e3          	bnez	a2,130dc <memcpy+0xf0>
   132f4:	00008067          	ret
   132f8:	00e78023          	sb	a4,0(a5)
   132fc:	0015c703          	lbu	a4,1(a1)
   13300:	00258e13          	addi	t3,a1,2
   13304:	00278313          	addi	t1,a5,2
   13308:	00e780a3          	sb	a4,1(a5)
   1330c:	01100e93          	li	t4,17
   13310:	000e0713          	mv	a4,t3
   13314:	00030793          	mv	a5,t1
   13318:	00272683          	lw	a3,2(a4)
   1331c:	0108581b          	srliw	a6,a6,0x10
   13320:	01078793          	addi	a5,a5,16
   13324:	0106959b          	slliw	a1,a3,0x10
   13328:	00b86833          	or	a6,a6,a1
   1332c:	00672583          	lw	a1,6(a4)
   13330:	ff07a823          	sw	a6,-16(a5)
   13334:	0106d69b          	srliw	a3,a3,0x10
   13338:	0105981b          	slliw	a6,a1,0x10
   1333c:	0106e6b3          	or	a3,a3,a6
   13340:	fed7aa23          	sw	a3,-12(a5)
   13344:	00a72683          	lw	a3,10(a4)
   13348:	0105d59b          	srliw	a1,a1,0x10
   1334c:	01070713          	addi	a4,a4,16
   13350:	0106981b          	slliw	a6,a3,0x10
   13354:	0105e5b3          	or	a1,a1,a6
   13358:	ffe72803          	lw	a6,-2(a4)
   1335c:	feb7ac23          	sw	a1,-8(a5)
   13360:	0106d69b          	srliw	a3,a3,0x10
   13364:	0108159b          	slliw	a1,a6,0x10
   13368:	00b6e6b3          	or	a3,a3,a1
   1336c:	fed7ae23          	sw	a3,-4(a5)
   13370:	40f886b3          	sub	a3,a7,a5
   13374:	fadee2e3          	bltu	t4,a3,13318 <memcpy+0x32c>
   13378:	fec60713          	addi	a4,a2,-20
   1337c:	00475713          	srli	a4,a4,0x4
   13380:	00170793          	addi	a5,a4,1
   13384:	00479793          	slli	a5,a5,0x4
   13388:	00fe05b3          	add	a1,t3,a5
   1338c:	fee60613          	addi	a2,a2,-18
   13390:	00f307b3          	add	a5,t1,a5
   13394:	e1dff06f          	j	131b0 <memcpy+0x1c4>
   13398:	00158593          	addi	a1,a1,1
   1339c:	00178313          	addi	t1,a5,1
   133a0:	00e78023          	sb	a4,0(a5)
   133a4:	01200e13          	li	t3,18
   133a8:	00058713          	mv	a4,a1
   133ac:	00030793          	mv	a5,t1
   133b0:	00372683          	lw	a3,3(a4)
   133b4:	0088581b          	srliw	a6,a6,0x8
   133b8:	01078793          	addi	a5,a5,16
   133bc:	01869e9b          	slliw	t4,a3,0x18
   133c0:	01d86833          	or	a6,a6,t4
   133c4:	ff07a823          	sw	a6,-16(a5)
   133c8:	00772803          	lw	a6,7(a4)
   133cc:	0086d69b          	srliw	a3,a3,0x8
   133d0:	01070713          	addi	a4,a4,16
   133d4:	01881e9b          	slliw	t4,a6,0x18
   133d8:	01d6e6b3          	or	a3,a3,t4
   133dc:	fed7aa23          	sw	a3,-12(a5)
   133e0:	ffb72683          	lw	a3,-5(a4)
   133e4:	0088581b          	srliw	a6,a6,0x8
   133e8:	01869e9b          	slliw	t4,a3,0x18
   133ec:	01d86833          	or	a6,a6,t4
   133f0:	ff07ac23          	sw	a6,-8(a5)
   133f4:	fff72803          	lw	a6,-1(a4)
   133f8:	0086d69b          	srliw	a3,a3,0x8
   133fc:	01881e9b          	slliw	t4,a6,0x18
   13400:	01d6e6b3          	or	a3,a3,t4
   13404:	fed7ae23          	sw	a3,-4(a5)
   13408:	40f886b3          	sub	a3,a7,a5
   1340c:	fade62e3          	bltu	t3,a3,133b0 <memcpy+0x3c4>
   13410:	fec60713          	addi	a4,a2,-20
   13414:	00475713          	srli	a4,a4,0x4
   13418:	00170793          	addi	a5,a4,1
   1341c:	00479793          	slli	a5,a5,0x4
   13420:	00f585b3          	add	a1,a1,a5
   13424:	fef60613          	addi	a2,a2,-17
   13428:	00f307b3          	add	a5,t1,a5
   1342c:	d85ff06f          	j	131b0 <memcpy+0x1c4>

0000000000013430 <__set_thread_area>:
   13430:	00050213          	mv	tp,a0
   13434:	00000513          	li	a0,0
   13438:	00008067          	ret

000000000001343c <__lseek>:
   1343c:	03e00893          	li	a7,62
   13440:	00000073          	ecall
   13444:	d8cff06f          	j	129d0 <__syscall_ret>

0000000000013448 <wcrtomb>:
   13448:	00050a63          	beqz	a0,1345c <wcrtomb+0x14>
   1344c:	07f00693          	li	a3,127
   13450:	0005879b          	sext.w	a5,a1
   13454:	02b6e263          	bltu	a3,a1,13478 <wcrtomb+0x30>
   13458:	00b50023          	sb	a1,0(a0)
   1345c:	00100513          	li	a0,1
   13460:	00008067          	ret
   13464:	00b50023          	sb	a1,0(a0)
   13468:	00100513          	li	a0,1
   1346c:	00813083          	ld	ra,8(sp)
   13470:	01010113          	addi	sp,sp,16
   13474:	00008067          	ret
   13478:	ff010113          	addi	sp,sp,-16
   1347c:	00113423          	sd	ra,8(sp)
   13480:	00020713          	mv	a4,tp
   13484:	fd073703          	ld	a4,-48(a4)
   13488:	00073703          	ld	a4,0(a4)
   1348c:	02071463          	bnez	a4,134b4 <wcrtomb+0x6c>
   13490:	ffff2737          	lui	a4,0xffff2
   13494:	0807071b          	addiw	a4,a4,128
   13498:	00f707bb          	addw	a5,a4,a5
   1349c:	fcf6f4e3          	bgeu	a3,a5,13464 <wcrtomb+0x1c>
   134a0:	cb4ff0ef          	jal	ra,12954 <__errno_location>
   134a4:	05400793          	li	a5,84
   134a8:	00f52023          	sw	a5,0(a0)
   134ac:	fff00513          	li	a0,-1
   134b0:	fbdff06f          	j	1346c <wcrtomb+0x24>
   134b4:	7ff00713          	li	a4,2047
   134b8:	02f76263          	bltu	a4,a5,134dc <wcrtomb+0x94>
   134bc:	4065d79b          	sraiw	a5,a1,0x6
   134c0:	03f5f593          	andi	a1,a1,63
   134c4:	fc07e793          	ori	a5,a5,-64
   134c8:	f805e593          	ori	a1,a1,-128
   134cc:	00f50023          	sb	a5,0(a0)
   134d0:	00b500a3          	sb	a1,1(a0)
   134d4:	00200513          	li	a0,2
   134d8:	f95ff06f          	j	1346c <wcrtomb+0x24>
   134dc:	0000d737          	lui	a4,0xd
   134e0:	7ff70713          	addi	a4,a4,2047 # d7ff <exit-0x2921>
   134e4:	00f77a63          	bgeu	a4,a5,134f8 <wcrtomb+0xb0>
   134e8:	ffff2737          	lui	a4,0xffff2
   134ec:	00f7073b          	addw	a4,a4,a5
   134f0:	000026b7          	lui	a3,0x2
   134f4:	02d77a63          	bgeu	a4,a3,13528 <wcrtomb+0xe0>
   134f8:	40c5d79b          	sraiw	a5,a1,0xc
   134fc:	fe07e793          	ori	a5,a5,-32
   13500:	00f50023          	sb	a5,0(a0)
   13504:	4065d79b          	sraiw	a5,a1,0x6
   13508:	03f7f793          	andi	a5,a5,63
   1350c:	03f5f593          	andi	a1,a1,63
   13510:	f807e793          	ori	a5,a5,-128
   13514:	f805e593          	ori	a1,a1,-128
   13518:	00f500a3          	sb	a5,1(a0)
   1351c:	00b50123          	sb	a1,2(a0)
   13520:	00300513          	li	a0,3
   13524:	f49ff06f          	j	1346c <wcrtomb+0x24>
   13528:	ffff0737          	lui	a4,0xffff0
   1352c:	00f707bb          	addw	a5,a4,a5
   13530:	00100737          	lui	a4,0x100
   13534:	f6e7f6e3          	bgeu	a5,a4,134a0 <wcrtomb+0x58>
   13538:	4125d79b          	sraiw	a5,a1,0x12
   1353c:	ff07e793          	ori	a5,a5,-16
   13540:	00f50023          	sb	a5,0(a0)
   13544:	40c5d79b          	sraiw	a5,a1,0xc
   13548:	03f7f793          	andi	a5,a5,63
   1354c:	f807e793          	ori	a5,a5,-128
   13550:	00f500a3          	sb	a5,1(a0)
   13554:	4065d79b          	sraiw	a5,a1,0x6
   13558:	03f7f793          	andi	a5,a5,63
   1355c:	03f5f593          	andi	a1,a1,63
   13560:	f807e793          	ori	a5,a5,-128
   13564:	f805e593          	ori	a1,a1,-128
   13568:	00f50123          	sb	a5,2(a0)
   1356c:	00b501a3          	sb	a1,3(a0)
   13570:	00400513          	li	a0,4
   13574:	ef9ff06f          	j	1346c <wcrtomb+0x24>

0000000000013578 <close_file>:
   13578:	08050063          	beqz	a0,135f8 <close_file+0x80>
   1357c:	08c52783          	lw	a5,140(a0)
   13580:	ff010113          	addi	sp,sp,-16
   13584:	00813023          	sd	s0,0(sp)
   13588:	00113423          	sd	ra,8(sp)
   1358c:	0007879b          	sext.w	a5,a5
   13590:	00050413          	mv	s0,a0
   13594:	0007c463          	bltz	a5,1359c <close_file+0x24>
   13598:	ddcff0ef          	jal	ra,12b74 <__lockfile>
   1359c:	02843703          	ld	a4,40(s0)
   135a0:	03843783          	ld	a5,56(s0)
   135a4:	00f70c63          	beq	a4,a5,135bc <close_file+0x44>
   135a8:	04843783          	ld	a5,72(s0)
   135ac:	00000613          	li	a2,0
   135b0:	00000593          	li	a1,0
   135b4:	00040513          	mv	a0,s0
   135b8:	000780e7          	jalr	a5
   135bc:	00843583          	ld	a1,8(s0)
   135c0:	01043783          	ld	a5,16(s0)
   135c4:	02f58263          	beq	a1,a5,135e8 <close_file+0x70>
   135c8:	05043703          	ld	a4,80(s0)
   135cc:	00040513          	mv	a0,s0
   135d0:	00013403          	ld	s0,0(sp)
   135d4:	00813083          	ld	ra,8(sp)
   135d8:	00100613          	li	a2,1
   135dc:	40f585b3          	sub	a1,a1,a5
   135e0:	01010113          	addi	sp,sp,16
   135e4:	00070067          	jr	a4 # 100000 <__global_pointer$+0xe7800>
   135e8:	00813083          	ld	ra,8(sp)
   135ec:	00013403          	ld	s0,0(sp)
   135f0:	01010113          	addi	sp,sp,16
   135f4:	00008067          	ret
   135f8:	00008067          	ret

00000000000135fc <__stdio_exit>:
   135fc:	ff010113          	addi	sp,sp,-16
   13600:	00813023          	sd	s0,0(sp)
   13604:	00113423          	sd	ra,8(sp)
   13608:	150000ef          	jal	ra,13758 <__ofl_lock>
   1360c:	00053403          	ld	s0,0(a0)
   13610:	02041463          	bnez	s0,13638 <__stdio_exit+0x3c>
   13614:	9701b503          	ld	a0,-1680(gp) # 18170 <__stderr_used>
   13618:	f61ff0ef          	jal	ra,13578 <close_file>
   1361c:	9301b503          	ld	a0,-1744(gp) # 18130 <__stdout_used>
   13620:	f59ff0ef          	jal	ra,13578 <close_file>
   13624:	00013403          	ld	s0,0(sp)
   13628:	00813083          	ld	ra,8(sp)
   1362c:	9701b503          	ld	a0,-1680(gp) # 18170 <__stderr_used>
   13630:	01010113          	addi	sp,sp,16
   13634:	f45ff06f          	j	13578 <close_file>
   13638:	00040513          	mv	a0,s0
   1363c:	f3dff0ef          	jal	ra,13578 <close_file>
   13640:	07043403          	ld	s0,112(s0)
   13644:	fcdff06f          	j	13610 <__stdio_exit+0x14>

0000000000013648 <__stdio_write>:
   13648:	fb010113          	addi	sp,sp,-80
   1364c:	03853783          	ld	a5,56(a0)
   13650:	03313423          	sd	s3,40(sp)
   13654:	02853983          	ld	s3,40(a0)
   13658:	04813023          	sd	s0,64(sp)
   1365c:	02913c23          	sd	s1,56(sp)
   13660:	40f989b3          	sub	s3,s3,a5
   13664:	03213823          	sd	s2,48(sp)
   13668:	03413023          	sd	s4,32(sp)
   1366c:	01313423          	sd	s3,8(sp)
   13670:	04113423          	sd	ra,72(sp)
   13674:	00050413          	mv	s0,a0
   13678:	00060913          	mv	s2,a2
   1367c:	00f13023          	sd	a5,0(sp)
   13680:	00b13823          	sd	a1,16(sp)
   13684:	00c13c23          	sd	a2,24(sp)
   13688:	00c989b3          	add	s3,s3,a2
   1368c:	00200a13          	li	s4,2
   13690:	00010493          	mv	s1,sp
   13694:	07842503          	lw	a0,120(s0)
   13698:	04200893          	li	a7,66
   1369c:	00048593          	mv	a1,s1
   136a0:	000a0613          	mv	a2,s4
   136a4:	00000073          	ecall
   136a8:	b28ff0ef          	jal	ra,129d0 <__syscall_ret>
   136ac:	04a99063          	bne	s3,a0,136ec <__stdio_write+0xa4>
   136b0:	05843783          	ld	a5,88(s0)
   136b4:	06043703          	ld	a4,96(s0)
   136b8:	02f43c23          	sd	a5,56(s0)
   136bc:	00e78733          	add	a4,a5,a4
   136c0:	02e43023          	sd	a4,32(s0)
   136c4:	02f43423          	sd	a5,40(s0)
   136c8:	04813083          	ld	ra,72(sp)
   136cc:	04013403          	ld	s0,64(sp)
   136d0:	03813483          	ld	s1,56(sp)
   136d4:	02813983          	ld	s3,40(sp)
   136d8:	02013a03          	ld	s4,32(sp)
   136dc:	00090513          	mv	a0,s2
   136e0:	03013903          	ld	s2,48(sp)
   136e4:	05010113          	addi	sp,sp,80
   136e8:	00008067          	ret
   136ec:	02055863          	bgez	a0,1371c <__stdio_write+0xd4>
   136f0:	00042783          	lw	a5,0(s0)
   136f4:	02043023          	sd	zero,32(s0)
   136f8:	02043c23          	sd	zero,56(s0)
   136fc:	0207e793          	ori	a5,a5,32
   13700:	00f42023          	sw	a5,0(s0)
   13704:	02043423          	sd	zero,40(s0)
   13708:	00200793          	li	a5,2
   1370c:	04fa0263          	beq	s4,a5,13750 <__stdio_write+0x108>
   13710:	0084b783          	ld	a5,8(s1)
   13714:	40f90933          	sub	s2,s2,a5
   13718:	fb1ff06f          	j	136c8 <__stdio_write+0x80>
   1371c:	0084b783          	ld	a5,8(s1)
   13720:	40a989b3          	sub	s3,s3,a0
   13724:	00a7f863          	bgeu	a5,a0,13734 <__stdio_write+0xec>
   13728:	40f50533          	sub	a0,a0,a5
   1372c:	01048493          	addi	s1,s1,16
   13730:	fffa0a1b          	addiw	s4,s4,-1
   13734:	0004b783          	ld	a5,0(s1)
   13738:	00a787b3          	add	a5,a5,a0
   1373c:	00f4b023          	sd	a5,0(s1)
   13740:	0084b783          	ld	a5,8(s1)
   13744:	40a78533          	sub	a0,a5,a0
   13748:	00a4b423          	sd	a0,8(s1)
   1374c:	f49ff06f          	j	13694 <__stdio_write+0x4c>
   13750:	00000913          	li	s2,0
   13754:	f75ff06f          	j	136c8 <__stdio_write+0x80>

0000000000013758 <__ofl_lock>:
   13758:	ff010113          	addi	sp,sp,-16
   1375c:	98018513          	addi	a0,gp,-1664 # 18180 <ofl_lock>
   13760:	00113423          	sd	ra,8(sp)
   13764:	01c000ef          	jal	ra,13780 <__lock>
   13768:	00813083          	ld	ra,8(sp)
   1376c:	97818513          	addi	a0,gp,-1672 # 18178 <ofl_head>
   13770:	01010113          	addi	sp,sp,16
   13774:	00008067          	ret

0000000000013778 <__ofl_unlock>:
   13778:	98018513          	addi	a0,gp,-1664 # 18180 <ofl_lock>
   1377c:	0fc0006f          	j	13878 <__unlock>

0000000000013780 <__lock>:
   13780:	9c018793          	addi	a5,gp,-1600 # 181c0 <__libc>
   13784:	00c7a783          	lw	a5,12(a5)
   13788:	00050713          	mv	a4,a0
   1378c:	0007879b          	sext.w	a5,a5
   13790:	0e078263          	beqz	a5,13874 <__lock+0xf4>
   13794:	800006b7          	lui	a3,0x80000
   13798:	00168693          	addi	a3,a3,1 # ffffffff80000001 <__global_pointer$+0xffffffff7ffe7801>
   1379c:	00000613          	li	a2,0
   137a0:	160527af          	lr.w.aqrl	a5,(a0)
   137a4:	00c79663          	bne	a5,a2,137b0 <__lock+0x30>
   137a8:	1ed525af          	sc.w.aqrl	a1,a3,(a0)
   137ac:	fe059ae3          	bnez	a1,137a0 <__lock+0x20>
   137b0:	0007879b          	sext.w	a5,a5
   137b4:	0c078063          	beqz	a5,13874 <__lock+0xf4>
   137b8:	80000537          	lui	a0,0x80000
   137bc:	00a00593          	li	a1,10
   137c0:	fff54513          	not	a0,a0
   137c4:	00078613          	mv	a2,a5
   137c8:	0007d463          	bgez	a5,137d0 <__lock+0x50>
   137cc:	00f5063b          	addw	a2,a0,a5
   137d0:	00c6883b          	addw	a6,a3,a2
   137d4:	160727af          	lr.w.aqrl	a5,(a4)
   137d8:	00c79663          	bne	a5,a2,137e4 <__lock+0x64>
   137dc:	1f0728af          	sc.w.aqrl	a7,a6,(a4)
   137e0:	fe089ae3          	bnez	a7,137d4 <__lock+0x54>
   137e4:	0007879b          	sext.w	a5,a5
   137e8:	08f60663          	beq	a2,a5,13874 <__lock+0xf4>
   137ec:	fff5859b          	addiw	a1,a1,-1
   137f0:	fc059ae3          	bnez	a1,137c4 <__lock+0x44>
   137f4:	00072783          	lw	a5,0(a4)
   137f8:	0007861b          	sext.w	a2,a5
   137fc:	0017879b          	addiw	a5,a5,1
   13800:	160726af          	lr.w.aqrl	a3,(a4)
   13804:	00c69663          	bne	a3,a2,13810 <__lock+0x90>
   13808:	1ef725af          	sc.w.aqrl	a1,a5,(a4)
   1380c:	fe059ae3          	bnez	a1,13800 <__lock+0x80>
   13810:	0006869b          	sext.w	a3,a3
   13814:	fed610e3          	bne	a2,a3,137f4 <__lock+0x74>
   13818:	80000837          	lui	a6,0x80000
   1381c:	fda00313          	li	t1,-38
   13820:	fff84e13          	not	t3,a6
   13824:	00078693          	mv	a3,a5
   13828:	0207d863          	bgez	a5,13858 <__lock+0xd8>
   1382c:	06200893          	li	a7,98
   13830:	00070513          	mv	a0,a4
   13834:	08000593          	li	a1,128
   13838:	00078613          	mv	a2,a5
   1383c:	00000693          	li	a3,0
   13840:	00000073          	ecall
   13844:	00651863          	bne	a0,t1,13854 <__lock+0xd4>
   13848:	00070513          	mv	a0,a4
   1384c:	00000593          	li	a1,0
   13850:	00000073          	ecall
   13854:	00fe06bb          	addw	a3,t3,a5
   13858:	00d8063b          	addw	a2,a6,a3
   1385c:	160727af          	lr.w.aqrl	a5,(a4)
   13860:	00d79663          	bne	a5,a3,1386c <__lock+0xec>
   13864:	1ec725af          	sc.w.aqrl	a1,a2,(a4)
   13868:	fe059ae3          	bnez	a1,1385c <__lock+0xdc>
   1386c:	0007879b          	sext.w	a5,a5
   13870:	faf69ae3          	bne	a3,a5,13824 <__lock+0xa4>
   13874:	00008067          	ret

0000000000013878 <__unlock>:
   13878:	00052703          	lw	a4,0(a0) # ffffffff80000000 <__global_pointer$+0xffffffff7ffe7800>
   1387c:	00050793          	mv	a5,a0
   13880:	0007071b          	sext.w	a4,a4
   13884:	06075263          	bgez	a4,138e8 <__unlock+0x70>
   13888:	800005b7          	lui	a1,0x80000
   1388c:	fff5c593          	not	a1,a1
   13890:	0007a703          	lw	a4,0(a5)
   13894:	0007069b          	sext.w	a3,a4
   13898:	00b7073b          	addw	a4,a4,a1
   1389c:	1607a62f          	lr.w.aqrl	a2,(a5)
   138a0:	00d61663          	bne	a2,a3,138ac <__unlock+0x34>
   138a4:	1ee7a52f          	sc.w.aqrl	a0,a4,(a5)
   138a8:	fe051ae3          	bnez	a0,1389c <__unlock+0x24>
   138ac:	0006071b          	sext.w	a4,a2
   138b0:	fee690e3          	bne	a3,a4,13890 <__unlock+0x18>
   138b4:	80000737          	lui	a4,0x80000
   138b8:	00170713          	addi	a4,a4,1 # ffffffff80000001 <__global_pointer$+0xffffffff7ffe7801>
   138bc:	02e68663          	beq	a3,a4,138e8 <__unlock+0x70>
   138c0:	06200893          	li	a7,98
   138c4:	00078513          	mv	a0,a5
   138c8:	08100593          	li	a1,129
   138cc:	00100613          	li	a2,1
   138d0:	00000073          	ecall
   138d4:	fda00713          	li	a4,-38
   138d8:	00e51863          	bne	a0,a4,138e8 <__unlock+0x70>
   138dc:	00078513          	mv	a0,a5
   138e0:	00100593          	li	a1,1
   138e4:	00000073          	ecall
   138e8:	00008067          	ret

00000000000138ec <__addtf3>:
   138ec:	fd010113          	addi	sp,sp,-48
   138f0:	02113423          	sd	ra,40(sp)
   138f4:	02813023          	sd	s0,32(sp)
   138f8:	00913c23          	sd	s1,24(sp)
   138fc:	01213823          	sd	s2,16(sp)
   13900:	01313423          	sd	s3,8(sp)
   13904:	01413023          	sd	s4,0(sp)
   13908:	002024f3          	frrm	s1
   1390c:	fff00e93          	li	t4,-1
   13910:	010ed713          	srli	a4,t4,0x10
   13914:	0305d413          	srli	s0,a1,0x30
   13918:	03f5d913          	srli	s2,a1,0x3f
   1391c:	00e5f5b3          	and	a1,a1,a4
   13920:	00359593          	slli	a1,a1,0x3
   13924:	03d55793          	srli	a5,a0,0x3d
   13928:	00008337          	lui	t1,0x8
   1392c:	fff30e13          	addi	t3,t1,-1 # 7fff <exit-0x8121>
   13930:	00b7e7b3          	or	a5,a5,a1
   13934:	03f6d893          	srli	a7,a3,0x3f
   13938:	0306d593          	srli	a1,a3,0x30
   1393c:	00e6f6b3          	and	a3,a3,a4
   13940:	00369693          	slli	a3,a3,0x3
   13944:	03d65713          	srli	a4,a2,0x3d
   13948:	01c47433          	and	s0,s0,t3
   1394c:	01c5f5b3          	and	a1,a1,t3
   13950:	00d76733          	or	a4,a4,a3
   13954:	40b406bb          	subw	a3,s0,a1
   13958:	0004849b          	sext.w	s1,s1
   1395c:	00351513          	slli	a0,a0,0x3
   13960:	00361613          	slli	a2,a2,0x3
   13964:	0006881b          	sext.w	a6,a3
   13968:	4d191a63          	bne	s2,a7,13e3c <__addtf3+0x550>
   1396c:	13005e63          	blez	a6,13aa8 <__addtf3+0x1bc>
   13970:	0a059863          	bnez	a1,13a20 <__addtf3+0x134>
   13974:	00c765b3          	or	a1,a4,a2
   13978:	02059263          	bnez	a1,1399c <__addtf3+0xb0>
   1397c:	4dc41a63          	bne	s0,t3,13e50 <__addtf3+0x564>
   13980:	00a7e733          	or	a4,a5,a0
   13984:	18070ee3          	beqz	a4,14320 <__addtf3+0xa34>
   13988:	0327d713          	srli	a4,a5,0x32
   1398c:	00040593          	mv	a1,s0
   13990:	00000813          	li	a6,0
   13994:	3e071a63          	bnez	a4,13d88 <__addtf3+0x49c>
   13998:	1440006f          	j	13adc <__addtf3+0x1f0>
   1399c:	fff6881b          	addiw	a6,a3,-1
   139a0:	06081463          	bnez	a6,13a08 <__addtf3+0x11c>
   139a4:	00a60633          	add	a2,a2,a0
   139a8:	00a63533          	sltu	a0,a2,a0
   139ac:	00f70733          	add	a4,a4,a5
   139b0:	00a70733          	add	a4,a4,a0
   139b4:	00040593          	mv	a1,s0
   139b8:	00060513          	mv	a0,a2
   139bc:	03375793          	srli	a5,a4,0x33
   139c0:	0017f793          	andi	a5,a5,1
   139c4:	0c078ee3          	beqz	a5,142a0 <__addtf3+0x9b4>
   139c8:	000086b7          	lui	a3,0x8
   139cc:	00158593          	addi	a1,a1,1 # ffffffff80000001 <__global_pointer$+0xffffffff7ffe7801>
   139d0:	fff68793          	addi	a5,a3,-1 # 7fff <exit-0x8121>
   139d4:	42f58463          	beq	a1,a5,13dfc <__addtf3+0x510>
   139d8:	fff00793          	li	a5,-1
   139dc:	03379793          	slli	a5,a5,0x33
   139e0:	fff78793          	addi	a5,a5,-1
   139e4:	00f777b3          	and	a5,a4,a5
   139e8:	00155713          	srli	a4,a0,0x1
   139ec:	00157513          	andi	a0,a0,1
   139f0:	00a76533          	or	a0,a4,a0
   139f4:	03f79713          	slli	a4,a5,0x3f
   139f8:	00a76533          	or	a0,a4,a0
   139fc:	0017d793          	srli	a5,a5,0x1
   13a00:	00000813          	li	a6,0
   13a04:	3840006f          	j	13d88 <__addtf3+0x49c>
   13a08:	03c41463          	bne	s0,t3,13a30 <__addtf3+0x144>
   13a0c:	00a7e733          	or	a4,a5,a0
   13a10:	100708e3          	beqz	a4,14320 <__addtf3+0xa34>
   13a14:	0327d713          	srli	a4,a5,0x32
   13a18:	00177713          	andi	a4,a4,1
   13a1c:	f71ff06f          	j	1398c <__addtf3+0xa0>
   13a20:	ffc406e3          	beq	s0,t3,13a0c <__addtf3+0x120>
   13a24:	00100693          	li	a3,1
   13a28:	03369693          	slli	a3,a3,0x33
   13a2c:	00d76733          	or	a4,a4,a3
   13a30:	07400693          	li	a3,116
   13a34:	0706c463          	blt	a3,a6,13a9c <__addtf3+0x1b0>
   13a38:	03f00693          	li	a3,63
   13a3c:	0306c663          	blt	a3,a6,13a68 <__addtf3+0x17c>
   13a40:	04000593          	li	a1,64
   13a44:	410585bb          	subw	a1,a1,a6
   13a48:	00b716b3          	sll	a3,a4,a1
   13a4c:	010658b3          	srl	a7,a2,a6
   13a50:	00b61633          	sll	a2,a2,a1
   13a54:	0116e6b3          	or	a3,a3,a7
   13a58:	00c03633          	snez	a2,a2
   13a5c:	00c6e633          	or	a2,a3,a2
   13a60:	01075733          	srl	a4,a4,a6
   13a64:	f41ff06f          	j	139a4 <__addtf3+0xb8>
   13a68:	fc08069b          	addiw	a3,a6,-64
   13a6c:	04000893          	li	a7,64
   13a70:	00d756b3          	srl	a3,a4,a3
   13a74:	00000593          	li	a1,0
   13a78:	01180863          	beq	a6,a7,13a88 <__addtf3+0x19c>
   13a7c:	08000593          	li	a1,128
   13a80:	410585bb          	subw	a1,a1,a6
   13a84:	00b715b3          	sll	a1,a4,a1
   13a88:	00c5e633          	or	a2,a1,a2
   13a8c:	00c03633          	snez	a2,a2
   13a90:	00c6e633          	or	a2,a3,a2
   13a94:	00000713          	li	a4,0
   13a98:	f0dff06f          	j	139a4 <__addtf3+0xb8>
   13a9c:	00c76633          	or	a2,a4,a2
   13aa0:	00c03633          	snez	a2,a2
   13aa4:	ff1ff06f          	j	13a94 <__addtf3+0x1a8>
   13aa8:	12080063          	beqz	a6,13bc8 <__addtf3+0x2dc>
   13aac:	08041063          	bnez	s0,13b2c <__addtf3+0x240>
   13ab0:	00a7e833          	or	a6,a5,a0
   13ab4:	02081863          	bnez	a6,13ae4 <__addtf3+0x1f8>
   13ab8:	00060513          	mv	a0,a2
   13abc:	7fc59263          	bne	a1,t3,142a0 <__addtf3+0x9b4>
   13ac0:	00c76533          	or	a0,a4,a2
   13ac4:	040502e3          	beqz	a0,14308 <__addtf3+0xa1c>
   13ac8:	03275793          	srli	a5,a4,0x32
   13acc:	0017f793          	andi	a5,a5,1
   13ad0:	040792e3          	bnez	a5,14314 <__addtf3+0xa28>
   13ad4:	00070793          	mv	a5,a4
   13ad8:	00060513          	mv	a0,a2
   13adc:	01000693          	li	a3,16
   13ae0:	1400006f          	j	13c20 <__addtf3+0x334>
   13ae4:	fff6c693          	not	a3,a3
   13ae8:	0006869b          	sext.w	a3,a3
   13aec:	00069c63          	bnez	a3,13b04 <__addtf3+0x218>
   13af0:	00c50533          	add	a0,a0,a2
   13af4:	00e78733          	add	a4,a5,a4
   13af8:	00c53633          	sltu	a2,a0,a2
   13afc:	00c70733          	add	a4,a4,a2
   13b00:	ebdff06f          	j	139bc <__addtf3+0xd0>
   13b04:	03c59e63          	bne	a1,t3,13b40 <__addtf3+0x254>
   13b08:	00c76533          	or	a0,a4,a2
   13b0c:	7e050e63          	beqz	a0,14308 <__addtf3+0xa1c>
   13b10:	03275793          	srli	a5,a4,0x32
   13b14:	0017f793          	andi	a5,a5,1
   13b18:	7e079e63          	bnez	a5,14314 <__addtf3+0xa28>
   13b1c:	00070793          	mv	a5,a4
   13b20:	00060513          	mv	a0,a2
   13b24:	00000813          	li	a6,0
   13b28:	fb5ff06f          	j	13adc <__addtf3+0x1f0>
   13b2c:	fdc58ee3          	beq	a1,t3,13b08 <__addtf3+0x21c>
   13b30:	00100813          	li	a6,1
   13b34:	03381813          	slli	a6,a6,0x33
   13b38:	40d006bb          	negw	a3,a3
   13b3c:	0107e7b3          	or	a5,a5,a6
   13b40:	07400813          	li	a6,116
   13b44:	06d84c63          	blt	a6,a3,13bbc <__addtf3+0x2d0>
   13b48:	03f00813          	li	a6,63
   13b4c:	02d84e63          	blt	a6,a3,13b88 <__addtf3+0x29c>
   13b50:	04000893          	li	a7,64
   13b54:	40d888bb          	subw	a7,a7,a3
   13b58:	00d55333          	srl	t1,a0,a3
   13b5c:	01179833          	sll	a6,a5,a7
   13b60:	01151533          	sll	a0,a0,a7
   13b64:	00686833          	or	a6,a6,t1
   13b68:	00a03533          	snez	a0,a0
   13b6c:	00a86533          	or	a0,a6,a0
   13b70:	00d7d6b3          	srl	a3,a5,a3
   13b74:	00c50533          	add	a0,a0,a2
   13b78:	00e686b3          	add	a3,a3,a4
   13b7c:	00c53633          	sltu	a2,a0,a2
   13b80:	00c68733          	add	a4,a3,a2
   13b84:	e39ff06f          	j	139bc <__addtf3+0xd0>
   13b88:	fc06881b          	addiw	a6,a3,-64
   13b8c:	04000313          	li	t1,64
   13b90:	0107d833          	srl	a6,a5,a6
   13b94:	00000893          	li	a7,0
   13b98:	00668863          	beq	a3,t1,13ba8 <__addtf3+0x2bc>
   13b9c:	08000893          	li	a7,128
   13ba0:	40d886bb          	subw	a3,a7,a3
   13ba4:	00d798b3          	sll	a7,a5,a3
   13ba8:	00a8e533          	or	a0,a7,a0
   13bac:	00a03533          	snez	a0,a0
   13bb0:	00a86533          	or	a0,a6,a0
   13bb4:	00000693          	li	a3,0
   13bb8:	fbdff06f          	j	13b74 <__addtf3+0x288>
   13bbc:	00a7e533          	or	a0,a5,a0
   13bc0:	00a03533          	snez	a0,a0
   13bc4:	ff1ff06f          	j	13bb4 <__addtf3+0x2c8>
   13bc8:	00140893          	addi	a7,s0,1
   13bcc:	ffe30693          	addi	a3,t1,-2
   13bd0:	00d8f333          	and	t1,a7,a3
   13bd4:	18031663          	bnez	t1,13d60 <__addtf3+0x474>
   13bd8:	00a7e8b3          	or	a7,a5,a0
   13bdc:	06041063          	bnez	s0,13c3c <__addtf3+0x350>
   13be0:	6c088c63          	beqz	a7,142b8 <__addtf3+0x9cc>
   13be4:	00c766b3          	or	a3,a4,a2
   13be8:	48068c63          	beqz	a3,14080 <__addtf3+0x794>
   13bec:	00c50633          	add	a2,a0,a2
   13bf0:	00e787b3          	add	a5,a5,a4
   13bf4:	00a63533          	sltu	a0,a2,a0
   13bf8:	00a787b3          	add	a5,a5,a0
   13bfc:	0337d713          	srli	a4,a5,0x33
   13c00:	00177713          	andi	a4,a4,1
   13c04:	6a070c63          	beqz	a4,142bc <__addtf3+0x9d0>
   13c08:	033e9713          	slli	a4,t4,0x33
   13c0c:	fff70713          	addi	a4,a4,-1
   13c10:	00e7f7b3          	and	a5,a5,a4
   13c14:	00060513          	mv	a0,a2
   13c18:	00000693          	li	a3,0
   13c1c:	00100593          	li	a1,1
   13c20:	00757713          	andi	a4,a0,7
   13c24:	1a071063          	bnez	a4,13dc4 <__addtf3+0x4d8>
   13c28:	08080663          	beqz	a6,13cb4 <__addtf3+0x3c8>
   13c2c:	0016f713          	andi	a4,a3,1
   13c30:	08070263          	beqz	a4,13cb4 <__addtf3+0x3c8>
   13c34:	0026e693          	ori	a3,a3,2
   13c38:	07c0006f          	j	13cb4 <__addtf3+0x3c8>
   13c3c:	03c41c63          	bne	s0,t3,13c74 <__addtf3+0x388>
   13c40:	78088c63          	beqz	a7,143d8 <__addtf3+0xaec>
   13c44:	0327d693          	srli	a3,a5,0x32
   13c48:	0016f693          	andi	a3,a3,1
   13c4c:	0016b693          	seqz	a3,a3
   13c50:	00469693          	slli	a3,a3,0x4
   13c54:	04859063          	bne	a1,s0,13c94 <__addtf3+0x3a8>
   13c58:	00c765b3          	or	a1,a4,a2
   13c5c:	02058063          	beqz	a1,13c7c <__addtf3+0x390>
   13c60:	03275593          	srli	a1,a4,0x32
   13c64:	0015f593          	andi	a1,a1,1
   13c68:	00059a63          	bnez	a1,13c7c <__addtf3+0x390>
   13c6c:	01000693          	li	a3,16
   13c70:	00c0006f          	j	13c7c <__addtf3+0x390>
   13c74:	00000693          	li	a3,0
   13c78:	ffc580e3          	beq	a1,t3,13c58 <__addtf3+0x36c>
   13c7c:	00089c63          	bnez	a7,13c94 <__addtf3+0x3a8>
   13c80:	00070793          	mv	a5,a4
   13c84:	00060513          	mv	a0,a2
   13c88:	000085b7          	lui	a1,0x8
   13c8c:	fff58593          	addi	a1,a1,-1 # 7fff <exit-0x8121>
   13c90:	f91ff06f          	j	13c20 <__addtf3+0x334>
   13c94:	00c76633          	or	a2,a4,a2
   13c98:	fe0608e3          	beqz	a2,13c88 <__addtf3+0x39c>
   13c9c:	00100793          	li	a5,1
   13ca0:	000085b7          	lui	a1,0x8
   13ca4:	00000913          	li	s2,0
   13ca8:	03279793          	slli	a5,a5,0x32
   13cac:	00000513          	li	a0,0
   13cb0:	fff58593          	addi	a1,a1,-1 # 7fff <exit-0x8121>
   13cb4:	0337d713          	srli	a4,a5,0x33
   13cb8:	00177713          	andi	a4,a4,1
   13cbc:	02070263          	beqz	a4,13ce0 <__addtf3+0x3f4>
   13cc0:	00008737          	lui	a4,0x8
   13cc4:	00158593          	addi	a1,a1,1
   13cc8:	fff70613          	addi	a2,a4,-1 # 7fff <exit-0x8121>
   13ccc:	6cc58463          	beq	a1,a2,14394 <__addtf3+0xaa8>
   13cd0:	fff00713          	li	a4,-1
   13cd4:	03371713          	slli	a4,a4,0x33
   13cd8:	fff70713          	addi	a4,a4,-1
   13cdc:	00e7f7b3          	and	a5,a5,a4
   13ce0:	00355713          	srli	a4,a0,0x3
   13ce4:	03d79513          	slli	a0,a5,0x3d
   13ce8:	00e56533          	or	a0,a0,a4
   13cec:	00008737          	lui	a4,0x8
   13cf0:	fff70713          	addi	a4,a4,-1 # 7fff <exit-0x8121>
   13cf4:	0037d793          	srli	a5,a5,0x3
   13cf8:	02e59063          	bne	a1,a4,13d18 <__addtf3+0x42c>
   13cfc:	00f56533          	or	a0,a0,a5
   13d00:	00000793          	li	a5,0
   13d04:	00050a63          	beqz	a0,13d18 <__addtf3+0x42c>
   13d08:	00100793          	li	a5,1
   13d0c:	02f79793          	slli	a5,a5,0x2f
   13d10:	00000513          	li	a0,0
   13d14:	00000913          	li	s2,0
   13d18:	03159593          	slli	a1,a1,0x31
   13d1c:	00f91913          	slli	s2,s2,0xf
   13d20:	0315d593          	srli	a1,a1,0x31
   13d24:	00b965b3          	or	a1,s2,a1
   13d28:	01079793          	slli	a5,a5,0x10
   13d2c:	03059913          	slli	s2,a1,0x30
   13d30:	0107d593          	srli	a1,a5,0x10
   13d34:	0125e5b3          	or	a1,a1,s2
   13d38:	00068463          	beqz	a3,13d40 <__addtf3+0x454>
   13d3c:	0016a073          	csrs	fflags,a3
   13d40:	02813083          	ld	ra,40(sp)
   13d44:	02013403          	ld	s0,32(sp)
   13d48:	01813483          	ld	s1,24(sp)
   13d4c:	01013903          	ld	s2,16(sp)
   13d50:	00813983          	ld	s3,8(sp)
   13d54:	00013a03          	ld	s4,0(sp)
   13d58:	03010113          	addi	sp,sp,48
   13d5c:	00008067          	ret
   13d60:	03c88863          	beq	a7,t3,13d90 <__addtf3+0x4a4>
   13d64:	00c50633          	add	a2,a0,a2
   13d68:	00a63533          	sltu	a0,a2,a0
   13d6c:	00e787b3          	add	a5,a5,a4
   13d70:	00a787b3          	add	a5,a5,a0
   13d74:	03f79513          	slli	a0,a5,0x3f
   13d78:	00165613          	srli	a2,a2,0x1
   13d7c:	00c56533          	or	a0,a0,a2
   13d80:	0017d793          	srli	a5,a5,0x1
   13d84:	00088593          	mv	a1,a7
   13d88:	00000693          	li	a3,0
   13d8c:	e95ff06f          	j	13c20 <__addtf3+0x334>
   13d90:	00048863          	beqz	s1,13da0 <__addtf3+0x4b4>
   13d94:	00300793          	li	a5,3
   13d98:	00f49863          	bne	s1,a5,13da8 <__addtf3+0x4bc>
   13d9c:	00091c63          	bnez	s2,13db4 <__addtf3+0x4c8>
   13da0:	00088593          	mv	a1,a7
   13da4:	0680006f          	j	13e0c <__addtf3+0x520>
   13da8:	00200793          	li	a5,2
   13dac:	00f49463          	bne	s1,a5,13db4 <__addtf3+0x4c8>
   13db0:	fe0918e3          	bnez	s2,13da0 <__addtf3+0x4b4>
   13db4:	fff00793          	li	a5,-1
   13db8:	fff00513          	li	a0,-1
   13dbc:	00068593          	mv	a1,a3
   13dc0:	00500693          	li	a3,5
   13dc4:	00200713          	li	a4,2
   13dc8:	0016e693          	ori	a3,a3,1
   13dcc:	5ae48e63          	beq	s1,a4,14388 <__addtf3+0xa9c>
   13dd0:	00300713          	li	a4,3
   13dd4:	5ae48463          	beq	s1,a4,1437c <__addtf3+0xa90>
   13dd8:	5a049a63          	bnez	s1,1438c <__addtf3+0xaa0>
   13ddc:	00f57713          	andi	a4,a0,15
   13de0:	00400613          	li	a2,4
   13de4:	5ac70463          	beq	a4,a2,1438c <__addtf3+0xaa0>
   13de8:	00450713          	addi	a4,a0,4
   13dec:	00a73533          	sltu	a0,a4,a0
   13df0:	00a787b3          	add	a5,a5,a0
   13df4:	00070513          	mv	a0,a4
   13df8:	5940006f          	j	1438c <__addtf3+0xaa0>
   13dfc:	00048863          	beqz	s1,13e0c <__addtf3+0x520>
   13e00:	00300793          	li	a5,3
   13e04:	00f49c63          	bne	s1,a5,13e1c <__addtf3+0x530>
   13e08:	02091063          	bnez	s2,13e28 <__addtf3+0x53c>
   13e0c:	00000793          	li	a5,0
   13e10:	00000513          	li	a0,0
   13e14:	00500693          	li	a3,5
   13e18:	e9dff06f          	j	13cb4 <__addtf3+0x3c8>
   13e1c:	00200793          	li	a5,2
   13e20:	00f49463          	bne	s1,a5,13e28 <__addtf3+0x53c>
   13e24:	fe0914e3          	bnez	s2,13e0c <__addtf3+0x520>
   13e28:	fff00793          	li	a5,-1
   13e2c:	fff00513          	li	a0,-1
   13e30:	ffe68593          	addi	a1,a3,-2
   13e34:	00000813          	li	a6,0
   13e38:	f89ff06f          	j	13dc0 <__addtf3+0x4d4>
   13e3c:	0f005863          	blez	a6,13f2c <__addtf3+0x640>
   13e40:	08059c63          	bnez	a1,13ed8 <__addtf3+0x5ec>
   13e44:	00c765b3          	or	a1,a4,a2
   13e48:	00059a63          	bnez	a1,13e5c <__addtf3+0x570>
   13e4c:	bdc400e3          	beq	s0,t3,13a0c <__addtf3+0x120>
   13e50:	00078713          	mv	a4,a5
   13e54:	00040593          	mv	a1,s0
   13e58:	4480006f          	j	142a0 <__addtf3+0x9b4>
   13e5c:	fff6881b          	addiw	a6,a3,-1
   13e60:	02081e63          	bnez	a6,13e9c <__addtf3+0x5b0>
   13e64:	40c50633          	sub	a2,a0,a2
   13e68:	00c53533          	sltu	a0,a0,a2
   13e6c:	40e78733          	sub	a4,a5,a4
   13e70:	40a70733          	sub	a4,a4,a0
   13e74:	00040593          	mv	a1,s0
   13e78:	00060513          	mv	a0,a2
   13e7c:	03375793          	srli	a5,a4,0x33
   13e80:	0017f793          	andi	a5,a5,1
   13e84:	40078e63          	beqz	a5,142a0 <__addtf3+0x9b4>
   13e88:	00d71713          	slli	a4,a4,0xd
   13e8c:	00d75993          	srli	s3,a4,0xd
   13e90:	00050a13          	mv	s4,a0
   13e94:	00058413          	mv	s0,a1
   13e98:	3240006f          	j	141bc <__addtf3+0x8d0>
   13e9c:	b7c408e3          	beq	s0,t3,13a0c <__addtf3+0x120>
   13ea0:	07400693          	li	a3,116
   13ea4:	0706ce63          	blt	a3,a6,13f20 <__addtf3+0x634>
   13ea8:	03f00693          	li	a3,63
   13eac:	0506c063          	blt	a3,a6,13eec <__addtf3+0x600>
   13eb0:	04000593          	li	a1,64
   13eb4:	410585bb          	subw	a1,a1,a6
   13eb8:	00b716b3          	sll	a3,a4,a1
   13ebc:	010658b3          	srl	a7,a2,a6
   13ec0:	00b61633          	sll	a2,a2,a1
   13ec4:	0116e6b3          	or	a3,a3,a7
   13ec8:	00c03633          	snez	a2,a2
   13ecc:	00c6e633          	or	a2,a3,a2
   13ed0:	01075733          	srl	a4,a4,a6
   13ed4:	f91ff06f          	j	13e64 <__addtf3+0x578>
   13ed8:	b3c40ae3          	beq	s0,t3,13a0c <__addtf3+0x120>
   13edc:	00100693          	li	a3,1
   13ee0:	03369693          	slli	a3,a3,0x33
   13ee4:	00d76733          	or	a4,a4,a3
   13ee8:	fb9ff06f          	j	13ea0 <__addtf3+0x5b4>
   13eec:	fc08069b          	addiw	a3,a6,-64
   13ef0:	04000893          	li	a7,64
   13ef4:	00d756b3          	srl	a3,a4,a3
   13ef8:	00000593          	li	a1,0
   13efc:	01180863          	beq	a6,a7,13f0c <__addtf3+0x620>
   13f00:	08000593          	li	a1,128
   13f04:	410585bb          	subw	a1,a1,a6
   13f08:	00b715b3          	sll	a1,a4,a1
   13f0c:	00c5e633          	or	a2,a1,a2
   13f10:	00c03633          	snez	a2,a2
   13f14:	00c6e633          	or	a2,a3,a2
   13f18:	00000713          	li	a4,0
   13f1c:	f49ff06f          	j	13e64 <__addtf3+0x578>
   13f20:	00c76633          	or	a2,a4,a2
   13f24:	00c03633          	snez	a2,a2
   13f28:	ff1ff06f          	j	13f18 <__addtf3+0x62c>
   13f2c:	12080063          	beqz	a6,1404c <__addtf3+0x760>
   13f30:	08041063          	bnez	s0,13fb0 <__addtf3+0x6c4>
   13f34:	00a7e833          	or	a6,a5,a0
   13f38:	02081663          	bnez	a6,13f64 <__addtf3+0x678>
   13f3c:	37c59863          	bne	a1,t3,142ac <__addtf3+0x9c0>
   13f40:	00c76533          	or	a0,a4,a2
   13f44:	3e050663          	beqz	a0,14330 <__addtf3+0xa44>
   13f48:	03275793          	srli	a5,a4,0x32
   13f4c:	0017f793          	andi	a5,a5,1
   13f50:	3e079663          	bnez	a5,1433c <__addtf3+0xa50>
   13f54:	00070793          	mv	a5,a4
   13f58:	00060513          	mv	a0,a2
   13f5c:	00088913          	mv	s2,a7
   13f60:	b7dff06f          	j	13adc <__addtf3+0x1f0>
   13f64:	fff6c693          	not	a3,a3
   13f68:	0006869b          	sext.w	a3,a3
   13f6c:	00069e63          	bnez	a3,13f88 <__addtf3+0x69c>
   13f70:	40a60533          	sub	a0,a2,a0
   13f74:	40f70733          	sub	a4,a4,a5
   13f78:	00a63633          	sltu	a2,a2,a0
   13f7c:	40c70733          	sub	a4,a4,a2
   13f80:	00088913          	mv	s2,a7
   13f84:	ef9ff06f          	j	13e7c <__addtf3+0x590>
   13f88:	03c59e63          	bne	a1,t3,13fc4 <__addtf3+0x6d8>
   13f8c:	00c76533          	or	a0,a4,a2
   13f90:	3a050063          	beqz	a0,14330 <__addtf3+0xa44>
   13f94:	03275793          	srli	a5,a4,0x32
   13f98:	0017f793          	andi	a5,a5,1
   13f9c:	3a079063          	bnez	a5,1433c <__addtf3+0xa50>
   13fa0:	00070793          	mv	a5,a4
   13fa4:	00060513          	mv	a0,a2
   13fa8:	00088913          	mv	s2,a7
   13fac:	b79ff06f          	j	13b24 <__addtf3+0x238>
   13fb0:	fdc58ee3          	beq	a1,t3,13f8c <__addtf3+0x6a0>
   13fb4:	00100813          	li	a6,1
   13fb8:	03381813          	slli	a6,a6,0x33
   13fbc:	40d006bb          	negw	a3,a3
   13fc0:	0107e7b3          	or	a5,a5,a6
   13fc4:	07400813          	li	a6,116
   13fc8:	06d84c63          	blt	a6,a3,14040 <__addtf3+0x754>
   13fcc:	03f00813          	li	a6,63
   13fd0:	02d84e63          	blt	a6,a3,1400c <__addtf3+0x720>
   13fd4:	04000313          	li	t1,64
   13fd8:	40d3033b          	subw	t1,t1,a3
   13fdc:	00679833          	sll	a6,a5,t1
   13fe0:	00d55e33          	srl	t3,a0,a3
   13fe4:	00651533          	sll	a0,a0,t1
   13fe8:	01c86833          	or	a6,a6,t3
   13fec:	00a03533          	snez	a0,a0
   13ff0:	00a86533          	or	a0,a6,a0
   13ff4:	00d7d7b3          	srl	a5,a5,a3
   13ff8:	40a60533          	sub	a0,a2,a0
   13ffc:	40f707b3          	sub	a5,a4,a5
   14000:	00a63633          	sltu	a2,a2,a0
   14004:	40c78733          	sub	a4,a5,a2
   14008:	f79ff06f          	j	13f80 <__addtf3+0x694>
   1400c:	fc06881b          	addiw	a6,a3,-64
   14010:	04000e13          	li	t3,64
   14014:	0107d833          	srl	a6,a5,a6
   14018:	00000313          	li	t1,0
   1401c:	01c68863          	beq	a3,t3,1402c <__addtf3+0x740>
   14020:	08000313          	li	t1,128
   14024:	40d306bb          	subw	a3,t1,a3
   14028:	00d79333          	sll	t1,a5,a3
   1402c:	00a36533          	or	a0,t1,a0
   14030:	00a03533          	snez	a0,a0
   14034:	00a86533          	or	a0,a6,a0
   14038:	00000793          	li	a5,0
   1403c:	fbdff06f          	j	13ff8 <__addtf3+0x70c>
   14040:	00a7e533          	or	a0,a5,a0
   14044:	00a03533          	snez	a0,a0
   14048:	ff1ff06f          	j	14038 <__addtf3+0x74c>
   1404c:	00140e93          	addi	t4,s0,1
   14050:	ffe30693          	addi	a3,t1,-2
   14054:	00def6b3          	and	a3,t4,a3
   14058:	12069a63          	bnez	a3,1418c <__addtf3+0x8a0>
   1405c:	00a7eeb3          	or	t4,a5,a0
   14060:	00c76333          	or	t1,a4,a2
   14064:	0c041663          	bnez	s0,14130 <__addtf3+0x844>
   14068:	060e9c63          	bnez	t4,140e0 <__addtf3+0x7f4>
   1406c:	24031c63          	bnez	t1,142c4 <__addtf3+0x9d8>
   14070:	ffe48913          	addi	s2,s1,-2
   14074:	00193913          	seqz	s2,s2
   14078:	00000793          	li	a5,0
   1407c:	00000513          	li	a0,0
   14080:	00f56733          	or	a4,a0,a5
   14084:	2e070463          	beqz	a4,1436c <__addtf3+0xa80>
   14088:	03f55713          	srli	a4,a0,0x3f
   1408c:	00179813          	slli	a6,a5,0x1
   14090:	00e80833          	add	a6,a6,a4
   14094:	00151713          	slli	a4,a0,0x1
   14098:	00777613          	andi	a2,a4,7
   1409c:	00000693          	li	a3,0
   140a0:	02060663          	beqz	a2,140cc <__addtf3+0x7e0>
   140a4:	00200693          	li	a3,2
   140a8:	24d48a63          	beq	s1,a3,142fc <__addtf3+0xa10>
   140ac:	00300693          	li	a3,3
   140b0:	22d48663          	beq	s1,a3,142dc <__addtf3+0x9f0>
   140b4:	00100693          	li	a3,1
   140b8:	00049a63          	bnez	s1,140cc <__addtf3+0x7e0>
   140bc:	00f77613          	andi	a2,a4,15
   140c0:	00400593          	li	a1,4
   140c4:	ffc73713          	sltiu	a4,a4,-4
   140c8:	22b61063          	bne	a2,a1,142e8 <__addtf3+0x9fc>
   140cc:	03485813          	srli	a6,a6,0x34
   140d0:	00184813          	xori	a6,a6,1
   140d4:	00187813          	andi	a6,a6,1
   140d8:	00000593          	li	a1,0
   140dc:	b45ff06f          	j	13c20 <__addtf3+0x334>
   140e0:	fa0300e3          	beqz	t1,14080 <__addtf3+0x794>
   140e4:	40c505b3          	sub	a1,a0,a2
   140e8:	00b53833          	sltu	a6,a0,a1
   140ec:	40e786b3          	sub	a3,a5,a4
   140f0:	410686b3          	sub	a3,a3,a6
   140f4:	0336d813          	srli	a6,a3,0x33
   140f8:	00187813          	andi	a6,a6,1
   140fc:	00080e63          	beqz	a6,14118 <__addtf3+0x82c>
   14100:	40a60533          	sub	a0,a2,a0
   14104:	40f707b3          	sub	a5,a4,a5
   14108:	00a63633          	sltu	a2,a2,a0
   1410c:	40c787b3          	sub	a5,a5,a2
   14110:	00088913          	mv	s2,a7
   14114:	f6dff06f          	j	14080 <__addtf3+0x794>
   14118:	00d5e533          	or	a0,a1,a3
   1411c:	1a051a63          	bnez	a0,142d0 <__addtf3+0x9e4>
   14120:	ffe48913          	addi	s2,s1,-2
   14124:	00193913          	seqz	s2,s2
   14128:	00000793          	li	a5,0
   1412c:	f55ff06f          	j	14080 <__addtf3+0x794>
   14130:	03c41a63          	bne	s0,t3,14164 <__addtf3+0x878>
   14134:	280e8c63          	beqz	t4,143cc <__addtf3+0xae0>
   14138:	0327d693          	srli	a3,a5,0x32
   1413c:	0016f693          	andi	a3,a3,1
   14140:	0016b693          	seqz	a3,a3
   14144:	00469693          	slli	a3,a3,0x4
   14148:	02859e63          	bne	a1,s0,14184 <__addtf3+0x898>
   1414c:	02030063          	beqz	t1,1416c <__addtf3+0x880>
   14150:	03275593          	srli	a1,a4,0x32
   14154:	0015f593          	andi	a1,a1,1
   14158:	00059a63          	bnez	a1,1416c <__addtf3+0x880>
   1415c:	01000693          	li	a3,16
   14160:	00c0006f          	j	1416c <__addtf3+0x880>
   14164:	00000693          	li	a3,0
   14168:	ffc582e3          	beq	a1,t3,1414c <__addtf3+0x860>
   1416c:	000e9c63          	bnez	t4,14184 <__addtf3+0x898>
   14170:	1c030e63          	beqz	t1,1434c <__addtf3+0xa60>
   14174:	00070793          	mv	a5,a4
   14178:	00060513          	mv	a0,a2
   1417c:	00088913          	mv	s2,a7
   14180:	b09ff06f          	j	13c88 <__addtf3+0x39c>
   14184:	b00302e3          	beqz	t1,13c88 <__addtf3+0x39c>
   14188:	b15ff06f          	j	13c9c <__addtf3+0x3b0>
   1418c:	40c50a33          	sub	s4,a0,a2
   14190:	014536b3          	sltu	a3,a0,s4
   14194:	40e789b3          	sub	s3,a5,a4
   14198:	40d989b3          	sub	s3,s3,a3
   1419c:	0339d693          	srli	a3,s3,0x33
   141a0:	0016f693          	andi	a3,a3,1
   141a4:	08068663          	beqz	a3,14230 <__addtf3+0x944>
   141a8:	40a60a33          	sub	s4,a2,a0
   141ac:	40f70733          	sub	a4,a4,a5
   141b0:	01463633          	sltu	a2,a2,s4
   141b4:	40c709b3          	sub	s3,a4,a2
   141b8:	00088913          	mv	s2,a7
   141bc:	08098063          	beqz	s3,1423c <__addtf3+0x950>
   141c0:	00098513          	mv	a0,s3
   141c4:	1f9010ef          	jal	ra,15bbc <__clzdi2>
   141c8:	0005051b          	sext.w	a0,a0
   141cc:	ff45059b          	addiw	a1,a0,-12
   141d0:	03f00793          	li	a5,63
   141d4:	00058693          	mv	a3,a1
   141d8:	06b7ca63          	blt	a5,a1,1424c <__addtf3+0x960>
   141dc:	04000713          	li	a4,64
   141e0:	40b7073b          	subw	a4,a4,a1
   141e4:	00b999b3          	sll	s3,s3,a1
   141e8:	00ea5733          	srl	a4,s4,a4
   141ec:	01376733          	or	a4,a4,s3
   141f0:	00ba1533          	sll	a0,s4,a1
   141f4:	0885cc63          	blt	a1,s0,1428c <__addtf3+0x9a0>
   141f8:	4086843b          	subw	s0,a3,s0
   141fc:	0014079b          	addiw	a5,s0,1
   14200:	03f00693          	li	a3,63
   14204:	04f6cc63          	blt	a3,a5,1425c <__addtf3+0x970>
   14208:	04000693          	li	a3,64
   1420c:	40f686bb          	subw	a3,a3,a5
   14210:	00f555b3          	srl	a1,a0,a5
   14214:	00d71633          	sll	a2,a4,a3
   14218:	00d51533          	sll	a0,a0,a3
   1421c:	00b66633          	or	a2,a2,a1
   14220:	00a03533          	snez	a0,a0
   14224:	00a66533          	or	a0,a2,a0
   14228:	00f757b3          	srl	a5,a4,a5
   1422c:	e55ff06f          	j	14080 <__addtf3+0x794>
   14230:	013a6533          	or	a0,s4,s3
   14234:	f80514e3          	bnez	a0,141bc <__addtf3+0x8d0>
   14238:	ee9ff06f          	j	14120 <__addtf3+0x834>
   1423c:	000a0513          	mv	a0,s4
   14240:	17d010ef          	jal	ra,15bbc <__clzdi2>
   14244:	0405051b          	addiw	a0,a0,64
   14248:	f85ff06f          	j	141cc <__addtf3+0x8e0>
   1424c:	fb45071b          	addiw	a4,a0,-76
   14250:	00ea1733          	sll	a4,s4,a4
   14254:	00000513          	li	a0,0
   14258:	f9dff06f          	j	141f4 <__addtf3+0x908>
   1425c:	fc14041b          	addiw	s0,s0,-63
   14260:	04000593          	li	a1,64
   14264:	00875433          	srl	s0,a4,s0
   14268:	00000693          	li	a3,0
   1426c:	00b78863          	beq	a5,a1,1427c <__addtf3+0x990>
   14270:	08000693          	li	a3,128
   14274:	40f686bb          	subw	a3,a3,a5
   14278:	00d716b3          	sll	a3,a4,a3
   1427c:	00d56533          	or	a0,a0,a3
   14280:	00a03533          	snez	a0,a0
   14284:	00a46533          	or	a0,s0,a0
   14288:	ea1ff06f          	j	14128 <__addtf3+0x83c>
   1428c:	fff00793          	li	a5,-1
   14290:	03379793          	slli	a5,a5,0x33
   14294:	fff78793          	addi	a5,a5,-1
   14298:	40b405b3          	sub	a1,s0,a1
   1429c:	00f77733          	and	a4,a4,a5
   142a0:	00070793          	mv	a5,a4
   142a4:	dc058ee3          	beqz	a1,14080 <__addtf3+0x794>
   142a8:	f58ff06f          	j	13a00 <__addtf3+0x114>
   142ac:	00060513          	mv	a0,a2
   142b0:	00088913          	mv	s2,a7
   142b4:	fedff06f          	j	142a0 <__addtf3+0x9b4>
   142b8:	00070793          	mv	a5,a4
   142bc:	00060513          	mv	a0,a2
   142c0:	dc1ff06f          	j	14080 <__addtf3+0x794>
   142c4:	00070793          	mv	a5,a4
   142c8:	00060513          	mv	a0,a2
   142cc:	e45ff06f          	j	14110 <__addtf3+0x824>
   142d0:	00068793          	mv	a5,a3
   142d4:	00058513          	mv	a0,a1
   142d8:	da9ff06f          	j	14080 <__addtf3+0x794>
   142dc:	00100693          	li	a3,1
   142e0:	de0916e3          	bnez	s2,140cc <__addtf3+0x7e0>
   142e4:	ff873713          	sltiu	a4,a4,-8
   142e8:	00174713          	xori	a4,a4,1
   142ec:	02071713          	slli	a4,a4,0x20
   142f0:	02075713          	srli	a4,a4,0x20
   142f4:	00e80833          	add	a6,a6,a4
   142f8:	dd5ff06f          	j	140cc <__addtf3+0x7e0>
   142fc:	00100693          	li	a3,1
   14300:	dc0906e3          	beqz	s2,140cc <__addtf3+0x7e0>
   14304:	fe1ff06f          	j	142e4 <__addtf3+0x9f8>
   14308:	00000793          	li	a5,0
   1430c:	00000693          	li	a3,0
   14310:	9a5ff06f          	j	13cb4 <__addtf3+0x3c8>
   14314:	00070793          	mv	a5,a4
   14318:	00060513          	mv	a0,a2
   1431c:	ee4ff06f          	j	13a00 <__addtf3+0x114>
   14320:	00000793          	li	a5,0
   14324:	00000513          	li	a0,0
   14328:	00040593          	mv	a1,s0
   1432c:	fe1ff06f          	j	1430c <__addtf3+0xa20>
   14330:	00000793          	li	a5,0
   14334:	00088913          	mv	s2,a7
   14338:	fd5ff06f          	j	1430c <__addtf3+0xa20>
   1433c:	00070793          	mv	a5,a4
   14340:	00060513          	mv	a0,a2
   14344:	00088913          	mv	s2,a7
   14348:	eb8ff06f          	j	13a00 <__addtf3+0x114>
   1434c:	00100793          	li	a5,1
   14350:	000085b7          	lui	a1,0x8
   14354:	00000513          	li	a0,0
   14358:	00000913          	li	s2,0
   1435c:	03279793          	slli	a5,a5,0x32
   14360:	fff58593          	addi	a1,a1,-1 # 7fff <exit-0x8121>
   14364:	01000693          	li	a3,16
   14368:	94dff06f          	j	13cb4 <__addtf3+0x3c8>
   1436c:	00000793          	li	a5,0
   14370:	00000513          	li	a0,0
   14374:	00000593          	li	a1,0
   14378:	f95ff06f          	j	1430c <__addtf3+0xa20>
   1437c:	00091863          	bnez	s2,1438c <__addtf3+0xaa0>
   14380:	00850713          	addi	a4,a0,8
   14384:	a69ff06f          	j	13dec <__addtf3+0x500>
   14388:	fe091ce3          	bnez	s2,14380 <__addtf3+0xa94>
   1438c:	8a0814e3          	bnez	a6,13c34 <__addtf3+0x348>
   14390:	925ff06f          	j	13cb4 <__addtf3+0x3c8>
   14394:	00000513          	li	a0,0
   14398:	02048463          	beqz	s1,143c0 <__addtf3+0xad4>
   1439c:	00300793          	li	a5,3
   143a0:	00f49a63          	bne	s1,a5,143b4 <__addtf3+0xac8>
   143a4:	00090e63          	beqz	s2,143c0 <__addtf3+0xad4>
   143a8:	fff00513          	li	a0,-1
   143ac:	ffe70593          	addi	a1,a4,-2
   143b0:	0100006f          	j	143c0 <__addtf3+0xad4>
   143b4:	00200793          	li	a5,2
   143b8:	fef498e3          	bne	s1,a5,143a8 <__addtf3+0xabc>
   143bc:	fe0906e3          	beqz	s2,143a8 <__addtf3+0xabc>
   143c0:	0056e693          	ori	a3,a3,5
   143c4:	00050793          	mv	a5,a0
   143c8:	919ff06f          	j	13ce0 <__addtf3+0x3f4>
   143cc:	00000693          	li	a3,0
   143d0:	da8590e3          	bne	a1,s0,14170 <__addtf3+0x884>
   143d4:	d79ff06f          	j	1414c <__addtf3+0x860>
   143d8:	00000693          	li	a3,0
   143dc:	8a8592e3          	bne	a1,s0,13c80 <__addtf3+0x394>
   143e0:	879ff06f          	j	13c58 <__addtf3+0x36c>

00000000000143e4 <__eqtf2>:
   143e4:	00050813          	mv	a6,a0
   143e8:	002027f3          	frrm	a5
   143ec:	00008537          	lui	a0,0x8
   143f0:	0305d893          	srli	a7,a1,0x30
   143f4:	fff50513          	addi	a0,a0,-1 # 7fff <exit-0x8121>
   143f8:	fff00793          	li	a5,-1
   143fc:	0107d793          	srli	a5,a5,0x10
   14400:	0306d313          	srli	t1,a3,0x30
   14404:	00a8f8b3          	and	a7,a7,a0
   14408:	00f5f733          	and	a4,a1,a5
   1440c:	00a37333          	and	t1,t1,a0
   14410:	00f6f7b3          	and	a5,a3,a5
   14414:	03f5d593          	srli	a1,a1,0x3f
   14418:	03f6d693          	srli	a3,a3,0x3f
   1441c:	00a89a63          	bne	a7,a0,14430 <__eqtf2+0x4c>
   14420:	01076533          	or	a0,a4,a6
   14424:	02051663          	bnez	a0,14450 <__eqtf2+0x6c>
   14428:	05131863          	bne	t1,a7,14478 <__eqtf2+0x94>
   1442c:	0080006f          	j	14434 <__eqtf2+0x50>
   14430:	04a31863          	bne	t1,a0,14480 <__eqtf2+0x9c>
   14434:	00c7e533          	or	a0,a5,a2
   14438:	04050463          	beqz	a0,14480 <__eqtf2+0x9c>
   1443c:	000086b7          	lui	a3,0x8
   14440:	fff68693          	addi	a3,a3,-1 # 7fff <exit-0x8121>
   14444:	02d89663          	bne	a7,a3,14470 <__eqtf2+0x8c>
   14448:	01076533          	or	a0,a4,a6
   1444c:	02050263          	beqz	a0,14470 <__eqtf2+0x8c>
   14450:	02f75713          	srli	a4,a4,0x2f
   14454:	04070c63          	beqz	a4,144ac <__eqtf2+0xc8>
   14458:	00008737          	lui	a4,0x8
   1445c:	fff70713          	addi	a4,a4,-1 # 7fff <exit-0x8121>
   14460:	00100513          	li	a0,1
   14464:	00e31c63          	bne	t1,a4,1447c <__eqtf2+0x98>
   14468:	00c7e633          	or	a2,a5,a2
   1446c:	00060863          	beqz	a2,1447c <__eqtf2+0x98>
   14470:	02f7d793          	srli	a5,a5,0x2f
   14474:	02078c63          	beqz	a5,144ac <__eqtf2+0xc8>
   14478:	00100513          	li	a0,1
   1447c:	00008067          	ret
   14480:	00100513          	li	a0,1
   14484:	fe689ce3          	bne	a7,t1,1447c <__eqtf2+0x98>
   14488:	fef71ae3          	bne	a4,a5,1447c <__eqtf2+0x98>
   1448c:	fec818e3          	bne	a6,a2,1447c <__eqtf2+0x98>
   14490:	00d58a63          	beq	a1,a3,144a4 <__eqtf2+0xc0>
   14494:	fe0894e3          	bnez	a7,1447c <__eqtf2+0x98>
   14498:	01076533          	or	a0,a4,a6
   1449c:	00a03533          	snez	a0,a0
   144a0:	00008067          	ret
   144a4:	00000513          	li	a0,0
   144a8:	00008067          	ret
   144ac:	00186073          	csrsi	fflags,16
   144b0:	fc9ff06f          	j	14478 <__eqtf2+0x94>

00000000000144b4 <__multf3>:
   144b4:	fa010113          	addi	sp,sp,-96
   144b8:	03613023          	sd	s6,32(sp)
   144bc:	01913423          	sd	s9,8(sp)
   144c0:	04113c23          	sd	ra,88(sp)
   144c4:	04813823          	sd	s0,80(sp)
   144c8:	04913423          	sd	s1,72(sp)
   144cc:	05213023          	sd	s2,64(sp)
   144d0:	03313c23          	sd	s3,56(sp)
   144d4:	03413823          	sd	s4,48(sp)
   144d8:	03513423          	sd	s5,40(sp)
   144dc:	01713c23          	sd	s7,24(sp)
   144e0:	01813823          	sd	s8,16(sp)
   144e4:	00060c93          	mv	s9,a2
   144e8:	00068b13          	mv	s6,a3
   144ec:	00202973          	frrm	s2
   144f0:	00008737          	lui	a4,0x8
   144f4:	0305da93          	srli	s5,a1,0x30
   144f8:	fff70713          	addi	a4,a4,-1 # 7fff <exit-0x8121>
   144fc:	01059993          	slli	s3,a1,0x10
   14500:	00eaf7b3          	and	a5,s5,a4
   14504:	0009091b          	sext.w	s2,s2
   14508:	0109d993          	srli	s3,s3,0x10
   1450c:	03f5db93          	srli	s7,a1,0x3f
   14510:	04078263          	beqz	a5,14554 <__multf3+0xa0>
   14514:	00050a13          	mv	s4,a0
   14518:	00078a9b          	sext.w	s5,a5
   1451c:	0ae78663          	beq	a5,a4,145c8 <__multf3+0x114>
   14520:	00399993          	slli	s3,s3,0x3
   14524:	03d55713          	srli	a4,a0,0x3d
   14528:	01376733          	or	a4,a4,s3
   1452c:	ffffcab7          	lui	s5,0xffffc
   14530:	00100993          	li	s3,1
   14534:	03399993          	slli	s3,s3,0x33
   14538:	001a8a93          	addi	s5,s5,1 # ffffffffffffc001 <__global_pointer$+0xfffffffffffe3801>
   1453c:	013769b3          	or	s3,a4,s3
   14540:	00351a13          	slli	s4,a0,0x3
   14544:	01578ab3          	add	s5,a5,s5
   14548:	00000c13          	li	s8,0
   1454c:	00000493          	li	s1,0
   14550:	0940006f          	j	145e4 <__multf3+0x130>
   14554:	00a9e7b3          	or	a5,s3,a0
   14558:	00050413          	mv	s0,a0
   1455c:	10078e63          	beqz	a5,14678 <__multf3+0x1c4>
   14560:	04098663          	beqz	s3,145ac <__multf3+0xf8>
   14564:	00098513          	mv	a0,s3
   14568:	654010ef          	jal	ra,15bbc <__clzdi2>
   1456c:	0005079b          	sext.w	a5,a0
   14570:	ff178693          	addi	a3,a5,-15
   14574:	03c00613          	li	a2,60
   14578:	0006871b          	sext.w	a4,a3
   1457c:	02d64e63          	blt	a2,a3,145b8 <__multf3+0x104>
   14580:	03d00693          	li	a3,61
   14584:	00370a1b          	addiw	s4,a4,3
   14588:	40e6873b          	subw	a4,a3,a4
   1458c:	014999b3          	sll	s3,s3,s4
   14590:	00e45733          	srl	a4,s0,a4
   14594:	013769b3          	or	s3,a4,s3
   14598:	01441a33          	sll	s4,s0,s4
   1459c:	ffffcab7          	lui	s5,0xffffc
   145a0:	011a8a93          	addi	s5,s5,17 # ffffffffffffc011 <__global_pointer$+0xfffffffffffe3811>
   145a4:	40fa8ab3          	sub	s5,s5,a5
   145a8:	fa1ff06f          	j	14548 <__multf3+0x94>
   145ac:	610010ef          	jal	ra,15bbc <__clzdi2>
   145b0:	0405079b          	addiw	a5,a0,64
   145b4:	fbdff06f          	j	14570 <__multf3+0xbc>
   145b8:	fc37071b          	addiw	a4,a4,-61
   145bc:	00e419b3          	sll	s3,s0,a4
   145c0:	00000a13          	li	s4,0
   145c4:	fd9ff06f          	j	1459c <__multf3+0xe8>
   145c8:	00a9e7b3          	or	a5,s3,a0
   145cc:	0c078063          	beqz	a5,1468c <__multf3+0x1d8>
   145d0:	02f9d793          	srli	a5,s3,0x2f
   145d4:	0017f793          	andi	a5,a5,1
   145d8:	00300c13          	li	s8,3
   145dc:	01000493          	li	s1,16
   145e0:	f60796e3          	bnez	a5,1454c <__multf3+0x98>
   145e4:	000086b7          	lui	a3,0x8
   145e8:	030b5513          	srli	a0,s6,0x30
   145ec:	fff68693          	addi	a3,a3,-1 # 7fff <exit-0x8121>
   145f0:	010b1413          	slli	s0,s6,0x10
   145f4:	00d57733          	and	a4,a0,a3
   145f8:	000c8793          	mv	a5,s9
   145fc:	01045413          	srli	s0,s0,0x10
   14600:	03fb5b13          	srli	s6,s6,0x3f
   14604:	08070c63          	beqz	a4,1469c <__multf3+0x1e8>
   14608:	0007051b          	sext.w	a0,a4
   1460c:	10d70263          	beq	a4,a3,14710 <__multf3+0x25c>
   14610:	00341413          	slli	s0,s0,0x3
   14614:	03dcd693          	srli	a3,s9,0x3d
   14618:	0086e6b3          	or	a3,a3,s0
   1461c:	ffffc537          	lui	a0,0xffffc
   14620:	00100413          	li	s0,1
   14624:	03341413          	slli	s0,s0,0x33
   14628:	00150513          	addi	a0,a0,1 # ffffffffffffc001 <__global_pointer$+0xfffffffffffe3801>
   1462c:	0086e433          	or	s0,a3,s0
   14630:	003c9793          	slli	a5,s9,0x3
   14634:	00a70533          	add	a0,a4,a0
   14638:	00000713          	li	a4,0
   1463c:	002c1693          	slli	a3,s8,0x2
   14640:	00e6e6b3          	or	a3,a3,a4
   14644:	01550ab3          	add	s5,a0,s5
   14648:	fff68693          	addi	a3,a3,-1
   1464c:	00e00593          	li	a1,14
   14650:	016bc633          	xor	a2,s7,s6
   14654:	001a8813          	addi	a6,s5,1
   14658:	16d5ee63          	bltu	a1,a3,147d4 <__multf3+0x320>
   1465c:	00002597          	auipc	a1,0x2
   14660:	18458593          	addi	a1,a1,388 # 167e0 <errmsg+0x730>
   14664:	00269693          	slli	a3,a3,0x2
   14668:	00b686b3          	add	a3,a3,a1
   1466c:	0006a683          	lw	a3,0(a3)
   14670:	00b686b3          	add	a3,a3,a1
   14674:	00068067          	jr	a3
   14678:	00000993          	li	s3,0
   1467c:	00000a13          	li	s4,0
   14680:	00000a93          	li	s5,0
   14684:	00100c13          	li	s8,1
   14688:	ec5ff06f          	j	1454c <__multf3+0x98>
   1468c:	00000993          	li	s3,0
   14690:	00000a13          	li	s4,0
   14694:	00200c13          	li	s8,2
   14698:	eb5ff06f          	j	1454c <__multf3+0x98>
   1469c:	008ce7b3          	or	a5,s9,s0
   146a0:	08078863          	beqz	a5,14730 <__multf3+0x27c>
   146a4:	04040663          	beqz	s0,146f0 <__multf3+0x23c>
   146a8:	00040513          	mv	a0,s0
   146ac:	510010ef          	jal	ra,15bbc <__clzdi2>
   146b0:	0005071b          	sext.w	a4,a0
   146b4:	ff170793          	addi	a5,a4,-15
   146b8:	03c00613          	li	a2,60
   146bc:	0007869b          	sext.w	a3,a5
   146c0:	04f64063          	blt	a2,a5,14700 <__multf3+0x24c>
   146c4:	03d00613          	li	a2,61
   146c8:	0036879b          	addiw	a5,a3,3
   146cc:	40d606bb          	subw	a3,a2,a3
   146d0:	00f41433          	sll	s0,s0,a5
   146d4:	00dcd6b3          	srl	a3,s9,a3
   146d8:	0086e433          	or	s0,a3,s0
   146dc:	00fc97b3          	sll	a5,s9,a5
   146e0:	ffffc537          	lui	a0,0xffffc
   146e4:	01150513          	addi	a0,a0,17 # ffffffffffffc011 <__global_pointer$+0xfffffffffffe3811>
   146e8:	40e50533          	sub	a0,a0,a4
   146ec:	f4dff06f          	j	14638 <__multf3+0x184>
   146f0:	000c8513          	mv	a0,s9
   146f4:	4c8010ef          	jal	ra,15bbc <__clzdi2>
   146f8:	0405071b          	addiw	a4,a0,64
   146fc:	fb9ff06f          	j	146b4 <__multf3+0x200>
   14700:	fc36869b          	addiw	a3,a3,-61
   14704:	00dc9433          	sll	s0,s9,a3
   14708:	00000793          	li	a5,0
   1470c:	fd5ff06f          	j	146e0 <__multf3+0x22c>
   14710:	008ce733          	or	a4,s9,s0
   14714:	02070863          	beqz	a4,14744 <__multf3+0x290>
   14718:	02f45713          	srli	a4,s0,0x2f
   1471c:	00177713          	andi	a4,a4,1
   14720:	02071a63          	bnez	a4,14754 <__multf3+0x2a0>
   14724:	00300713          	li	a4,3
   14728:	01000493          	li	s1,16
   1472c:	f11ff06f          	j	1463c <__multf3+0x188>
   14730:	00000413          	li	s0,0
   14734:	00000793          	li	a5,0
   14738:	00000513          	li	a0,0
   1473c:	00100713          	li	a4,1
   14740:	efdff06f          	j	1463c <__multf3+0x188>
   14744:	00000413          	li	s0,0
   14748:	00000793          	li	a5,0
   1474c:	00200713          	li	a4,2
   14750:	eedff06f          	j	1463c <__multf3+0x188>
   14754:	00300713          	li	a4,3
   14758:	ee5ff06f          	j	1463c <__multf3+0x188>
   1475c:	00100413          	li	s0,1
   14760:	00008737          	lui	a4,0x8
   14764:	02f41413          	slli	s0,s0,0x2f
   14768:	00000513          	li	a0,0
   1476c:	fff70713          	addi	a4,a4,-1 # 7fff <exit-0x8121>
   14770:	00000613          	li	a2,0
   14774:	01000493          	li	s1,16
   14778:	03171713          	slli	a4,a4,0x31
   1477c:	03175713          	srli	a4,a4,0x31
   14780:	00f61613          	slli	a2,a2,0xf
   14784:	01041413          	slli	s0,s0,0x10
   14788:	00e66733          	or	a4,a2,a4
   1478c:	03071713          	slli	a4,a4,0x30
   14790:	01045593          	srli	a1,s0,0x10
   14794:	00e5e5b3          	or	a1,a1,a4
   14798:	00048463          	beqz	s1,147a0 <__multf3+0x2ec>
   1479c:	0014a073          	csrs	fflags,s1
   147a0:	05813083          	ld	ra,88(sp)
   147a4:	05013403          	ld	s0,80(sp)
   147a8:	04813483          	ld	s1,72(sp)
   147ac:	04013903          	ld	s2,64(sp)
   147b0:	03813983          	ld	s3,56(sp)
   147b4:	03013a03          	ld	s4,48(sp)
   147b8:	02813a83          	ld	s5,40(sp)
   147bc:	02013b03          	ld	s6,32(sp)
   147c0:	01813b83          	ld	s7,24(sp)
   147c4:	01013c03          	ld	s8,16(sp)
   147c8:	00813c83          	ld	s9,8(sp)
   147cc:	06010113          	addi	sp,sp,96
   147d0:	00008067          	ret
   147d4:	fff00513          	li	a0,-1
   147d8:	02055513          	srli	a0,a0,0x20
   147dc:	014983b3          	add	t2,s3,s4
   147e0:	00878f33          	add	t5,a5,s0
   147e4:	020a5e93          	srli	t4,s4,0x20
   147e8:	0207d693          	srli	a3,a5,0x20
   147ec:	0143bfb3          	sltu	t6,t2,s4
   147f0:	00ff32b3          	sltu	t0,t5,a5
   147f4:	00aa7a33          	and	s4,s4,a0
   147f8:	00a7f7b3          	and	a5,a5,a0
   147fc:	03478533          	mul	a0,a5,s4
   14800:	000f8e13          	mv	t3,t6
   14804:	00028313          	mv	t1,t0
   14808:	02fe87b3          	mul	a5,t4,a5
   1480c:	02de8733          	mul	a4,t4,a3
   14810:	034686b3          	mul	a3,a3,s4
   14814:	02055a13          	srli	s4,a0,0x20
   14818:	00f686b3          	add	a3,a3,a5
   1481c:	00da0a33          	add	s4,s4,a3
   14820:	00fa7863          	bgeu	s4,a5,14830 <__multf3+0x37c>
   14824:	00100793          	li	a5,1
   14828:	02079793          	slli	a5,a5,0x20
   1482c:	00f70733          	add	a4,a4,a5
   14830:	fff00693          	li	a3,-1
   14834:	0206d693          	srli	a3,a3,0x20
   14838:	00da75b3          	and	a1,s4,a3
   1483c:	00d57533          	and	a0,a0,a3
   14840:	02059593          	slli	a1,a1,0x20
   14844:	0203d893          	srli	a7,t2,0x20
   14848:	020f5793          	srli	a5,t5,0x20
   1484c:	020a5e93          	srli	t4,s4,0x20
   14850:	00a585b3          	add	a1,a1,a0
   14854:	00d3f533          	and	a0,t2,a3
   14858:	00df76b3          	and	a3,t5,a3
   1485c:	02f88b33          	mul	s6,a7,a5
   14860:	00ee8eb3          	add	t4,t4,a4
   14864:	02a787b3          	mul	a5,a5,a0
   14868:	02a68733          	mul	a4,a3,a0
   1486c:	02d886b3          	mul	a3,a7,a3
   14870:	00d78533          	add	a0,a5,a3
   14874:	02075793          	srli	a5,a4,0x20
   14878:	00a787b3          	add	a5,a5,a0
   1487c:	00d7f863          	bgeu	a5,a3,1488c <__multf3+0x3d8>
   14880:	00100693          	li	a3,1
   14884:	02069693          	slli	a3,a3,0x20
   14888:	00db0b33          	add	s6,s6,a3
   1488c:	fff00893          	li	a7,-1
   14890:	0208d893          	srli	a7,a7,0x20
   14894:	0117f6b3          	and	a3,a5,a7
   14898:	02045b93          	srli	s7,s0,0x20
   1489c:	01177533          	and	a0,a4,a7
   148a0:	0207da13          	srli	s4,a5,0x20
   148a4:	0209d793          	srli	a5,s3,0x20
   148a8:	0119f9b3          	and	s3,s3,a7
   148ac:	011478b3          	and	a7,s0,a7
   148b0:	03198433          	mul	s0,s3,a7
   148b4:	02069693          	slli	a3,a3,0x20
   148b8:	00a68533          	add	a0,a3,a0
   148bc:	031788b3          	mul	a7,a5,a7
   148c0:	037787b3          	mul	a5,a5,s7
   148c4:	033b8bb3          	mul	s7,s7,s3
   148c8:	02045993          	srli	s3,s0,0x20
   148cc:	011b8bb3          	add	s7,s7,a7
   148d0:	017989b3          	add	s3,s3,s7
   148d4:	0119f863          	bgeu	s3,a7,148e4 <__multf3+0x430>
   148d8:	00100713          	li	a4,1
   148dc:	02071713          	slli	a4,a4,0x20
   148e0:	00e787b3          	add	a5,a5,a4
   148e4:	0209d693          	srli	a3,s3,0x20
   148e8:	00f687b3          	add	a5,a3,a5
   148ec:	fff00693          	li	a3,-1
   148f0:	0206d693          	srli	a3,a3,0x20
   148f4:	00d9f733          	and	a4,s3,a3
   148f8:	00d47433          	and	s0,s0,a3
   148fc:	01d506b3          	add	a3,a0,t4
   14900:	01d6b533          	sltu	a0,a3,t4
   14904:	00aa08b3          	add	a7,s4,a0
   14908:	40600333          	neg	t1,t1
   1490c:	016888b3          	add	a7,a7,s6
   14910:	007373b3          	and	t2,t1,t2
   14914:	41c00e33          	neg	t3,t3
   14918:	011383b3          	add	t2,t2,a7
   1491c:	01ee7f33          	and	t5,t3,t5
   14920:	00a8b533          	sltu	a0,a7,a0
   14924:	005fffb3          	and	t6,t6,t0
   14928:	007f0f33          	add	t5,t5,t2
   1492c:	0113b8b3          	sltu	a7,t2,a7
   14930:	01f50533          	add	a0,a0,t6
   14934:	40b68333          	sub	t1,a3,a1
   14938:	01150533          	add	a0,a0,a7
   1493c:	0066be33          	sltu	t3,a3,t1
   14940:	41df08b3          	sub	a7,t5,t4
   14944:	02071713          	slli	a4,a4,0x20
   14948:	007f33b3          	sltu	t2,t5,t2
   1494c:	011f3fb3          	sltu	t6,t5,a7
   14950:	00870733          	add	a4,a4,s0
   14954:	41c888b3          	sub	a7,a7,t3
   14958:	00750533          	add	a0,a0,t2
   1495c:	00000e13          	li	t3,0
   14960:	0066f663          	bgeu	a3,t1,1496c <__multf3+0x4b8>
   14964:	41ee8f33          	sub	t5,t4,t5
   14968:	001f3e13          	seqz	t3,t5
   1496c:	01fe6f33          	or	t5,t3,t6
   14970:	40e30e33          	sub	t3,t1,a4
   14974:	40f886b3          	sub	a3,a7,a5
   14978:	00f50533          	add	a0,a0,a5
   1497c:	01c337b3          	sltu	a5,t1,t3
   14980:	00d8b8b3          	sltu	a7,a7,a3
   14984:	40f707b3          	sub	a5,a4,a5
   14988:	00000e93          	li	t4,0
   1498c:	01c37463          	bgeu	t1,t3,14994 <__multf3+0x4e0>
   14990:	0016be93          	seqz	t4,a3
   14994:	00f686b3          	add	a3,a3,a5
   14998:	00e6b733          	sltu	a4,a3,a4
   1499c:	00a70733          	add	a4,a4,a0
   149a0:	41e70733          	sub	a4,a4,t5
   149a4:	011ee8b3          	or	a7,t4,a7
   149a8:	41170733          	sub	a4,a4,a7
   149ac:	00de1793          	slli	a5,t3,0xd
   149b0:	00d71713          	slli	a4,a4,0xd
   149b4:	0336d413          	srli	s0,a3,0x33
   149b8:	00b7e7b3          	or	a5,a5,a1
   149bc:	00876433          	or	s0,a4,s0
   149c0:	00f037b3          	snez	a5,a5
   149c4:	033e5513          	srli	a0,t3,0x33
   149c8:	03475713          	srli	a4,a4,0x34
   149cc:	00a7e7b3          	or	a5,a5,a0
   149d0:	00d69693          	slli	a3,a3,0xd
   149d4:	00177713          	andi	a4,a4,1
   149d8:	00d7e7b3          	or	a5,a5,a3
   149dc:	0a070463          	beqz	a4,14a84 <__multf3+0x5d0>
   149e0:	0017d713          	srli	a4,a5,0x1
   149e4:	0017f793          	andi	a5,a5,1
   149e8:	03f41513          	slli	a0,s0,0x3f
   149ec:	00f767b3          	or	a5,a4,a5
   149f0:	00a7e7b3          	or	a5,a5,a0
   149f4:	00145413          	srli	s0,s0,0x1
   149f8:	00004737          	lui	a4,0x4
   149fc:	fff70713          	addi	a4,a4,-1 # 3fff <exit-0xc121>
   14a00:	00e80733          	add	a4,a6,a4
   14a04:	12e05263          	blez	a4,14b28 <__multf3+0x674>
   14a08:	0077f693          	andi	a3,a5,7
   14a0c:	08068863          	beqz	a3,14a9c <__multf3+0x5e8>
   14a10:	00200693          	li	a3,2
   14a14:	0014e493          	ori	s1,s1,1
   14a18:	08d90063          	beq	s2,a3,14a98 <__multf3+0x5e4>
   14a1c:	00300693          	li	a3,3
   14a20:	06d90663          	beq	s2,a3,14a8c <__multf3+0x5d8>
   14a24:	06091c63          	bnez	s2,14a9c <__multf3+0x5e8>
   14a28:	00f7f693          	andi	a3,a5,15
   14a2c:	00400593          	li	a1,4
   14a30:	06b68663          	beq	a3,a1,14a9c <__multf3+0x5e8>
   14a34:	00478693          	addi	a3,a5,4
   14a38:	00f6b7b3          	sltu	a5,a3,a5
   14a3c:	00f40433          	add	s0,s0,a5
   14a40:	00068793          	mv	a5,a3
   14a44:	0580006f          	j	14a9c <__multf3+0x5e8>
   14a48:	000b8613          	mv	a2,s7
   14a4c:	00098413          	mv	s0,s3
   14a50:	000a0793          	mv	a5,s4
   14a54:	000c0713          	mv	a4,s8
   14a58:	00200693          	li	a3,2
   14a5c:	2ad70063          	beq	a4,a3,14cfc <__multf3+0x848>
   14a60:	00300693          	li	a3,3
   14a64:	2ad70663          	beq	a4,a3,14d10 <__multf3+0x85c>
   14a68:	00100693          	li	a3,1
   14a6c:	f8d716e3          	bne	a4,a3,149f8 <__multf3+0x544>
   14a70:	00000413          	li	s0,0
   14a74:	00000513          	li	a0,0
   14a78:	2600006f          	j	14cd8 <__multf3+0x824>
   14a7c:	000b0613          	mv	a2,s6
   14a80:	fd9ff06f          	j	14a58 <__multf3+0x5a4>
   14a84:	000a8813          	mv	a6,s5
   14a88:	f71ff06f          	j	149f8 <__multf3+0x544>
   14a8c:	00061863          	bnez	a2,14a9c <__multf3+0x5e8>
   14a90:	00878693          	addi	a3,a5,8
   14a94:	fa5ff06f          	j	14a38 <__multf3+0x584>
   14a98:	fe061ce3          	bnez	a2,14a90 <__multf3+0x5dc>
   14a9c:	03445693          	srli	a3,s0,0x34
   14aa0:	0016f693          	andi	a3,a3,1
   14aa4:	00068e63          	beqz	a3,14ac0 <__multf3+0x60c>
   14aa8:	fff00713          	li	a4,-1
   14aac:	03471713          	slli	a4,a4,0x34
   14ab0:	fff70713          	addi	a4,a4,-1
   14ab4:	00e47433          	and	s0,s0,a4
   14ab8:	00004737          	lui	a4,0x4
   14abc:	00e80733          	add	a4,a6,a4
   14ac0:	000086b7          	lui	a3,0x8
   14ac4:	ffe68593          	addi	a1,a3,-2 # 7ffe <exit-0x8122>
   14ac8:	00e5cc63          	blt	a1,a4,14ae0 <__multf3+0x62c>
   14acc:	03d41513          	slli	a0,s0,0x3d
   14ad0:	0037d793          	srli	a5,a5,0x3
   14ad4:	00f56533          	or	a0,a0,a5
   14ad8:	00345413          	srli	s0,s0,0x3
   14adc:	c9dff06f          	j	14778 <__multf3+0x2c4>
   14ae0:	00200793          	li	a5,2
   14ae4:	02f90a63          	beq	s2,a5,14b18 <__multf3+0x664>
   14ae8:	00300793          	li	a5,3
   14aec:	fff68713          	addi	a4,a3,-1
   14af0:	00f90863          	beq	s2,a5,14b00 <__multf3+0x64c>
   14af4:	00091863          	bnez	s2,14b04 <__multf3+0x650>
   14af8:	00000513          	li	a0,0
   14afc:	0100006f          	j	14b0c <__multf3+0x658>
   14b00:	fe060ce3          	beqz	a2,14af8 <__multf3+0x644>
   14b04:	fff00513          	li	a0,-1
   14b08:	00058713          	mv	a4,a1
   14b0c:	0054e493          	ori	s1,s1,5
   14b10:	00050413          	mv	s0,a0
   14b14:	c65ff06f          	j	14778 <__multf3+0x2c4>
   14b18:	fe0606e3          	beqz	a2,14b04 <__multf3+0x650>
   14b1c:	00000513          	li	a0,0
   14b20:	fff68713          	addi	a4,a3,-1
   14b24:	fe9ff06f          	j	14b0c <__multf3+0x658>
   14b28:	00100693          	li	a3,1
   14b2c:	06071463          	bnez	a4,14b94 <__multf3+0x6e0>
   14b30:	0077f593          	andi	a1,a5,7
   14b34:	00040693          	mv	a3,s0
   14b38:	04058863          	beqz	a1,14b88 <__multf3+0x6d4>
   14b3c:	00200593          	li	a1,2
   14b40:	0014e493          	ori	s1,s1,1
   14b44:	04b90063          	beq	s2,a1,14b84 <__multf3+0x6d0>
   14b48:	00300593          	li	a1,3
   14b4c:	02b90663          	beq	s2,a1,14b78 <__multf3+0x6c4>
   14b50:	02091c63          	bnez	s2,14b88 <__multf3+0x6d4>
   14b54:	00f7f593          	andi	a1,a5,15
   14b58:	00400513          	li	a0,4
   14b5c:	02a58663          	beq	a1,a0,14b88 <__multf3+0x6d4>
   14b60:	ffc7b693          	sltiu	a3,a5,-4
   14b64:	0016c693          	xori	a3,a3,1
   14b68:	02069693          	slli	a3,a3,0x20
   14b6c:	0206d693          	srli	a3,a3,0x20
   14b70:	008686b3          	add	a3,a3,s0
   14b74:	0140006f          	j	14b88 <__multf3+0x6d4>
   14b78:	00061863          	bnez	a2,14b88 <__multf3+0x6d4>
   14b7c:	ff87b693          	sltiu	a3,a5,-8
   14b80:	fe5ff06f          	j	14b64 <__multf3+0x6b0>
   14b84:	fe061ce3          	bnez	a2,14b7c <__multf3+0x6c8>
   14b88:	0346d693          	srli	a3,a3,0x34
   14b8c:	0016c693          	xori	a3,a3,1
   14b90:	0016f693          	andi	a3,a3,1
   14b94:	00100813          	li	a6,1
   14b98:	40e80833          	sub	a6,a6,a4
   14b9c:	07400713          	li	a4,116
   14ba0:	11074263          	blt	a4,a6,14ca4 <__multf3+0x7f0>
   14ba4:	03f00593          	li	a1,63
   14ba8:	0008071b          	sext.w	a4,a6
   14bac:	0705c463          	blt	a1,a6,14c14 <__multf3+0x760>
   14bb0:	04000593          	li	a1,64
   14bb4:	40e585bb          	subw	a1,a1,a4
   14bb8:	00b41533          	sll	a0,s0,a1
   14bbc:	00e7d833          	srl	a6,a5,a4
   14bc0:	00b797b3          	sll	a5,a5,a1
   14bc4:	01056533          	or	a0,a0,a6
   14bc8:	00f037b3          	snez	a5,a5
   14bcc:	00f56533          	or	a0,a0,a5
   14bd0:	00e45433          	srl	s0,s0,a4
   14bd4:	00757793          	andi	a5,a0,7
   14bd8:	08078063          	beqz	a5,14c58 <__multf3+0x7a4>
   14bdc:	00200793          	li	a5,2
   14be0:	0014e493          	ori	s1,s1,1
   14be4:	06f90863          	beq	s2,a5,14c54 <__multf3+0x7a0>
   14be8:	00300793          	li	a5,3
   14bec:	04f90e63          	beq	s2,a5,14c48 <__multf3+0x794>
   14bf0:	06091463          	bnez	s2,14c58 <__multf3+0x7a4>
   14bf4:	00f57793          	andi	a5,a0,15
   14bf8:	00400713          	li	a4,4
   14bfc:	04e78e63          	beq	a5,a4,14c58 <__multf3+0x7a4>
   14c00:	00450793          	addi	a5,a0,4
   14c04:	00a7b533          	sltu	a0,a5,a0
   14c08:	00a40433          	add	s0,s0,a0
   14c0c:	00078513          	mv	a0,a5
   14c10:	0480006f          	j	14c58 <__multf3+0x7a4>
   14c14:	fc07051b          	addiw	a0,a4,-64
   14c18:	04000893          	li	a7,64
   14c1c:	00a45533          	srl	a0,s0,a0
   14c20:	00000593          	li	a1,0
   14c24:	01180863          	beq	a6,a7,14c34 <__multf3+0x780>
   14c28:	08000593          	li	a1,128
   14c2c:	40e5873b          	subw	a4,a1,a4
   14c30:	00e415b3          	sll	a1,s0,a4
   14c34:	00f5e7b3          	or	a5,a1,a5
   14c38:	00f037b3          	snez	a5,a5
   14c3c:	00f56533          	or	a0,a0,a5
   14c40:	00000413          	li	s0,0
   14c44:	f91ff06f          	j	14bd4 <__multf3+0x720>
   14c48:	00061863          	bnez	a2,14c58 <__multf3+0x7a4>
   14c4c:	00850793          	addi	a5,a0,8
   14c50:	fb5ff06f          	j	14c04 <__multf3+0x750>
   14c54:	fe061ce3          	bnez	a2,14c4c <__multf3+0x798>
   14c58:	03345793          	srli	a5,s0,0x33
   14c5c:	0017f793          	andi	a5,a5,1
   14c60:	02078063          	beqz	a5,14c80 <__multf3+0x7cc>
   14c64:	0014e493          	ori	s1,s1,1
   14c68:	00000413          	li	s0,0
   14c6c:	00000513          	li	a0,0
   14c70:	00100713          	li	a4,1
   14c74:	b00682e3          	beqz	a3,14778 <__multf3+0x2c4>
   14c78:	0024e493          	ori	s1,s1,2
   14c7c:	afdff06f          	j	14778 <__multf3+0x2c4>
   14c80:	03d41793          	slli	a5,s0,0x3d
   14c84:	00355513          	srli	a0,a0,0x3
   14c88:	00a7e533          	or	a0,a5,a0
   14c8c:	00345413          	srli	s0,s0,0x3
   14c90:	00000713          	li	a4,0
   14c94:	ae0682e3          	beqz	a3,14778 <__multf3+0x2c4>
   14c98:	0014f793          	andi	a5,s1,1
   14c9c:	ac078ee3          	beqz	a5,14778 <__multf3+0x2c4>
   14ca0:	fd9ff06f          	j	14c78 <__multf3+0x7c4>
   14ca4:	0087e533          	or	a0,a5,s0
   14ca8:	02050463          	beqz	a0,14cd0 <__multf3+0x81c>
   14cac:	00200793          	li	a5,2
   14cb0:	0014e493          	ori	s1,s1,1
   14cb4:	02f90e63          	beq	s2,a5,14cf0 <__multf3+0x83c>
   14cb8:	00300793          	li	a5,3
   14cbc:	02f90263          	beq	s2,a5,14ce0 <__multf3+0x82c>
   14cc0:	00100793          	li	a5,1
   14cc4:	00091463          	bnez	s2,14ccc <__multf3+0x818>
   14cc8:	00500793          	li	a5,5
   14ccc:	0037d513          	srli	a0,a5,0x3
   14cd0:	0024e493          	ori	s1,s1,2
   14cd4:	00000413          	li	s0,0
   14cd8:	00000713          	li	a4,0
   14cdc:	a9dff06f          	j	14778 <__multf3+0x2c4>
   14ce0:	00900793          	li	a5,9
   14ce4:	fe0604e3          	beqz	a2,14ccc <__multf3+0x818>
   14ce8:	00100793          	li	a5,1
   14cec:	fe1ff06f          	j	14ccc <__multf3+0x818>
   14cf0:	00900793          	li	a5,9
   14cf4:	fc061ce3          	bnez	a2,14ccc <__multf3+0x818>
   14cf8:	ff1ff06f          	j	14ce8 <__multf3+0x834>
   14cfc:	00008737          	lui	a4,0x8
   14d00:	00000413          	li	s0,0
   14d04:	00000513          	li	a0,0
   14d08:	fff70713          	addi	a4,a4,-1 # 7fff <exit-0x8121>
   14d0c:	a6dff06f          	j	14778 <__multf3+0x2c4>
   14d10:	00100413          	li	s0,1
   14d14:	00008737          	lui	a4,0x8
   14d18:	02f41413          	slli	s0,s0,0x2f
   14d1c:	00000513          	li	a0,0
   14d20:	fff70713          	addi	a4,a4,-1 # 7fff <exit-0x8121>
   14d24:	00000613          	li	a2,0
   14d28:	a51ff06f          	j	14778 <__multf3+0x2c4>

0000000000014d2c <__subtf3>:
   14d2c:	fd010113          	addi	sp,sp,-48
   14d30:	02113423          	sd	ra,40(sp)
   14d34:	02813023          	sd	s0,32(sp)
   14d38:	00913c23          	sd	s1,24(sp)
   14d3c:	01213823          	sd	s2,16(sp)
   14d40:	01313423          	sd	s3,8(sp)
   14d44:	01413023          	sd	s4,0(sp)
   14d48:	00202473          	frrm	s0
   14d4c:	fff00713          	li	a4,-1
   14d50:	01075713          	srli	a4,a4,0x10
   14d54:	00008837          	lui	a6,0x8
   14d58:	0305d893          	srli	a7,a1,0x30
   14d5c:	fff80813          	addi	a6,a6,-1 # 7fff <exit-0x8121>
   14d60:	0306de93          	srli	t4,a3,0x30
   14d64:	03f5d493          	srli	s1,a1,0x3f
   14d68:	03f6d313          	srli	t1,a3,0x3f
   14d6c:	00e5f5b3          	and	a1,a1,a4
   14d70:	00e6f6b3          	and	a3,a3,a4
   14d74:	00359593          	slli	a1,a1,0x3
   14d78:	03d55793          	srli	a5,a0,0x3d
   14d7c:	03d65713          	srli	a4,a2,0x3d
   14d80:	0108f8b3          	and	a7,a7,a6
   14d84:	010efeb3          	and	t4,t4,a6
   14d88:	00369693          	slli	a3,a3,0x3
   14d8c:	00b7e7b3          	or	a5,a5,a1
   14d90:	0004041b          	sext.w	s0,s0
   14d94:	00088993          	mv	s3,a7
   14d98:	00351513          	slli	a0,a0,0x3
   14d9c:	000e8593          	mv	a1,t4
   14da0:	00d76733          	or	a4,a4,a3
   14da4:	00361613          	slli	a2,a2,0x3
   14da8:	010e9663          	bne	t4,a6,14db4 <__subtf3+0x88>
   14dac:	00c766b3          	or	a3,a4,a2
   14db0:	00069463          	bnez	a3,14db8 <__subtf3+0x8c>
   14db4:	00134313          	xori	t1,t1,1
   14db8:	41d886bb          	subw	a3,a7,t4
   14dbc:	00008e37          	lui	t3,0x8
   14dc0:	0006881b          	sext.w	a6,a3
   14dc4:	fffe0f13          	addi	t5,t3,-1 # 7fff <exit-0x8121>
   14dc8:	4c931663          	bne	t1,s1,15294 <__subtf3+0x568>
   14dcc:	13005c63          	blez	a6,14f04 <__subtf3+0x1d8>
   14dd0:	0a0e9663          	bnez	t4,14e7c <__subtf3+0x150>
   14dd4:	00c765b3          	or	a1,a4,a2
   14dd8:	00059a63          	bnez	a1,14dec <__subtf3+0xc0>
   14ddc:	09e88063          	beq	a7,t5,14e5c <__subtf3+0x130>
   14de0:	00078713          	mv	a4,a5
   14de4:	00088593          	mv	a1,a7
   14de8:	1010006f          	j	156e8 <__subtf3+0x9bc>
   14dec:	fff6881b          	addiw	a6,a3,-1
   14df0:	06081463          	bnez	a6,14e58 <__subtf3+0x12c>
   14df4:	00a60633          	add	a2,a2,a0
   14df8:	00a63533          	sltu	a0,a2,a0
   14dfc:	00f70733          	add	a4,a4,a5
   14e00:	00a70733          	add	a4,a4,a0
   14e04:	00088593          	mv	a1,a7
   14e08:	00060513          	mv	a0,a2
   14e0c:	03375793          	srli	a5,a4,0x33
   14e10:	0017f793          	andi	a5,a5,1
   14e14:	0c078ae3          	beqz	a5,156e8 <__subtf3+0x9bc>
   14e18:	000086b7          	lui	a3,0x8
   14e1c:	00158593          	addi	a1,a1,1
   14e20:	fff68793          	addi	a5,a3,-1 # 7fff <exit-0x8121>
   14e24:	42f58e63          	beq	a1,a5,15260 <__subtf3+0x534>
   14e28:	fff00793          	li	a5,-1
   14e2c:	03379793          	slli	a5,a5,0x33
   14e30:	fff78793          	addi	a5,a5,-1
   14e34:	00f777b3          	and	a5,a4,a5
   14e38:	00155713          	srli	a4,a0,0x1
   14e3c:	00157513          	andi	a0,a0,1
   14e40:	00a76533          	or	a0,a4,a0
   14e44:	03f79713          	slli	a4,a5,0x3f
   14e48:	00a76533          	or	a0,a4,a0
   14e4c:	0017d793          	srli	a5,a5,0x1
   14e50:	00000813          	li	a6,0
   14e54:	3900006f          	j	151e4 <__subtf3+0x4b8>
   14e58:	03e89a63          	bne	a7,t5,14e8c <__subtf3+0x160>
   14e5c:	00a7e733          	or	a4,a5,a0
   14e60:	100704e3          	beqz	a4,15768 <__subtf3+0xa3c>
   14e64:	0327d713          	srli	a4,a5,0x32
   14e68:	00177713          	andi	a4,a4,1
   14e6c:	00088593          	mv	a1,a7
   14e70:	00000813          	li	a6,0
   14e74:	36071863          	bnez	a4,151e4 <__subtf3+0x4b8>
   14e78:	0c00006f          	j	14f38 <__subtf3+0x20c>
   14e7c:	ffe880e3          	beq	a7,t5,14e5c <__subtf3+0x130>
   14e80:	00100693          	li	a3,1
   14e84:	03369693          	slli	a3,a3,0x33
   14e88:	00d76733          	or	a4,a4,a3
   14e8c:	07400693          	li	a3,116
   14e90:	0706c463          	blt	a3,a6,14ef8 <__subtf3+0x1cc>
   14e94:	03f00693          	li	a3,63
   14e98:	0306c663          	blt	a3,a6,14ec4 <__subtf3+0x198>
   14e9c:	04000593          	li	a1,64
   14ea0:	410585bb          	subw	a1,a1,a6
   14ea4:	00b716b3          	sll	a3,a4,a1
   14ea8:	01065333          	srl	t1,a2,a6
   14eac:	00b61633          	sll	a2,a2,a1
   14eb0:	0066e6b3          	or	a3,a3,t1
   14eb4:	00c03633          	snez	a2,a2
   14eb8:	00c6e633          	or	a2,a3,a2
   14ebc:	01075733          	srl	a4,a4,a6
   14ec0:	f35ff06f          	j	14df4 <__subtf3+0xc8>
   14ec4:	fc08069b          	addiw	a3,a6,-64
   14ec8:	04000313          	li	t1,64
   14ecc:	00d756b3          	srl	a3,a4,a3
   14ed0:	00000593          	li	a1,0
   14ed4:	00680863          	beq	a6,t1,14ee4 <__subtf3+0x1b8>
   14ed8:	08000593          	li	a1,128
   14edc:	410585bb          	subw	a1,a1,a6
   14ee0:	00b715b3          	sll	a1,a4,a1
   14ee4:	00c5e633          	or	a2,a1,a2
   14ee8:	00c03633          	snez	a2,a2
   14eec:	00c6e633          	or	a2,a3,a2
   14ef0:	00000713          	li	a4,0
   14ef4:	f01ff06f          	j	14df4 <__subtf3+0xc8>
   14ef8:	00c76633          	or	a2,a4,a2
   14efc:	00c03633          	snez	a2,a2
   14f00:	ff1ff06f          	j	14ef0 <__subtf3+0x1c4>
   14f04:	12080063          	beqz	a6,15024 <__subtf3+0x2f8>
   14f08:	08089063          	bnez	a7,14f88 <__subtf3+0x25c>
   14f0c:	00a7e833          	or	a6,a5,a0
   14f10:	02081863          	bnez	a6,14f40 <__subtf3+0x214>
   14f14:	00060513          	mv	a0,a2
   14f18:	7dee9863          	bne	t4,t5,156e8 <__subtf3+0x9bc>
   14f1c:	00c76533          	or	a0,a4,a2
   14f20:	020508e3          	beqz	a0,15750 <__subtf3+0xa24>
   14f24:	03275793          	srli	a5,a4,0x32
   14f28:	0017f793          	andi	a5,a5,1
   14f2c:	020798e3          	bnez	a5,1575c <__subtf3+0xa30>
   14f30:	00070793          	mv	a5,a4
   14f34:	00060513          	mv	a0,a2
   14f38:	01000693          	li	a3,16
   14f3c:	1440006f          	j	15080 <__subtf3+0x354>
   14f40:	fff6c693          	not	a3,a3
   14f44:	0006869b          	sext.w	a3,a3
   14f48:	00069c63          	bnez	a3,14f60 <__subtf3+0x234>
   14f4c:	00c50533          	add	a0,a0,a2
   14f50:	00e78733          	add	a4,a5,a4
   14f54:	00c53633          	sltu	a2,a0,a2
   14f58:	00c70733          	add	a4,a4,a2
   14f5c:	eb1ff06f          	j	14e0c <__subtf3+0xe0>
   14f60:	03ee9e63          	bne	t4,t5,14f9c <__subtf3+0x270>
   14f64:	00c76533          	or	a0,a4,a2
   14f68:	7e050463          	beqz	a0,15750 <__subtf3+0xa24>
   14f6c:	03275793          	srli	a5,a4,0x32
   14f70:	0017f793          	andi	a5,a5,1
   14f74:	7e079463          	bnez	a5,1575c <__subtf3+0xa30>
   14f78:	00070793          	mv	a5,a4
   14f7c:	00060513          	mv	a0,a2
   14f80:	00000813          	li	a6,0
   14f84:	fb5ff06f          	j	14f38 <__subtf3+0x20c>
   14f88:	fdee8ee3          	beq	t4,t5,14f64 <__subtf3+0x238>
   14f8c:	00100813          	li	a6,1
   14f90:	03381813          	slli	a6,a6,0x33
   14f94:	40d006bb          	negw	a3,a3
   14f98:	0107e7b3          	or	a5,a5,a6
   14f9c:	07400813          	li	a6,116
   14fa0:	06d84c63          	blt	a6,a3,15018 <__subtf3+0x2ec>
   14fa4:	03f00813          	li	a6,63
   14fa8:	02d84e63          	blt	a6,a3,14fe4 <__subtf3+0x2b8>
   14fac:	04000893          	li	a7,64
   14fb0:	40d888bb          	subw	a7,a7,a3
   14fb4:	00d55333          	srl	t1,a0,a3
   14fb8:	01179833          	sll	a6,a5,a7
   14fbc:	01151533          	sll	a0,a0,a7
   14fc0:	00686833          	or	a6,a6,t1
   14fc4:	00a03533          	snez	a0,a0
   14fc8:	00a86533          	or	a0,a6,a0
   14fcc:	00d7d6b3          	srl	a3,a5,a3
   14fd0:	00c50533          	add	a0,a0,a2
   14fd4:	00e686b3          	add	a3,a3,a4
   14fd8:	00c53633          	sltu	a2,a0,a2
   14fdc:	00c68733          	add	a4,a3,a2
   14fe0:	e2dff06f          	j	14e0c <__subtf3+0xe0>
   14fe4:	fc06881b          	addiw	a6,a3,-64
   14fe8:	04000313          	li	t1,64
   14fec:	0107d833          	srl	a6,a5,a6
   14ff0:	00000893          	li	a7,0
   14ff4:	00668863          	beq	a3,t1,15004 <__subtf3+0x2d8>
   14ff8:	08000893          	li	a7,128
   14ffc:	40d886bb          	subw	a3,a7,a3
   15000:	00d798b3          	sll	a7,a5,a3
   15004:	00a8e533          	or	a0,a7,a0
   15008:	00a03533          	snez	a0,a0
   1500c:	00a86533          	or	a0,a6,a0
   15010:	00000693          	li	a3,0
   15014:	fbdff06f          	j	14fd0 <__subtf3+0x2a4>
   15018:	00a7e533          	or	a0,a5,a0
   1501c:	00a03533          	snez	a0,a0
   15020:	ff1ff06f          	j	15010 <__subtf3+0x2e4>
   15024:	00188593          	addi	a1,a7,1
   15028:	ffee0693          	addi	a3,t3,-2
   1502c:	00d5f333          	and	t1,a1,a3
   15030:	18031863          	bnez	t1,151c0 <__subtf3+0x494>
   15034:	00a7e5b3          	or	a1,a5,a0
   15038:	06089263          	bnez	a7,1509c <__subtf3+0x370>
   1503c:	6c058263          	beqz	a1,15700 <__subtf3+0x9d4>
   15040:	00c766b3          	or	a3,a4,a2
   15044:	48068263          	beqz	a3,154c8 <__subtf3+0x79c>
   15048:	00c50633          	add	a2,a0,a2
   1504c:	00e787b3          	add	a5,a5,a4
   15050:	00a63533          	sltu	a0,a2,a0
   15054:	00a787b3          	add	a5,a5,a0
   15058:	0337d713          	srli	a4,a5,0x33
   1505c:	00177713          	andi	a4,a4,1
   15060:	6a070263          	beqz	a4,15704 <__subtf3+0x9d8>
   15064:	fff00713          	li	a4,-1
   15068:	03371713          	slli	a4,a4,0x33
   1506c:	fff70713          	addi	a4,a4,-1
   15070:	00e7f7b3          	and	a5,a5,a4
   15074:	00060513          	mv	a0,a2
   15078:	00000693          	li	a3,0
   1507c:	00100593          	li	a1,1
   15080:	00757713          	andi	a4,a0,7
   15084:	1a071263          	bnez	a4,15228 <__subtf3+0x4fc>
   15088:	08080663          	beqz	a6,15114 <__subtf3+0x3e8>
   1508c:	0016f713          	andi	a4,a3,1
   15090:	08070263          	beqz	a4,15114 <__subtf3+0x3e8>
   15094:	0026e693          	ori	a3,a3,2
   15098:	07c0006f          	j	15114 <__subtf3+0x3e8>
   1509c:	03e89c63          	bne	a7,t5,150d4 <__subtf3+0x3a8>
   150a0:	78058063          	beqz	a1,15820 <__subtf3+0xaf4>
   150a4:	0327d693          	srli	a3,a5,0x32
   150a8:	0016f693          	andi	a3,a3,1
   150ac:	0016b693          	seqz	a3,a3
   150b0:	00469693          	slli	a3,a3,0x4
   150b4:	051e9063          	bne	t4,a7,150f4 <__subtf3+0x3c8>
   150b8:	00c768b3          	or	a7,a4,a2
   150bc:	02088063          	beqz	a7,150dc <__subtf3+0x3b0>
   150c0:	03275893          	srli	a7,a4,0x32
   150c4:	0018f893          	andi	a7,a7,1
   150c8:	00089a63          	bnez	a7,150dc <__subtf3+0x3b0>
   150cc:	01000693          	li	a3,16
   150d0:	00c0006f          	j	150dc <__subtf3+0x3b0>
   150d4:	00000693          	li	a3,0
   150d8:	ffee80e3          	beq	t4,t5,150b8 <__subtf3+0x38c>
   150dc:	00059c63          	bnez	a1,150f4 <__subtf3+0x3c8>
   150e0:	00070793          	mv	a5,a4
   150e4:	00060513          	mv	a0,a2
   150e8:	000085b7          	lui	a1,0x8
   150ec:	fff58593          	addi	a1,a1,-1 # 7fff <exit-0x8121>
   150f0:	f91ff06f          	j	15080 <__subtf3+0x354>
   150f4:	00c76633          	or	a2,a4,a2
   150f8:	fe0608e3          	beqz	a2,150e8 <__subtf3+0x3bc>
   150fc:	00100793          	li	a5,1
   15100:	000085b7          	lui	a1,0x8
   15104:	00000493          	li	s1,0
   15108:	03279793          	slli	a5,a5,0x32
   1510c:	00000513          	li	a0,0
   15110:	fff58593          	addi	a1,a1,-1 # 7fff <exit-0x8121>
   15114:	0337d713          	srli	a4,a5,0x33
   15118:	00177713          	andi	a4,a4,1
   1511c:	02070263          	beqz	a4,15140 <__subtf3+0x414>
   15120:	00008737          	lui	a4,0x8
   15124:	00158593          	addi	a1,a1,1
   15128:	fff70613          	addi	a2,a4,-1 # 7fff <exit-0x8121>
   1512c:	6ac58863          	beq	a1,a2,157dc <__subtf3+0xab0>
   15130:	fff00713          	li	a4,-1
   15134:	03371713          	slli	a4,a4,0x33
   15138:	fff70713          	addi	a4,a4,-1
   1513c:	00e7f7b3          	and	a5,a5,a4
   15140:	00355713          	srli	a4,a0,0x3
   15144:	03d79513          	slli	a0,a5,0x3d
   15148:	00e56533          	or	a0,a0,a4
   1514c:	00008737          	lui	a4,0x8
   15150:	fff70713          	addi	a4,a4,-1 # 7fff <exit-0x8121>
   15154:	0037d793          	srli	a5,a5,0x3
   15158:	02e59063          	bne	a1,a4,15178 <__subtf3+0x44c>
   1515c:	00f56533          	or	a0,a0,a5
   15160:	00000793          	li	a5,0
   15164:	00050a63          	beqz	a0,15178 <__subtf3+0x44c>
   15168:	00100793          	li	a5,1
   1516c:	02f79793          	slli	a5,a5,0x2f
   15170:	00000513          	li	a0,0
   15174:	00000493          	li	s1,0
   15178:	03159593          	slli	a1,a1,0x31
   1517c:	00f4949b          	slliw	s1,s1,0xf
   15180:	0315d593          	srli	a1,a1,0x31
   15184:	0095e5b3          	or	a1,a1,s1
   15188:	01079793          	slli	a5,a5,0x10
   1518c:	03059493          	slli	s1,a1,0x30
   15190:	0107d593          	srli	a1,a5,0x10
   15194:	0095e5b3          	or	a1,a1,s1
   15198:	00068463          	beqz	a3,151a0 <__subtf3+0x474>
   1519c:	0016a073          	csrs	fflags,a3
   151a0:	02813083          	ld	ra,40(sp)
   151a4:	02013403          	ld	s0,32(sp)
   151a8:	01813483          	ld	s1,24(sp)
   151ac:	01013903          	ld	s2,16(sp)
   151b0:	00813983          	ld	s3,8(sp)
   151b4:	00013a03          	ld	s4,0(sp)
   151b8:	03010113          	addi	sp,sp,48
   151bc:	00008067          	ret
   151c0:	03e58663          	beq	a1,t5,151ec <__subtf3+0x4c0>
   151c4:	00c50633          	add	a2,a0,a2
   151c8:	00a63533          	sltu	a0,a2,a0
   151cc:	00e787b3          	add	a5,a5,a4
   151d0:	00a787b3          	add	a5,a5,a0
   151d4:	03f79513          	slli	a0,a5,0x3f
   151d8:	00165613          	srli	a2,a2,0x1
   151dc:	00c56533          	or	a0,a0,a2
   151e0:	0017d793          	srli	a5,a5,0x1
   151e4:	00000693          	li	a3,0
   151e8:	e99ff06f          	j	15080 <__subtf3+0x354>
   151ec:	00040863          	beqz	s0,151fc <__subtf3+0x4d0>
   151f0:	00300793          	li	a5,3
   151f4:	00f41c63          	bne	s0,a5,1520c <__subtf3+0x4e0>
   151f8:	02049063          	bnez	s1,15218 <__subtf3+0x4ec>
   151fc:	00000793          	li	a5,0
   15200:	00000513          	li	a0,0
   15204:	00500693          	li	a3,5
   15208:	f0dff06f          	j	15114 <__subtf3+0x3e8>
   1520c:	00200793          	li	a5,2
   15210:	00f41463          	bne	s0,a5,15218 <__subtf3+0x4ec>
   15214:	fe0494e3          	bnez	s1,151fc <__subtf3+0x4d0>
   15218:	fff00793          	li	a5,-1
   1521c:	fff00513          	li	a0,-1
   15220:	00068593          	mv	a1,a3
   15224:	00500693          	li	a3,5
   15228:	00200713          	li	a4,2
   1522c:	0016e693          	ori	a3,a3,1
   15230:	5ae40063          	beq	s0,a4,157d0 <__subtf3+0xaa4>
   15234:	00300713          	li	a4,3
   15238:	58e40663          	beq	s0,a4,157c4 <__subtf3+0xa98>
   1523c:	58041c63          	bnez	s0,157d4 <__subtf3+0xaa8>
   15240:	00f57713          	andi	a4,a0,15
   15244:	00400613          	li	a2,4
   15248:	58c70663          	beq	a4,a2,157d4 <__subtf3+0xaa8>
   1524c:	00450713          	addi	a4,a0,4
   15250:	00a73533          	sltu	a0,a4,a0
   15254:	00a787b3          	add	a5,a5,a0
   15258:	00070513          	mv	a0,a4
   1525c:	5780006f          	j	157d4 <__subtf3+0xaa8>
   15260:	f8040ee3          	beqz	s0,151fc <__subtf3+0x4d0>
   15264:	00300793          	li	a5,3
   15268:	00f41e63          	bne	s0,a5,15284 <__subtf3+0x558>
   1526c:	f80488e3          	beqz	s1,151fc <__subtf3+0x4d0>
   15270:	fff00793          	li	a5,-1
   15274:	fff00513          	li	a0,-1
   15278:	ffe68593          	addi	a1,a3,-2
   1527c:	00000813          	li	a6,0
   15280:	fa5ff06f          	j	15224 <__subtf3+0x4f8>
   15284:	00200793          	li	a5,2
   15288:	fef414e3          	bne	s0,a5,15270 <__subtf3+0x544>
   1528c:	f60498e3          	bnez	s1,151fc <__subtf3+0x4d0>
   15290:	fe1ff06f          	j	15270 <__subtf3+0x544>
   15294:	0f005063          	blez	a6,15374 <__subtf3+0x648>
   15298:	080e9463          	bnez	t4,15320 <__subtf3+0x5f4>
   1529c:	00c765b3          	or	a1,a4,a2
   152a0:	b2058ee3          	beqz	a1,14ddc <__subtf3+0xb0>
   152a4:	fff6881b          	addiw	a6,a3,-1
   152a8:	02081e63          	bnez	a6,152e4 <__subtf3+0x5b8>
   152ac:	40c50633          	sub	a2,a0,a2
   152b0:	00c53533          	sltu	a0,a0,a2
   152b4:	40e78733          	sub	a4,a5,a4
   152b8:	40a70733          	sub	a4,a4,a0
   152bc:	00088593          	mv	a1,a7
   152c0:	00060513          	mv	a0,a2
   152c4:	03375793          	srli	a5,a4,0x33
   152c8:	0017f793          	andi	a5,a5,1
   152cc:	40078e63          	beqz	a5,156e8 <__subtf3+0x9bc>
   152d0:	00d71713          	slli	a4,a4,0xd
   152d4:	00d75913          	srli	s2,a4,0xd
   152d8:	00050a13          	mv	s4,a0
   152dc:	00058993          	mv	s3,a1
   152e0:	3240006f          	j	15604 <__subtf3+0x8d8>
   152e4:	b7e88ce3          	beq	a7,t5,14e5c <__subtf3+0x130>
   152e8:	07400693          	li	a3,116
   152ec:	0706ce63          	blt	a3,a6,15368 <__subtf3+0x63c>
   152f0:	03f00693          	li	a3,63
   152f4:	0506c063          	blt	a3,a6,15334 <__subtf3+0x608>
   152f8:	04000593          	li	a1,64
   152fc:	410585bb          	subw	a1,a1,a6
   15300:	00b716b3          	sll	a3,a4,a1
   15304:	01065333          	srl	t1,a2,a6
   15308:	00b61633          	sll	a2,a2,a1
   1530c:	0066e6b3          	or	a3,a3,t1
   15310:	00c03633          	snez	a2,a2
   15314:	00c6e633          	or	a2,a3,a2
   15318:	01075733          	srl	a4,a4,a6
   1531c:	f91ff06f          	j	152ac <__subtf3+0x580>
   15320:	b3e88ee3          	beq	a7,t5,14e5c <__subtf3+0x130>
   15324:	00100693          	li	a3,1
   15328:	03369693          	slli	a3,a3,0x33
   1532c:	00d76733          	or	a4,a4,a3
   15330:	fb9ff06f          	j	152e8 <__subtf3+0x5bc>
   15334:	fc08069b          	addiw	a3,a6,-64
   15338:	04000313          	li	t1,64
   1533c:	00d756b3          	srl	a3,a4,a3
   15340:	00000593          	li	a1,0
   15344:	00680863          	beq	a6,t1,15354 <__subtf3+0x628>
   15348:	08000593          	li	a1,128
   1534c:	410585bb          	subw	a1,a1,a6
   15350:	00b715b3          	sll	a1,a4,a1
   15354:	00c5e633          	or	a2,a1,a2
   15358:	00c03633          	snez	a2,a2
   1535c:	00c6e633          	or	a2,a3,a2
   15360:	00000713          	li	a4,0
   15364:	f49ff06f          	j	152ac <__subtf3+0x580>
   15368:	00c76633          	or	a2,a4,a2
   1536c:	00c03633          	snez	a2,a2
   15370:	ff1ff06f          	j	15360 <__subtf3+0x634>
   15374:	12080063          	beqz	a6,15494 <__subtf3+0x768>
   15378:	08089063          	bnez	a7,153f8 <__subtf3+0x6cc>
   1537c:	00a7e833          	or	a6,a5,a0
   15380:	02081663          	bnez	a6,153ac <__subtf3+0x680>
   15384:	37ee9863          	bne	t4,t5,156f4 <__subtf3+0x9c8>
   15388:	00c76533          	or	a0,a4,a2
   1538c:	3e050663          	beqz	a0,15778 <__subtf3+0xa4c>
   15390:	03275793          	srli	a5,a4,0x32
   15394:	0017f793          	andi	a5,a5,1
   15398:	3e079663          	bnez	a5,15784 <__subtf3+0xa58>
   1539c:	00070793          	mv	a5,a4
   153a0:	00060513          	mv	a0,a2
   153a4:	00030493          	mv	s1,t1
   153a8:	b91ff06f          	j	14f38 <__subtf3+0x20c>
   153ac:	fff6c693          	not	a3,a3
   153b0:	0006869b          	sext.w	a3,a3
   153b4:	00069e63          	bnez	a3,153d0 <__subtf3+0x6a4>
   153b8:	40a60533          	sub	a0,a2,a0
   153bc:	40f70733          	sub	a4,a4,a5
   153c0:	00a63633          	sltu	a2,a2,a0
   153c4:	40c70733          	sub	a4,a4,a2
   153c8:	00030493          	mv	s1,t1
   153cc:	ef9ff06f          	j	152c4 <__subtf3+0x598>
   153d0:	03ee9e63          	bne	t4,t5,1540c <__subtf3+0x6e0>
   153d4:	00c76533          	or	a0,a4,a2
   153d8:	3a050063          	beqz	a0,15778 <__subtf3+0xa4c>
   153dc:	03275793          	srli	a5,a4,0x32
   153e0:	0017f793          	andi	a5,a5,1
   153e4:	3a079063          	bnez	a5,15784 <__subtf3+0xa58>
   153e8:	00070793          	mv	a5,a4
   153ec:	00060513          	mv	a0,a2
   153f0:	00030493          	mv	s1,t1
   153f4:	b8dff06f          	j	14f80 <__subtf3+0x254>
   153f8:	fdee8ee3          	beq	t4,t5,153d4 <__subtf3+0x6a8>
   153fc:	00100813          	li	a6,1
   15400:	03381813          	slli	a6,a6,0x33
   15404:	40d006bb          	negw	a3,a3
   15408:	0107e7b3          	or	a5,a5,a6
   1540c:	07400813          	li	a6,116
   15410:	06d84c63          	blt	a6,a3,15488 <__subtf3+0x75c>
   15414:	03f00813          	li	a6,63
   15418:	02d84e63          	blt	a6,a3,15454 <__subtf3+0x728>
   1541c:	04000893          	li	a7,64
   15420:	40d888bb          	subw	a7,a7,a3
   15424:	01179833          	sll	a6,a5,a7
   15428:	00d55e33          	srl	t3,a0,a3
   1542c:	01151533          	sll	a0,a0,a7
   15430:	01c86833          	or	a6,a6,t3
   15434:	00a03533          	snez	a0,a0
   15438:	00a86533          	or	a0,a6,a0
   1543c:	00d7d7b3          	srl	a5,a5,a3
   15440:	40a60533          	sub	a0,a2,a0
   15444:	40f707b3          	sub	a5,a4,a5
   15448:	00a63633          	sltu	a2,a2,a0
   1544c:	40c78733          	sub	a4,a5,a2
   15450:	f79ff06f          	j	153c8 <__subtf3+0x69c>
   15454:	fc06881b          	addiw	a6,a3,-64
   15458:	04000e13          	li	t3,64
   1545c:	0107d833          	srl	a6,a5,a6
   15460:	00000893          	li	a7,0
   15464:	01c68863          	beq	a3,t3,15474 <__subtf3+0x748>
   15468:	08000893          	li	a7,128
   1546c:	40d886bb          	subw	a3,a7,a3
   15470:	00d798b3          	sll	a7,a5,a3
   15474:	00a8e533          	or	a0,a7,a0
   15478:	00a03533          	snez	a0,a0
   1547c:	00a86533          	or	a0,a6,a0
   15480:	00000793          	li	a5,0
   15484:	fbdff06f          	j	15440 <__subtf3+0x714>
   15488:	00a7e533          	or	a0,a5,a0
   1548c:	00a03533          	snez	a0,a0
   15490:	ff1ff06f          	j	15480 <__subtf3+0x754>
   15494:	00188593          	addi	a1,a7,1
   15498:	ffee0693          	addi	a3,t3,-2
   1549c:	00d5f6b3          	and	a3,a1,a3
   154a0:	12069a63          	bnez	a3,155d4 <__subtf3+0x8a8>
   154a4:	00a7ee33          	or	t3,a5,a0
   154a8:	00c765b3          	or	a1,a4,a2
   154ac:	0c089663          	bnez	a7,15578 <__subtf3+0x84c>
   154b0:	060e1c63          	bnez	t3,15528 <__subtf3+0x7fc>
   154b4:	24059c63          	bnez	a1,1570c <__subtf3+0x9e0>
   154b8:	ffe40493          	addi	s1,s0,-2
   154bc:	0014b493          	seqz	s1,s1
   154c0:	00000793          	li	a5,0
   154c4:	00000513          	li	a0,0
   154c8:	00f56733          	or	a4,a0,a5
   154cc:	2e070463          	beqz	a4,157b4 <__subtf3+0xa88>
   154d0:	03f55713          	srli	a4,a0,0x3f
   154d4:	00179813          	slli	a6,a5,0x1
   154d8:	00e80833          	add	a6,a6,a4
   154dc:	00151713          	slli	a4,a0,0x1
   154e0:	00777613          	andi	a2,a4,7
   154e4:	00000693          	li	a3,0
   154e8:	02060663          	beqz	a2,15514 <__subtf3+0x7e8>
   154ec:	00200693          	li	a3,2
   154f0:	24d40a63          	beq	s0,a3,15744 <__subtf3+0xa18>
   154f4:	00300693          	li	a3,3
   154f8:	22d40663          	beq	s0,a3,15724 <__subtf3+0x9f8>
   154fc:	00100693          	li	a3,1
   15500:	00041a63          	bnez	s0,15514 <__subtf3+0x7e8>
   15504:	00f77613          	andi	a2,a4,15
   15508:	00400593          	li	a1,4
   1550c:	ffc73713          	sltiu	a4,a4,-4
   15510:	22b61063          	bne	a2,a1,15730 <__subtf3+0xa04>
   15514:	03485813          	srli	a6,a6,0x34
   15518:	00184813          	xori	a6,a6,1
   1551c:	00187813          	andi	a6,a6,1
   15520:	00000593          	li	a1,0
   15524:	b5dff06f          	j	15080 <__subtf3+0x354>
   15528:	fa0580e3          	beqz	a1,154c8 <__subtf3+0x79c>
   1552c:	40c505b3          	sub	a1,a0,a2
   15530:	00b53833          	sltu	a6,a0,a1
   15534:	40e786b3          	sub	a3,a5,a4
   15538:	410686b3          	sub	a3,a3,a6
   1553c:	0336d813          	srli	a6,a3,0x33
   15540:	00187813          	andi	a6,a6,1
   15544:	00080e63          	beqz	a6,15560 <__subtf3+0x834>
   15548:	40a60533          	sub	a0,a2,a0
   1554c:	40f707b3          	sub	a5,a4,a5
   15550:	00a63633          	sltu	a2,a2,a0
   15554:	40c787b3          	sub	a5,a5,a2
   15558:	00030493          	mv	s1,t1
   1555c:	f6dff06f          	j	154c8 <__subtf3+0x79c>
   15560:	00d5e533          	or	a0,a1,a3
   15564:	1a051a63          	bnez	a0,15718 <__subtf3+0x9ec>
   15568:	ffe40493          	addi	s1,s0,-2
   1556c:	0014b493          	seqz	s1,s1
   15570:	00000793          	li	a5,0
   15574:	f55ff06f          	j	154c8 <__subtf3+0x79c>
   15578:	03e89a63          	bne	a7,t5,155ac <__subtf3+0x880>
   1557c:	280e0c63          	beqz	t3,15814 <__subtf3+0xae8>
   15580:	0327d693          	srli	a3,a5,0x32
   15584:	0016f693          	andi	a3,a3,1
   15588:	0016b693          	seqz	a3,a3
   1558c:	00469693          	slli	a3,a3,0x4
   15590:	031e9e63          	bne	t4,a7,155cc <__subtf3+0x8a0>
   15594:	02058063          	beqz	a1,155b4 <__subtf3+0x888>
   15598:	03275893          	srli	a7,a4,0x32
   1559c:	0018f893          	andi	a7,a7,1
   155a0:	00089a63          	bnez	a7,155b4 <__subtf3+0x888>
   155a4:	01000693          	li	a3,16
   155a8:	00c0006f          	j	155b4 <__subtf3+0x888>
   155ac:	00000693          	li	a3,0
   155b0:	ffee82e3          	beq	t4,t5,15594 <__subtf3+0x868>
   155b4:	000e1c63          	bnez	t3,155cc <__subtf3+0x8a0>
   155b8:	1c058e63          	beqz	a1,15794 <__subtf3+0xa68>
   155bc:	00070793          	mv	a5,a4
   155c0:	00060513          	mv	a0,a2
   155c4:	00030493          	mv	s1,t1
   155c8:	b21ff06f          	j	150e8 <__subtf3+0x3bc>
   155cc:	b0058ee3          	beqz	a1,150e8 <__subtf3+0x3bc>
   155d0:	b2dff06f          	j	150fc <__subtf3+0x3d0>
   155d4:	40c50a33          	sub	s4,a0,a2
   155d8:	014536b3          	sltu	a3,a0,s4
   155dc:	40e78933          	sub	s2,a5,a4
   155e0:	40d90933          	sub	s2,s2,a3
   155e4:	03395693          	srli	a3,s2,0x33
   155e8:	0016f693          	andi	a3,a3,1
   155ec:	08068663          	beqz	a3,15678 <__subtf3+0x94c>
   155f0:	40a60a33          	sub	s4,a2,a0
   155f4:	40f70733          	sub	a4,a4,a5
   155f8:	01463633          	sltu	a2,a2,s4
   155fc:	40c70933          	sub	s2,a4,a2
   15600:	00030493          	mv	s1,t1
   15604:	08090063          	beqz	s2,15684 <__subtf3+0x958>
   15608:	00090513          	mv	a0,s2
   1560c:	5b0000ef          	jal	ra,15bbc <__clzdi2>
   15610:	0005051b          	sext.w	a0,a0
   15614:	ff45059b          	addiw	a1,a0,-12
   15618:	03f00793          	li	a5,63
   1561c:	00058693          	mv	a3,a1
   15620:	06b7ca63          	blt	a5,a1,15694 <__subtf3+0x968>
   15624:	04000713          	li	a4,64
   15628:	40b7073b          	subw	a4,a4,a1
   1562c:	00b91933          	sll	s2,s2,a1
   15630:	00ea5733          	srl	a4,s4,a4
   15634:	01276733          	or	a4,a4,s2
   15638:	00ba1533          	sll	a0,s4,a1
   1563c:	0935cc63          	blt	a1,s3,156d4 <__subtf3+0x9a8>
   15640:	413686bb          	subw	a3,a3,s3
   15644:	0016879b          	addiw	a5,a3,1
   15648:	03f00613          	li	a2,63
   1564c:	04f64c63          	blt	a2,a5,156a4 <__subtf3+0x978>
   15650:	04000693          	li	a3,64
   15654:	40f686bb          	subw	a3,a3,a5
   15658:	00f555b3          	srl	a1,a0,a5
   1565c:	00d71633          	sll	a2,a4,a3
   15660:	00d51533          	sll	a0,a0,a3
   15664:	00b66633          	or	a2,a2,a1
   15668:	00a03533          	snez	a0,a0
   1566c:	00a66533          	or	a0,a2,a0
   15670:	00f757b3          	srl	a5,a4,a5
   15674:	e55ff06f          	j	154c8 <__subtf3+0x79c>
   15678:	012a6533          	or	a0,s4,s2
   1567c:	f80514e3          	bnez	a0,15604 <__subtf3+0x8d8>
   15680:	ee9ff06f          	j	15568 <__subtf3+0x83c>
   15684:	000a0513          	mv	a0,s4
   15688:	534000ef          	jal	ra,15bbc <__clzdi2>
   1568c:	0405051b          	addiw	a0,a0,64
   15690:	f85ff06f          	j	15614 <__subtf3+0x8e8>
   15694:	fb45071b          	addiw	a4,a0,-76
   15698:	00ea1733          	sll	a4,s4,a4
   1569c:	00000513          	li	a0,0
   156a0:	f9dff06f          	j	1563c <__subtf3+0x910>
   156a4:	fc16869b          	addiw	a3,a3,-63
   156a8:	04000813          	li	a6,64
   156ac:	00d75633          	srl	a2,a4,a3
   156b0:	00000693          	li	a3,0
   156b4:	01078863          	beq	a5,a6,156c4 <__subtf3+0x998>
   156b8:	08000693          	li	a3,128
   156bc:	40f686bb          	subw	a3,a3,a5
   156c0:	00d716b3          	sll	a3,a4,a3
   156c4:	00d56533          	or	a0,a0,a3
   156c8:	00a03533          	snez	a0,a0
   156cc:	00a66533          	or	a0,a2,a0
   156d0:	ea1ff06f          	j	15570 <__subtf3+0x844>
   156d4:	fff00793          	li	a5,-1
   156d8:	03379793          	slli	a5,a5,0x33
   156dc:	fff78793          	addi	a5,a5,-1
   156e0:	40b985b3          	sub	a1,s3,a1
   156e4:	00f77733          	and	a4,a4,a5
   156e8:	00070793          	mv	a5,a4
   156ec:	dc058ee3          	beqz	a1,154c8 <__subtf3+0x79c>
   156f0:	f60ff06f          	j	14e50 <__subtf3+0x124>
   156f4:	00060513          	mv	a0,a2
   156f8:	00030493          	mv	s1,t1
   156fc:	fedff06f          	j	156e8 <__subtf3+0x9bc>
   15700:	00070793          	mv	a5,a4
   15704:	00060513          	mv	a0,a2
   15708:	dc1ff06f          	j	154c8 <__subtf3+0x79c>
   1570c:	00070793          	mv	a5,a4
   15710:	00060513          	mv	a0,a2
   15714:	e45ff06f          	j	15558 <__subtf3+0x82c>
   15718:	00068793          	mv	a5,a3
   1571c:	00058513          	mv	a0,a1
   15720:	da9ff06f          	j	154c8 <__subtf3+0x79c>
   15724:	00100693          	li	a3,1
   15728:	de0496e3          	bnez	s1,15514 <__subtf3+0x7e8>
   1572c:	ff873713          	sltiu	a4,a4,-8
   15730:	00174713          	xori	a4,a4,1
   15734:	02071713          	slli	a4,a4,0x20
   15738:	02075713          	srli	a4,a4,0x20
   1573c:	00e80833          	add	a6,a6,a4
   15740:	dd5ff06f          	j	15514 <__subtf3+0x7e8>
   15744:	00100693          	li	a3,1
   15748:	dc0486e3          	beqz	s1,15514 <__subtf3+0x7e8>
   1574c:	fe1ff06f          	j	1572c <__subtf3+0xa00>
   15750:	00000793          	li	a5,0
   15754:	00000693          	li	a3,0
   15758:	9bdff06f          	j	15114 <__subtf3+0x3e8>
   1575c:	00070793          	mv	a5,a4
   15760:	00060513          	mv	a0,a2
   15764:	eecff06f          	j	14e50 <__subtf3+0x124>
   15768:	00000793          	li	a5,0
   1576c:	00000513          	li	a0,0
   15770:	00088593          	mv	a1,a7
   15774:	fe1ff06f          	j	15754 <__subtf3+0xa28>
   15778:	00000793          	li	a5,0
   1577c:	00030493          	mv	s1,t1
   15780:	fd5ff06f          	j	15754 <__subtf3+0xa28>
   15784:	00070793          	mv	a5,a4
   15788:	00060513          	mv	a0,a2
   1578c:	00030493          	mv	s1,t1
   15790:	ec0ff06f          	j	14e50 <__subtf3+0x124>
   15794:	00100793          	li	a5,1
   15798:	000085b7          	lui	a1,0x8
   1579c:	00000513          	li	a0,0
   157a0:	00000493          	li	s1,0
   157a4:	03279793          	slli	a5,a5,0x32
   157a8:	fff58593          	addi	a1,a1,-1 # 7fff <exit-0x8121>
   157ac:	01000693          	li	a3,16
   157b0:	965ff06f          	j	15114 <__subtf3+0x3e8>
   157b4:	00000793          	li	a5,0
   157b8:	00000513          	li	a0,0
   157bc:	00000593          	li	a1,0
   157c0:	f95ff06f          	j	15754 <__subtf3+0xa28>
   157c4:	00049863          	bnez	s1,157d4 <__subtf3+0xaa8>
   157c8:	00850713          	addi	a4,a0,8
   157cc:	a85ff06f          	j	15250 <__subtf3+0x524>
   157d0:	fe049ce3          	bnez	s1,157c8 <__subtf3+0xa9c>
   157d4:	8c0810e3          	bnez	a6,15094 <__subtf3+0x368>
   157d8:	93dff06f          	j	15114 <__subtf3+0x3e8>
   157dc:	00000513          	li	a0,0
   157e0:	02040463          	beqz	s0,15808 <__subtf3+0xadc>
   157e4:	00300793          	li	a5,3
   157e8:	00f41a63          	bne	s0,a5,157fc <__subtf3+0xad0>
   157ec:	00048e63          	beqz	s1,15808 <__subtf3+0xadc>
   157f0:	fff00513          	li	a0,-1
   157f4:	ffe70593          	addi	a1,a4,-2
   157f8:	0100006f          	j	15808 <__subtf3+0xadc>
   157fc:	00200793          	li	a5,2
   15800:	fef418e3          	bne	s0,a5,157f0 <__subtf3+0xac4>
   15804:	fe0486e3          	beqz	s1,157f0 <__subtf3+0xac4>
   15808:	0056e693          	ori	a3,a3,5
   1580c:	00050793          	mv	a5,a0
   15810:	931ff06f          	j	15140 <__subtf3+0x414>
   15814:	00000693          	li	a3,0
   15818:	db1e90e3          	bne	t4,a7,155b8 <__subtf3+0x88c>
   1581c:	d79ff06f          	j	15594 <__subtf3+0x868>
   15820:	00000693          	li	a3,0
   15824:	8b1e9ee3          	bne	t4,a7,150e0 <__subtf3+0x3b4>
   15828:	891ff06f          	j	150b8 <__subtf3+0x38c>

000000000001582c <__fixtfsi>:
   1582c:	00050613          	mv	a2,a0
   15830:	002027f3          	frrm	a5
   15834:	00004837          	lui	a6,0x4
   15838:	00159693          	slli	a3,a1,0x1
   1583c:	01059793          	slli	a5,a1,0x10
   15840:	0316d693          	srli	a3,a3,0x31
   15844:	ffe80713          	addi	a4,a6,-2 # 3ffe <exit-0xc122>
   15848:	0107d793          	srli	a5,a5,0x10
   1584c:	03f5d593          	srli	a1,a1,0x3f
   15850:	00d74c63          	blt	a4,a3,15868 <__fixtfsi+0x3c>
   15854:	08069c63          	bnez	a3,158ec <__fixtfsi+0xc0>
   15858:	00f567b3          	or	a5,a0,a5
   1585c:	00000513          	li	a0,0
   15860:	04079063          	bnez	a5,158a0 <__fixtfsi+0x74>
   15864:	00008067          	ret
   15868:	01d80713          	addi	a4,a6,29
   1586c:	02d75e63          	bge	a4,a3,158a8 <__fixtfsi+0x7c>
   15870:	80000737          	lui	a4,0x80000
   15874:	fff74713          	not	a4,a4
   15878:	00b7053b          	addw	a0,a4,a1
   1587c:	06058c63          	beqz	a1,158f4 <__fixtfsi+0xc8>
   15880:	01e80813          	addi	a6,a6,30
   15884:	01000713          	li	a4,16
   15888:	07069863          	bne	a3,a6,158f8 <__fixtfsi+0xcc>
   1588c:	0117d693          	srli	a3,a5,0x11
   15890:	06069463          	bnez	a3,158f8 <__fixtfsi+0xcc>
   15894:	02f79793          	slli	a5,a5,0x2f
   15898:	00c7e7b3          	or	a5,a5,a2
   1589c:	04078663          	beqz	a5,158e8 <__fixtfsi+0xbc>
   158a0:	00100713          	li	a4,1
   158a4:	0540006f          	j	158f8 <__fixtfsi+0xcc>
   158a8:	00100513          	li	a0,1
   158ac:	03051513          	slli	a0,a0,0x30
   158b0:	00a7e7b3          	or	a5,a5,a0
   158b4:	ffffc737          	lui	a4,0xffffc
   158b8:	00004537          	lui	a0,0x4
   158bc:	02f5051b          	addiw	a0,a0,47
   158c0:	0117071b          	addiw	a4,a4,17
   158c4:	00d7073b          	addw	a4,a4,a3
   158c8:	40d506bb          	subw	a3,a0,a3
   158cc:	00e79733          	sll	a4,a5,a4
   158d0:	00d7d7b3          	srl	a5,a5,a3
   158d4:	00c76733          	or	a4,a4,a2
   158d8:	0007851b          	sext.w	a0,a5
   158dc:	00058463          	beqz	a1,158e4 <__fixtfsi+0xb8>
   158e0:	40a0053b          	negw	a0,a0
   158e4:	fa071ee3          	bnez	a4,158a0 <__fixtfsi+0x74>
   158e8:	00008067          	ret
   158ec:	00000513          	li	a0,0
   158f0:	fb1ff06f          	j	158a0 <__fixtfsi+0x74>
   158f4:	01000713          	li	a4,16
   158f8:	00172073          	csrs	fflags,a4
   158fc:	fedff06f          	j	158e8 <__fixtfsi+0xbc>

0000000000015900 <__fixunstfsi>:
   15900:	00050693          	mv	a3,a0
   15904:	002027f3          	frrm	a5
   15908:	00004637          	lui	a2,0x4
   1590c:	00159713          	slli	a4,a1,0x1
   15910:	01059793          	slli	a5,a1,0x10
   15914:	03175713          	srli	a4,a4,0x31
   15918:	ffe60513          	addi	a0,a2,-2 # 3ffe <exit-0xc122>
   1591c:	0107d793          	srli	a5,a5,0x10
   15920:	03f5d593          	srli	a1,a1,0x3f
   15924:	00e54c63          	blt	a0,a4,1593c <__fixunstfsi+0x3c>
   15928:	06071c63          	bnez	a4,159a0 <__fixunstfsi+0xa0>
   1592c:	00f6e7b3          	or	a5,a3,a5
   15930:	06079863          	bnez	a5,159a0 <__fixunstfsi+0xa0>
   15934:	00000513          	li	a0,0
   15938:	00008067          	ret
   1593c:	01e60513          	addi	a0,a2,30
   15940:	00059463          	bnez	a1,15948 <__fixunstfsi+0x48>
   15944:	01f60513          	addi	a0,a2,31
   15948:	04a75463          	bge	a4,a0,15990 <__fixunstfsi+0x90>
   1594c:	06059063          	bnez	a1,159ac <__fixunstfsi+0xac>
   15950:	00100613          	li	a2,1
   15954:	03061613          	slli	a2,a2,0x30
   15958:	00c7e7b3          	or	a5,a5,a2
   1595c:	000045b7          	lui	a1,0x4
   15960:	ffffc637          	lui	a2,0xffffc
   15964:	02f5859b          	addiw	a1,a1,47
   15968:	0116061b          	addiw	a2,a2,17
   1596c:	40e585bb          	subw	a1,a1,a4
   15970:	00e6073b          	addw	a4,a2,a4
   15974:	00b7d5b3          	srl	a1,a5,a1
   15978:	00e797b3          	sll	a5,a5,a4
   1597c:	00d7e7b3          	or	a5,a5,a3
   15980:	0005851b          	sext.w	a0,a1
   15984:	00078c63          	beqz	a5,1599c <__fixunstfsi+0x9c>
   15988:	00100793          	li	a5,1
   1598c:	00c0006f          	j	15998 <__fixunstfsi+0x98>
   15990:	fff58513          	addi	a0,a1,-1 # 3fff <exit-0xc121>
   15994:	01000793          	li	a5,16
   15998:	0017a073          	csrs	fflags,a5
   1599c:	00008067          	ret
   159a0:	00100793          	li	a5,1
   159a4:	00000513          	li	a0,0
   159a8:	ff1ff06f          	j	15998 <__fixunstfsi+0x98>
   159ac:	01000793          	li	a5,16
   159b0:	ff5ff06f          	j	159a4 <__fixunstfsi+0xa4>

00000000000159b4 <__floatsitf>:
   159b4:	fe010113          	addi	sp,sp,-32
   159b8:	00113c23          	sd	ra,24(sp)
   159bc:	00813823          	sd	s0,16(sp)
   159c0:	00913423          	sd	s1,8(sp)
   159c4:	06050463          	beqz	a0,15a2c <__floatsitf+0x78>
   159c8:	0005079b          	sext.w	a5,a0
   159cc:	03f55493          	srli	s1,a0,0x3f
   159d0:	00055463          	bgez	a0,159d8 <__floatsitf+0x24>
   159d4:	40f007bb          	negw	a5,a5
   159d8:	02079413          	slli	s0,a5,0x20
   159dc:	02045413          	srli	s0,s0,0x20
   159e0:	00040513          	mv	a0,s0
   159e4:	1d8000ef          	jal	ra,15bbc <__clzdi2>
   159e8:	000045b7          	lui	a1,0x4
   159ec:	03e5859b          	addiw	a1,a1,62
   159f0:	ff15079b          	addiw	a5,a0,-15
   159f4:	40a585bb          	subw	a1,a1,a0
   159f8:	00f417b3          	sll	a5,s0,a5
   159fc:	01813083          	ld	ra,24(sp)
   15a00:	01013403          	ld	s0,16(sp)
   15a04:	00f4949b          	slliw	s1,s1,0xf
   15a08:	0095e5b3          	or	a1,a1,s1
   15a0c:	01079793          	slli	a5,a5,0x10
   15a10:	03059593          	slli	a1,a1,0x30
   15a14:	0107d793          	srli	a5,a5,0x10
   15a18:	00813483          	ld	s1,8(sp)
   15a1c:	00000513          	li	a0,0
   15a20:	00b7e5b3          	or	a1,a5,a1
   15a24:	02010113          	addi	sp,sp,32
   15a28:	00008067          	ret
   15a2c:	00000793          	li	a5,0
   15a30:	00000593          	li	a1,0
   15a34:	00000493          	li	s1,0
   15a38:	fc5ff06f          	j	159fc <__floatsitf+0x48>

0000000000015a3c <__floatunsitf>:
   15a3c:	ff010113          	addi	sp,sp,-16
   15a40:	00113423          	sd	ra,8(sp)
   15a44:	00813023          	sd	s0,0(sp)
   15a48:	04050663          	beqz	a0,15a94 <__floatunsitf+0x58>
   15a4c:	02051413          	slli	s0,a0,0x20
   15a50:	02045413          	srli	s0,s0,0x20
   15a54:	00040513          	mv	a0,s0
   15a58:	164000ef          	jal	ra,15bbc <__clzdi2>
   15a5c:	000045b7          	lui	a1,0x4
   15a60:	03e5859b          	addiw	a1,a1,62
   15a64:	40a585bb          	subw	a1,a1,a0
   15a68:	ff15051b          	addiw	a0,a0,-15
   15a6c:	00a41433          	sll	s0,s0,a0
   15a70:	01041413          	slli	s0,s0,0x10
   15a74:	01045413          	srli	s0,s0,0x10
   15a78:	03059593          	slli	a1,a1,0x30
   15a7c:	00813083          	ld	ra,8(sp)
   15a80:	00b465b3          	or	a1,s0,a1
   15a84:	00013403          	ld	s0,0(sp)
   15a88:	00000513          	li	a0,0
   15a8c:	01010113          	addi	sp,sp,16
   15a90:	00008067          	ret
   15a94:	00000413          	li	s0,0
   15a98:	00000593          	li	a1,0
   15a9c:	fd5ff06f          	j	15a70 <__floatunsitf+0x34>

0000000000015aa0 <__extenddftf2>:
   15aa0:	fe010113          	addi	sp,sp,-32
   15aa4:	00113c23          	sd	ra,24(sp)
   15aa8:	00813823          	sd	s0,16(sp)
   15aac:	00913423          	sd	s1,8(sp)
   15ab0:	002027f3          	frrm	a5
   15ab4:	03455793          	srli	a5,a0,0x34
   15ab8:	7ff7f793          	andi	a5,a5,2047
   15abc:	00178713          	addi	a4,a5,1
   15ac0:	00c51413          	slli	s0,a0,0xc
   15ac4:	7fe77713          	andi	a4,a4,2046
   15ac8:	00c45413          	srli	s0,s0,0xc
   15acc:	03f55493          	srli	s1,a0,0x3f
   15ad0:	02070063          	beqz	a4,15af0 <__extenddftf2+0x50>
   15ad4:	00004737          	lui	a4,0x4
   15ad8:	c0070713          	addi	a4,a4,-1024 # 3c00 <exit-0xc520>
   15adc:	00445593          	srli	a1,s0,0x4
   15ae0:	00e787b3          	add	a5,a5,a4
   15ae4:	03c41413          	slli	s0,s0,0x3c
   15ae8:	00000713          	li	a4,0
   15aec:	0880006f          	j	15b74 <__extenddftf2+0xd4>
   15af0:	04079a63          	bnez	a5,15b44 <__extenddftf2+0xa4>
   15af4:	00000593          	li	a1,0
   15af8:	fe0408e3          	beqz	s0,15ae8 <__extenddftf2+0x48>
   15afc:	00040513          	mv	a0,s0
   15b00:	0bc000ef          	jal	ra,15bbc <__clzdi2>
   15b04:	0005071b          	sext.w	a4,a0
   15b08:	00e00793          	li	a5,14
   15b0c:	02e7c463          	blt	a5,a4,15b34 <__extenddftf2+0x94>
   15b10:	00f00593          	li	a1,15
   15b14:	40a585bb          	subw	a1,a1,a0
   15b18:	0315079b          	addiw	a5,a0,49
   15b1c:	00b455b3          	srl	a1,s0,a1
   15b20:	00f41433          	sll	s0,s0,a5
   15b24:	00004737          	lui	a4,0x4
   15b28:	c0c7071b          	addiw	a4,a4,-1012
   15b2c:	40a707bb          	subw	a5,a4,a0
   15b30:	fb9ff06f          	j	15ae8 <__extenddftf2+0x48>
   15b34:	ff15059b          	addiw	a1,a0,-15
   15b38:	00b415b3          	sll	a1,s0,a1
   15b3c:	00000413          	li	s0,0
   15b40:	fe5ff06f          	j	15b24 <__extenddftf2+0x84>
   15b44:	06040463          	beqz	s0,15bac <__extenddftf2+0x10c>
   15b48:	00100793          	li	a5,1
   15b4c:	03379713          	slli	a4,a5,0x33
   15b50:	00e47733          	and	a4,s0,a4
   15b54:	00445593          	srli	a1,s0,0x4
   15b58:	02f79793          	slli	a5,a5,0x2f
   15b5c:	00173713          	seqz	a4,a4
   15b60:	00f5e5b3          	or	a1,a1,a5
   15b64:	000087b7          	lui	a5,0x8
   15b68:	00471713          	slli	a4,a4,0x4
   15b6c:	03c41413          	slli	s0,s0,0x3c
   15b70:	fff78793          	addi	a5,a5,-1 # 7fff <exit-0x8121>
   15b74:	00f4951b          	slliw	a0,s1,0xf
   15b78:	01059593          	slli	a1,a1,0x10
   15b7c:	00a7e7b3          	or	a5,a5,a0
   15b80:	03079793          	slli	a5,a5,0x30
   15b84:	0105d593          	srli	a1,a1,0x10
   15b88:	00f5e5b3          	or	a1,a1,a5
   15b8c:	00070463          	beqz	a4,15b94 <__extenddftf2+0xf4>
   15b90:	00172073          	csrs	fflags,a4
   15b94:	01813083          	ld	ra,24(sp)
   15b98:	00040513          	mv	a0,s0
   15b9c:	01013403          	ld	s0,16(sp)
   15ba0:	00813483          	ld	s1,8(sp)
   15ba4:	02010113          	addi	sp,sp,32
   15ba8:	00008067          	ret
   15bac:	000087b7          	lui	a5,0x8
   15bb0:	00000593          	li	a1,0
   15bb4:	fff78793          	addi	a5,a5,-1 # 7fff <exit-0x8121>
   15bb8:	f31ff06f          	j	15ae8 <__extenddftf2+0x48>

0000000000015bbc <__clzdi2>:
   15bbc:	03800793          	li	a5,56
   15bc0:	00f55733          	srl	a4,a0,a5
   15bc4:	0ff77713          	zext.b	a4,a4
   15bc8:	00071663          	bnez	a4,15bd4 <__clzdi2+0x18>
   15bcc:	ff878793          	addi	a5,a5,-8
   15bd0:	fe0798e3          	bnez	a5,15bc0 <__clzdi2+0x4>
   15bd4:	04000713          	li	a4,64
   15bd8:	40f70733          	sub	a4,a4,a5
   15bdc:	00f55533          	srl	a0,a0,a5
   15be0:	00002797          	auipc	a5,0x2
   15be4:	5207b783          	ld	a5,1312(a5) # 18100 <_GLOBAL_OFFSET_TABLE_+0x8>
   15be8:	00a78533          	add	a0,a5,a0
   15bec:	00054503          	lbu	a0,0(a0) # 4000 <exit-0xc120>
   15bf0:	40a7053b          	subw	a0,a4,a0
   15bf4:	00008067          	ret
