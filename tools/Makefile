MUSL_PREFIX = riscv64-linux-musl
MUSL_GCC = $(MUSL_PREFIX)-gcc
MUSL_STRIP = $(MUSL_PREFIX)-strip
CC = "$(MUSL_GCC) -U__linux__ -D__BYTE_ORDER=__LITTLE_ENDIAN -static"
ARCH = riscv64
sqlite3_pkg := sqlite-amalgamation-3410100
sqlite3_dir := ./$(sqlite3_pkg)
SQLITE3_CFLAGS := -DSQLITE_THREADSAFE=0 -DSQLITE_OMIT_LOAD_EXTENSION -static -static -no-pie -fno-builtin -ffreestanding

ifeq ($(ARCH), riscv64)
  LDFLAGS += --no-relax
endif

redis:
	cd ./redis-3.2.9 && make CC=$(CC) MALLOC=libc persist-settings # 编译依赖库
	cd ./redis-3.2.9 && make CC=$(CC) MALLOC=libc # 编译redis
	cd ./redis-3.2.9/src && cp redis-cli redis-server redis-benchmark ../../../sdcard
	cd ./redis-3.2.9 &&  cp redis.conf ../../sdcard


# Download sqlite source code
$(sqlite3_dir)/sqlite3.c:
	echo "Download sqlite source code"
	wget https://sqlite.org/2023/$(sqlite3_pkg).zip -P ./
	unzip ./$(sqlite3_pkg).zip -d ./ && rm -f ./$(sqlite3_pkg).zip

sqlite3:$(sqlite3_dir)/sqlite3.c
	cd $(sqlite3_dir) && $(MUSL_GCC) $(SQLITE3_CFLAGS) sqlite3.c shell.c -o sqlite3
	cp $(sqlite3_dir)/sqlite3 ./sdcard

app:redis sqlite3
	make clean

clean:
	cd ./redis-3.2.9 && make clean
	cd ./$(sqlite3_pkg) && rm sqlite3