use alloc::string::ToString;

use kprobe::{register_kprobe, unregister_kprobe, KprobeBuilder, ProbeArgs};

use crate::{
    kprobe::{KPROBE_MANAGER, KPROBE_POINT_LIST},
    trap::CommonTrapFrame,
};

#[inline(never)]
#[no_mangle]
fn detect_func(x: usize, y: usize) -> usize {
    let hart = 0;
    println_color!(34, "detect_func: hart_id: {}, x: {}, y:{}", hart, x, y);
    hart
}

fn pre_handler(regs: &dyn ProbeArgs) {
    let pt_regs = regs.as_any().downcast_ref::<CommonTrapFrame>().unwrap();
    println_color!(34, "pre_handler: is kernel: {}", pt_regs.is_kernel());
}

fn post_handler(regs: &dyn ProbeArgs) {
    let pt_regs = regs.as_any().downcast_ref::<CommonTrapFrame>().unwrap();
    println_color!(34, "post_handler: is kernel: {}", pt_regs.is_kernel());
}

pub fn kprobe_test() {
    println_color!(
        34,
        "kprobe test for [detect_func]: {:#x}",
        detect_func as usize
    );
    let kprobe_builder = KprobeBuilder::new(
        None,
        detect_func as usize,
        0,
        pre_handler,
        post_handler,
        true,
    );

    let mut manager = KPROBE_MANAGER.lock();
    let mut kprobe_list = KPROBE_POINT_LIST.lock();

    let kprobe = register_kprobe(&mut manager, &mut kprobe_list, kprobe_builder);
    let new_pre_handler = |regs: &dyn ProbeArgs| {
        let pt_regs = regs.as_any().downcast_ref::<CommonTrapFrame>().unwrap();
        println_color!(34, "new_pre_handler: is kernel: {}", pt_regs.is_kernel());
    };

    let builder2 = KprobeBuilder::new(
        Some("kprobe::detect_func".to_string()),
        detect_func as usize,
        0,
        new_pre_handler,
        post_handler,
        true,
    );
    let kprobe2 = register_kprobe(&mut manager, &mut kprobe_list, builder2);
    drop(manager);
    drop(kprobe_list);
    println_color!(
        34,
        "install 2 kprobes at [detect_func]: {:#x}",
        detect_func as usize
    );
    detect_func(1, 2);

    let mut manager = KPROBE_MANAGER.lock();
    let mut kprobe_list = KPROBE_POINT_LIST.lock();
    unregister_kprobe(&mut manager, &mut kprobe_list, kprobe);
    unregister_kprobe(&mut manager, &mut kprobe_list, kprobe2);
    println_color!(
        34,
        "uninstall 2 kprobes at [detect_func]: {:#x}",
        detect_func as usize
    );

    detect_func(3, 4);
    println_color!(34, "kprobe test passed");
}
