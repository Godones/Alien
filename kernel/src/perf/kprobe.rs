use alloc::{boxed::Box, sync::Arc};
use core::any::Any;

use bpf_basic::perf::PerfProbeArgs;
use kprobe::{CallBackFunc, KprobeBuilder, ProbeArgs};
use rbpf::EbpfVmRaw;
use vfs::kfile::File;

use super::PerfEventOps;
use crate::{
    ebpf::{prog::BpfProg, BPF_HELPER_FUN_SET},
    kprobe::{register_kprobe, unregister_kprobe, KProbeContext, KernelKprobe, KprobeAuxiliary},
    trap::CommonTrapFrame,
};

#[derive(Debug)]
pub struct KprobePerfEvent {
    _args: PerfProbeArgs,
    kprobe: Arc<KernelKprobe>,
}

impl Drop for KprobePerfEvent {
    fn drop(&mut self) {
        unregister_kprobe(self.kprobe.clone());
    }
}

impl PerfEventOps for KprobePerfEvent {
    fn enable(&mut self) -> constants::AlienResult<()> {
        self.kprobe.enable();
        Ok(())
    }

    fn disable(&mut self) -> constants::AlienResult<()> {
        self.kprobe.disable();
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct KprobePerfCallBack {
    _bpf_prog_file: Arc<BpfProg>,
    vm: EbpfVmRaw<'static>,
}

unsafe impl Send for KprobePerfCallBack {}
unsafe impl Sync for KprobePerfCallBack {}

impl KprobePerfCallBack {
    fn new(bpf_prog_file: Arc<BpfProg>, vm: EbpfVmRaw<'static>) -> Self {
        Self {
            _bpf_prog_file: bpf_prog_file,
            vm,
        }
    }
}

impl CallBackFunc for KprobePerfCallBack {
    fn call(&self, trap_frame: &dyn ProbeArgs) {
        let trap_frame = trap_frame
            .as_any()
            .downcast_ref::<CommonTrapFrame>()
            .unwrap();
        let pt_regs = KProbeContext::from(trap_frame);
        let probe_context = unsafe {
            core::slice::from_raw_parts_mut(
                &pt_regs as *const KProbeContext as *mut u8,
                size_of::<KProbeContext>(),
            )
        };
        // log::error!("call kprobe callback");
        let res = self.vm.execute_program(probe_context);
        if res.is_err() {
            log::error!("kprobe callback error: {:?}", res);
        }
    }
}

impl KprobePerfEvent {
    pub fn set_bpf_prog(&mut self, bpf_prog: Arc<dyn File>) -> bpf_basic::Result<()> {
        let bpf_prog = bpf_prog.downcast_arc::<BpfProg>().unwrap();
        let prog = bpf_prog.insns();
        let prog = unsafe { core::slice::from_raw_parts_mut(prog.as_ptr() as *mut u8, prog.len()) };
        let mut vm = EbpfVmRaw::new(Some(prog)).unwrap();
        for (key, value) in BPF_HELPER_FUN_SET.iter() {
            vm.register_helper(*key, *value).unwrap();
        }
        // create a callback to execute the ebpf prog
        let callback = Box::new(KprobePerfCallBack::new(bpf_prog, vm));
        // update callback for kprobe
        self.kprobe.update_event_callback(callback);
        Ok(())
    }
}

fn perf_probe_arg_to_kprobe_builder(args: &PerfProbeArgs) -> KprobeBuilder<KprobeAuxiliary> {
    let symbol = &args.name;
    let addr = unsafe { ksym::addr_from_symbol(symbol).unwrap() as usize };
    // let addr = syscall_entry as usize;
    println_color!(32, "perf_probe: symbol: {}, addr: {:#x}", symbol, addr);
    let builder = KprobeBuilder::new(Some(symbol.clone()), addr, 0, |_| {}, |_| {}, false);
    builder
}

pub fn perf_event_open_kprobe(args: PerfProbeArgs) -> KprobePerfEvent {
    let symbol = &args.name;
    log::info!("create kprobe for symbol: {symbol}");
    let builder = perf_probe_arg_to_kprobe_builder(&args);
    let kprobe = register_kprobe(builder);
    println_color!(32, "create kprobe ok");
    KprobePerfEvent {
        _args: args,
        kprobe,
    }
}
