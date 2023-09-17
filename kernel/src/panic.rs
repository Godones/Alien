//! panic 处理
use crate::sbi::system_shutdown;
use crate::trace::find_symbol_with_addr;
use core::panic::PanicInfo;
use core::sync::atomic::AtomicBool;
use tracer::{CompilerTracer, DwarfProvider, Tracer, TracerProvider};

/// 递归标志
static RECURSION: AtomicBool = AtomicBool::new(false);

/// 错误处理
///
/// 发生 panic 是进行结果处理.目前我们会读取符号表信息，进行堆栈回溯
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    if let Some(p) = info.location() {
        println!(
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        );
    } else {
        println!("no location information available");
    }
    if !RECURSION.swap(true, core::sync::atomic::Ordering::SeqCst) {
        back_trace();
    }
    println!("!TEST FINISH!");
    system_shutdown();
    loop {}
}

#[derive(Clone)]
struct TracerProviderImpl;
impl TracerProvider for TracerProviderImpl {
    fn address2symbol(&self, addr: usize) -> Option<(usize, &'static str)> {
        warn!("address2symbol: {:#x}", addr);
        find_symbol_with_addr(addr)
    }
}

extern "C" {
    fn kernel_eh_frame();
    fn kernel_eh_frame_end();
    fn kernel_eh_frame_hdr();
    fn kernel_eh_frame_hdr_end();
}
struct DwarfProviderImpl;
impl DwarfProvider for DwarfProviderImpl {
    fn kernel_eh_frame_hdr(&self) -> usize {
        kernel_eh_frame_hdr as usize
    }

    fn kernel_eh_frame(&self) -> usize {
        kernel_eh_frame as usize
    }

    fn kernel_eh_frame_hdr_end(&self) -> usize {
        kernel_eh_frame_hdr_end as usize
    }

    fn kernel_eh_frame_end(&self) -> usize {
        kernel_eh_frame_end as usize
    }
}

/// 打印堆栈回溯信息
fn back_trace() {
    println!("---START BACKTRACE---");
    let tracer = CompilerTracer::new(TracerProviderImpl);
    // let tracer = FramePointTracer::new(TracerProviderImpl);
    // let tracer = DwarfTracer::new(DwarfProviderImpl,TracerProviderImpl);
    for x in tracer.trace() {
        println!("[{:#x}] (+{:0>4x}) {}", x.func_addr, x.bias, x.func_name);
    }
    println!("---END   BACKTRACE---");
}
