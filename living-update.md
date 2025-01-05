| Phase             | Scheduler | VFS | Block |
|-------------------|-----------|-----|-------|
| Load new domain   |    8213 / 2843     |  1133    |   321    |
| Sync              |     26 / 3291    |  19   |       |
| State Transfer    |    750ns      |   57   |    3   |
| Replace           |     9us       |    10  |     750ns |
| Resource cleanup  |     8us       |   176    |   7  |
| SRCU |   x  |x |69|

```
[2] Domain vfs2 already registered
[2][Domain:24] try to update domain: vfs2, ty: VfsDomain
[2] [Load new domain] Time elapsed: 1039 us
[2] [Task Sync] Time elapsed: 23 us
[2] [Reinit and state transfer] Time elapsed: 72 us
[2] [Domain swap] Time elapsed: 13 us
[2] [Recycle resources] Time elapsed: 8 us
[2] Try to replace vfs domain vfs with vfs2 ok

0ms: 5484544 bytes
100ms: 5496168 bytes
200ms: 5471592 bytes
300ms: 5463400 bytes
400ms: 5479784 bytes
500ms: 5475688 bytes
600ms: 5475688 bytes
700ms: 5467496 bytes
800ms: 5475688 bytes
900ms: 5475688 bytes
1000ms: 5471592 bytes
1100ms: 5463400 bytes
1200ms: 5467496 bytes
1300ms: 5447016 bytes
1400ms: 5475688 bytes
1500ms: 5520744 bytes
1600ms: 5508456 bytes
1700ms: 5492072 bytes
1800ms: 5496168 bytes
1900ms: 5492072 bytes
2000ms: 5483880 bytes
2100ms: 5496168 bytes
2200ms: 5496168 bytes
2300ms: 5492072 bytes
2400ms: 5475688 bytes
2500ms: 5442920 bytes
2600ms: 5471592 bytes
2700ms: 5475688 bytes
2800ms: 5487976 bytes
2900ms: 5483880 bytes
3000ms: 5475688 bytes
3100ms: 5475688 bytes
3200ms: 5479784 bytes
3300ms: 5483880 bytes
3400ms: 5483880 bytes
3500ms: 5487976 bytes
3600ms: 5487976 bytes
3700ms: 5504360 bytes
3800ms: 5508456 bytes
3900ms: 5508456 bytes
4000ms: 5504360 bytes
4100ms: 5504360 bytes
4200ms: 5496168 bytes
4300ms: 5492072 bytes
4400ms: 5492072 bytes
4500ms: 5504360 bytes
4600ms: 5496168 bytes
4700ms: 5504360 bytes
4800ms: 5508456 bytes
4900ms: 5463400 bytes
5000ms: 5377384 bytes
5100ms: 5537128 bytes
5200ms: 5541224 bytes
5300ms: 5537128 bytes
5400ms: 5545320 bytes
5500ms: 5487976 bytes
5600ms: 5467496 bytes
5700ms: 5471592 bytes
5800ms: 5487976 bytes
5900ms: 5492072 bytes
6000ms: 5475688 bytes
6100ms: 5467496 bytes
6200ms: 5475688 bytes
6300ms: 5471592 bytes
6400ms: 5471592 bytes
6500ms: 5471592 bytes
6600ms: 5521408 bytes
6700ms: 5504360 bytes
6800ms: 5492072 bytes
6900ms: 5487976 bytes
7000ms: 5487976 bytes
7100ms: 5487976 bytes
7200ms: 5492072 bytes
7300ms: 5500264 bytes
7400ms: 5500264 bytes
7500ms: 5557608 bytes
7600ms: 5553512 bytes
7700ms: 5492072 bytes
7800ms: 5467496 bytes
7900ms: 5471592 bytes
8000ms: 5471592 bytes
8100ms: 5471592 bytes
8200ms: 5471592 bytes
8300ms: 5471592 bytes
8400ms: 5471592 bytes
8500ms: 5471592 bytes
8600ms: 5545320 bytes
8700ms: 5582184 bytes
8800ms: 5582184 bytes
8900ms: 5508456 bytes
9000ms: 5479784 bytes
9100ms: 5487976 bytes
9200ms: 5487976 bytes
9300ms: 5487976 bytes
9400ms: 5492072 bytes
9500ms: 5492072 bytes
9600ms: 5487976 bytes
9700ms: 5492072 bytes
9800ms: 5496168 bytes

```
