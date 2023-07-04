

// // pthread基本数据结构
// pub struct pthread_t(usize);


// // pub struct pthread_attr{
// //     flags: u64,
// //     stacksize: usize,
// //     stackaddr: *u8,
// //     *status: ;
// //     policy: isize;
// //     __guardsize: usize;
// // };


// // // 比较两个线程ID
// // pub fn pthread_equal(t1: pthread_t , t2: pthread) -> isize {
// //     if t1.0 == t2.0 {
// //         0
// //     } else if t1.0 < t2.0 {
// //         -1
// //     } else {
// //         1
// //     }
// // }

// // // 获取当前线程 ID
// // pub fn pthread_self() -> pthread_t{
// //    	pthread_t(sys_gettid())
// // }

// // 创建线程
// // tid: 线程ID通过tid返回
// // attr: 为线程属性
// // func: 为线程的启动例程
// // arg: 为传入线程函数的参数
// // 返回值: 成功返回0, 失败返回错误码
// pub fn pthread_create(tid: pthread_t,  )-> isize;

// // // 终止线程
// // // 终止本线程，可通过pthread_join查看设置的终止状态。成功返回0，失败返回错误码
// // pub fn pthread_exit(void *status) -> isize;

// // // 取消ID为tid的其他线程，成功返回0，失败返回错误码。
// // pub fn pthread_cancel(tid: pthread_t) -> isize; 

// // // 等待线程终止
// // // 成功返回0, 出错返回错误码。status返回线程终止参数。
// // pub fn pthread_join(tid: pthread_t, status: void*) -> isize;


// // // 线程终止清理函数

// // // 压入清理函数。成功返回0，失败返回错误码。
// // pub fn pthread_cleanup_push(void* (*pCleanFunc)(void *), void *arg) -> isize;

// // // 清除清理函数。当execute为0时，只清除函数而不调用它，反之则否。成功返回0，失败返回错误码。
// // pub fn pthread_clean_pop(int execute) -> int;