//! panic 处理

use core::{panic::PanicInfo, sync::atomic::AtomicBool};

use platform::{println, system_shutdown};
#[cfg(all(not(feature = "debug-eh-frame"), not(feature = "debug-frame-point")))]
use tracer::CompilerTracer;
#[cfg(feature = "debug-eh-frame")]
use tracer::DwarfTracer;
#[cfg(feature = "debug-frame-point")]
use tracer::FramePointTracer;
use tracer::{Tracer, TracerProvider};

use crate::symbol::find_symbol_with_addr;

/// 递归标志
static RECURSION: AtomicBool = AtomicBool::new(false);

/// 错误处理
///
/// 发生 panic 是进行结果处理.目前我们会读取符号表信息，进行堆栈回溯
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    if let Some(p) = info.location() {
        println!("line {}, file {}: {}", p.line(), p.file(), info.message());
    } else {
        println!("no location information available");
    }
    if !RECURSION.swap(true, core::sync::atomic::Ordering::SeqCst) {
        back_trace();
    }
    println!("!TEST FINISH!");
    system_shutdown();
}

#[derive(Clone)]
struct TracerProviderImpl;
impl TracerProvider for TracerProviderImpl {
    fn address2symbol(&self, addr: usize) -> Option<(usize, &'static str)> {
        find_symbol_with_addr(addr)
    }
}

#[cfg(feature = "debug-eh-frame")]
extern "C" {
    fn kernel_eh_frame();
    fn kernel_eh_frame_end();
    fn kernel_eh_frame_hdr();
    fn kernel_eh_frame_hdr_end();
}

#[cfg(feature = "debug-eh-frame")]
struct DwarfProviderImpl;

#[cfg(feature = "debug-eh-frame")]
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
    #[cfg(all(not(feature = "debug-eh-frame"), not(feature = "debug-frame-point")))]
    let tracer = CompilerTracer::new(TracerProviderImpl);
    #[cfg(feature = "debug-frame-point")]
    let tracer = FramePointTracer::new(TracerProviderImpl);
    #[cfg(feature = "debug-eh-frame")]
    let tracer = DwarfTracer::new(DwarfProviderImpl, TracerProviderImpl);
    for x in tracer.trace() {
        println!("[{:#x}] (+{:0>4x}) {}", x.func_addr, x.bias, x.func_name);
    }
    println!("---END   BACKTRACE---");
}
