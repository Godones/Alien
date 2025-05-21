use core::any::Any;

use bpf_basic::{
    linux_bpf::{perf_event_sample_format, perf_sw_ids},
    perf::{bpf::BpfPerfEvent, PerfProbeArgs},
};
use constants::AlienResult;

use super::PerfEventOps;

impl PerfEventOps for BpfPerfEvent {
    fn enable(&mut self) -> AlienResult<()> {
        BpfPerfEvent::enable(self).unwrap();
        Ok(())
    }

    fn disable(&mut self) -> AlienResult<()> {
        BpfPerfEvent::disable(self).unwrap();
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn readable(&self) -> bool {
        BpfPerfEvent::readable(&self)
    }

    fn writeable(&self) -> bool {
        false
    }
}

pub fn perf_event_open_bpf(args: PerfProbeArgs) -> BpfPerfEvent {
    // For bpf prog output
    assert_eq!(args.config, perf_sw_ids::PERF_COUNT_SW_BPF_OUTPUT);
    assert_eq!(
        args.sample_type,
        Some(perf_event_sample_format::PERF_SAMPLE_RAW)
    );
    BpfPerfEvent::new(args)
}
