MUSL_PREFIX = riscv64-linux-musl
MUSL_GCC = $(MUSL_PREFIX)-gcc
MUSL_STRIP = $(MUSL_PREFIX)-strip

CC = "$(MUSL_GCC) -U__linux__ -D__BYTE_ORDER=__LITTLE_ENDIAN -static"

redis:
	cd ./redis-3.2.9 && make CC=$(CC) MALLOC=libc persist-settings # 编译依赖库
	cd ./redis-3.2.9 && make CC=$(CC) MALLOC=libc # 编译redis
	cd ./redis-3.2.9/src && cp redis-cli redis-server redis-benchmark ../../../sdcard
	cd ./redis-3.2.9 &&  cp redis.conf ../../sdcard

clean:
	cd ./redis-3.2.9 && make clean