use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::{cmp::min, fmt::Debug};

use config::*;
use mem::{VmmPageAllocator, FRAME_REF_MANAGER};
use page_table::{
    addr::{align_up_4k, PhysAddr, VirtAddr},
    pte::MappingFlags,
    table::Sv39PageTable,
};
use spin::Lazy;
use xmas_elf::program::{SegmentData, Type};

use crate::{
    fs,
    ipc::ShmInfo,
    mm::elf::{ELFError, ELFInfo, ELFReader},
    trap::TrapFrame,
};

extern "C" {
    fn strampoline();
}
#[derive(Debug)]
pub struct UserStack {
    pub virt_stack_top: usize,
    pub stack_top: usize,
    pub stack_bottom: usize,
}

impl UserStack {
    pub fn new(phy_stack_top: usize, virt_stack_top: usize) -> Self {
        Self {
            virt_stack_top,
            stack_top: phy_stack_top,
            stack_bottom: phy_stack_top - FRAME_SIZE,
        }
    }
    pub fn push(&mut self, data: usize) -> Result<usize, &'static str> {
        if self.stack_top - 8 < self.stack_bottom {
            return Err("Stack Overflow");
        }
        unsafe {
            self.stack_top -= 8;
            *(self.stack_top as *mut usize) = data;
        }
        trace!(
            "stack top: {:#x}, data:{:#x?}",
            self.virt_stack_top - (FRAME_SIZE - (self.stack_top - self.stack_bottom)),
            data
        );
        Ok(self.virt_stack_top - (FRAME_SIZE - (self.stack_top - self.stack_bottom)))
    }

    pub fn push_str(&mut self, data: &str) -> Result<usize, &'static str> {
        self.push_bytes(data.as_bytes())
    }

    pub fn push_bytes(&mut self, data: &[u8]) -> Result<usize, &'static str> {
        let len = data.len();
        // align 8
        let start = self.stack_top - len;
        let start = start & !7;
        if start < self.stack_bottom {
            return Err("Stack Overflow");
        }
        unsafe {
            self.stack_top = start;
            let ptr = self.stack_top as *mut u8;
            ptr.copy_from_nonoverlapping(data.as_ptr(), len);
        }
        trace!(
            "stack top: {:#x}",
            self.virt_stack_top - (FRAME_SIZE - (self.stack_top - self.stack_bottom))
        );
        Ok(self.virt_stack_top - (FRAME_SIZE - (self.stack_top - self.stack_bottom)))
    }

    pub fn align_to(&mut self, align: usize) -> Result<usize, &'static str> {
        let start = self.stack_top & !(align - 1);
        if start < self.stack_bottom {
            return Err("Stack Overflow");
        }
        self.stack_top = start;
        Ok(self.virt_stack_top - (FRAME_SIZE - (self.stack_top - self.stack_bottom)))
    }
}

pub fn build_thread_address_space(
    table: &mut Sv39PageTable<VmmPageAllocator>,
    thread_num_within: usize,
) -> &'static mut TrapFrame {
    let address = TRAP_CONTEXT_BASE - FRAME_SIZE * thread_num_within;
    let (_virt_dst, phy_dst, _) = table
        .map_region_no_target(
            VirtAddr::from(address),
            FRAME_SIZE,
            "RWVAD".into(),
            true,
            false,
        )
        .unwrap()
        .next()
        .unwrap();
    // copy data
    let (phy, _flag, page_size) = table.query(VirtAddr::from(TRAP_CONTEXT_BASE)).unwrap();
    assert_eq!(usize::from(page_size), FRAME_SIZE);
    // copy data
    let src_ptr = phy.as_usize() as *const u8;
    let dst_ptr = phy_dst.as_usize() as *mut u8;
    unsafe {
        core::ptr::copy(src_ptr, dst_ptr, usize::from(page_size));
    }
    TrapFrame::from_raw_ptr(dst_ptr as *mut TrapFrame)
}

pub fn build_cow_address_space(
    p_table: &mut Sv39PageTable<VmmPageAllocator>,
    shm: BTreeMap<usize, ShmInfo>,
) -> Sv39PageTable<VmmPageAllocator> {
    let mut address_space = Sv39PageTable::<VmmPageAllocator>::try_new().unwrap();
    for (v_addr, target) in p_table.get_record().into_iter() {
        trace!("v_addr: {:?}, target: {}", v_addr, target);
        let (phy, flag, page_size) = p_table.query(v_addr).unwrap();

        // shm should remap, we can't use cow for it
        let is_in_segs = |addr: usize| -> bool {
            for (_id, shminfo) in shm.iter() {
                if addr >= shminfo.start_va && addr < shminfo.end_va {
                    return true;
                }
            }
            false
        };

        if v_addr.as_usize() == TRAP_CONTEXT_BASE {
            // for Trap_context, we remap it
            assert_eq!(usize::from(page_size), TRAMPOLINE - TRAP_CONTEXT_BASE);
            let dst = address_space
                .map_no_target(v_addr, page_size, flag, false)
                .unwrap();
            // copy data
            let src_ptr = phy.as_usize() as *const u8;
            let dst_ptr = dst.as_usize() as *mut u8;
            unsafe {
                core::ptr::copy(src_ptr, dst_ptr, usize::from(page_size));
            }
        } else if is_in_segs(v_addr.as_usize()) {
            // for shm, we now skip it
            address_space.map(v_addr, phy, page_size, flag).unwrap();
        } else {
            // cow
            // checkout whether pte flags has `W` flag
            let mut flags = flag.clone();
            if !flag.contains(MappingFlags::V) {
                // if flags is not valid, we just map it
                address_space.map(v_addr, phy, page_size, flags).unwrap();
                if target {
                    address_space.get_record_mut().insert(v_addr, true);
                }
                continue;
            }
            if flag.contains(MappingFlags::W) {
                flags -= MappingFlags::W;
                flags |= MappingFlags::RSD; // we use the RSD flag to indicate that this page is a cow page
                                            // update parent's flag and clear dirty
                p_table.modify_pte_flags(v_addr, flags, false).unwrap();
            }
            address_space.map(v_addr, phy, page_size, flags).unwrap();
            // add ref for alloc page
            if target {
                for i in 0..usize::from(page_size) / FRAME_SIZE {
                    let page_number = (phy + FRAME_SIZE * i).as_usize() >> FRAME_BITS;
                    // FRAME_REF_MANAGER.lock().get_ref(page_number);
                    FRAME_REF_MANAGER.lock().add_ref(page_number);
                }
                address_space.get_record_mut().insert(v_addr, true);
            }
        }
    }
    address_space
}

static LD_MUSL_RV64_CACHE: Lazy<Vec<u8>> = Lazy::new(|| {
    let mut data = vec![];
    fs::read_all("/tests/libc.so", &mut data);
    data
});

pub fn build_elf_address_space(
    elf: &[u8],
    args: &mut Vec<String>,
    name: &str,
) -> Result<ELFInfo, ELFError> {
    let mut address_space = Sv39PageTable::<VmmPageAllocator>::try_new().unwrap();
    const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
    if elf[0..4] != ELF_MAGIC {
        return Err(ELFError::NotELF);
    }
    let elf = xmas_elf::ElfFile::new(elf).map_err(|_| ELFError::NotELF)?;
    // check whether it's a dynamic linked elf
    if let Some(inter) = elf
        .program_iter()
        .find(|ph| ph.get_type().unwrap() == Type::Interp)
    {
        let data = match inter.get_data(&elf).unwrap() {
            SegmentData::Undefined(data) => data,
            _ => return Err(ELFError::NoEntrySegment),
        };
        let path = core::str::from_utf8(data).unwrap();
        assert!(
            path.starts_with("/lib/ld-musl-riscv64"),
            "interpreter: {:?}",
            path
        );
        let mut new_args = vec!["/tests/libc.so\0".to_string()];
        // args.remove(0); // remove the first arg
        new_args.extend(args.clone());
        *args = new_args;
        // load interpreter
        warn!("load interpreter: {}, new_args:{:?}", path, args);
        return build_elf_address_space(&LD_MUSL_RV64_CACHE, args, "/tests/libc.so");
    }

    // calculate bias for dynamic linked elf
    // if elf is static linked, bias is 0
    let bias = match elf.header.pt2.type_().as_type() {
        // static
        xmas_elf::header::Type::Executable => 0,
        xmas_elf::header::Type::SharedObject => {
            match elf
                .program_iter()
                .filter(|ph| ph.get_type().unwrap() == Type::Interp)
                .count()
            {
                // It's a loader!
                0 => ELF_BASE_RELOCATE,
                // It's a dynamically linked ELF.
                1 => 0,
                // Emmm, It has multiple interpreters.
                _ => return Err(ELFError::NotSupported),
            }
        }
        _ => return Err(ELFError::NotSupported),
    };
    trace!("bias: {:#x}", bias);

    let tls = elf
        .program_iter()
        .find(|x| x.get_type().unwrap() == Type::Tls)
        .map(|ph| ph.virtual_addr())
        .unwrap_or(0);

    warn!("ELF tls: {:#x}", tls);

    let mut break_addr = 0usize;
    elf.program_iter()
        .filter(|ph| ph.get_type() == Ok(Type::Load))
        .for_each(|ph| {
            let start_addr = ph.virtual_addr() as usize + bias;
            let end_addr = start_addr + ph.mem_size() as usize;
            let mut permission: MappingFlags = "UVAD".into();
            let ph_flags = ph.flags();
            if ph_flags.is_read() {
                permission |= MappingFlags::R;
            }
            if ph_flags.is_write() {
                permission |= MappingFlags::W;
            }
            if ph_flags.is_execute() {
                permission |= MappingFlags::X;
            }
            let vaddr = VirtAddr::from(start_addr).align_down_4k();
            let end_vaddr = VirtAddr::from(end_addr).align_up_4k();
            // 记录程序地址空间的最大地址
            break_addr = end_addr;
            let len = end_vaddr.as_usize() - vaddr.as_usize();
            warn!(
                "load segment: {:#x} - {:#x} -> {:#x}-{:#x}, permission: {:?}",
                start_addr,
                end_addr,
                vaddr.as_usize(),
                end_vaddr.as_usize(),
                permission
            );
            let mut data =
                &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize];
            let map_info = address_space
                .map_region_no_target(vaddr, len, permission, false, false)
                .unwrap();
            // copy data
            let mut page_offset = start_addr & (FRAME_SIZE - 1);
            let mut count = 0;
            map_info
                .into_iter()
                .for_each(|(_vir, phy, page_size)| unsafe {
                    let size: usize = page_size.into();
                    let min = min(size - page_offset, data.len());
                    let dst = (phy.as_usize() + page_offset) as *mut u8;
                    core::ptr::copy(data.as_ptr(), dst, min);
                    data = &data[min..];
                    count += min;
                    page_offset = 0;
                });
            assert_eq!(count, ph.file_size() as usize);
        });

    // 地址向上取整对齐4
    let ceil_addr = align_up_4k(break_addr + FRAME_SIZE);
    // 留出一个用户栈的位置+隔离页
    let top = ceil_addr + USER_STACK_SIZE + FRAME_SIZE;
    warn!(
        "user stack: {:#x} - {:#x}",
        top - USER_STACK_SIZE - FRAME_SIZE,
        top - FRAME_SIZE
    );
    // map user stack
    address_space
        .map_region_no_target(
            VirtAddr::from(top - USER_STACK_SIZE - FRAME_SIZE),
            USER_STACK_SIZE,
            "RWUAD".into(),
            false,
            true,
        )
        .unwrap();
    // 初始化一个有效页
    address_space
        .validate(VirtAddr::from(top - FRAME_SIZE * 2), "RWUVAD".into())
        .unwrap();
    let heap_bottom = top;
    // align to 4k
    warn!("trap context: {:#x} - {:#x}", TRAP_CONTEXT_BASE, TRAMPOLINE);
    address_space
        .map_region_no_target(
            VirtAddr::from(TRAP_CONTEXT_BASE),
            TRAMPOLINE - TRAP_CONTEXT_BASE,
            "RWVAD".into(),
            true,
            false,
        )
        .unwrap();
    warn!(
        "TRAMPOLINE: {:#x} - {:#x}",
        TRAMPOLINE,
        TRAMPOLINE + FRAME_SIZE
    );
    address_space
        .map_region(
            VirtAddr::from(TRAMPOLINE),
            PhysAddr::from(strampoline as usize),
            FRAME_SIZE,
            "RXVAD".into(),
            true,
        )
        .unwrap();

    let res = if let Some(phdr) = elf
        .program_iter()
        .find(|ph| ph.get_type() == Ok(Type::Phdr))
    {
        // if phdr exists in program header, use it
        Ok(phdr.virtual_addr())
    } else if let Some(elf_addr) = elf
        .program_iter()
        .find(|ph| ph.get_type() == Ok(Type::Load) && ph.offset() == 0)
    {
        // otherwise, check if elf is loaded from the beginning, then phdr can be inferred.
        // Ok(elf_addr.virtual_addr())
        Ok(elf_addr.virtual_addr() + elf.header.pt2.ph_offset())
    } else {
        warn!("elf: no phdr found, tls might not work");
        Err(ELFError::NoEntrySegment)
    }
    .unwrap_or(0);
    warn!(
        "entry: {:#x}, phdr:{:#x}",
        elf.header.pt2.entry_point() + bias as u64,
        res + bias as u64
    );
    // relocate if elf is dynamically linked
    if bias != 0 {
        if let Ok(kvs) = elf.relocate_plt(bias) {
            kvs.into_iter().for_each(|kv| {
                trace!("relocate: {:#x} -> {:#x}", kv.0, kv.1);
                let (addr, ..) = address_space.query(VirtAddr::from(kv.0)).unwrap();
                unsafe { (addr.as_usize() as *mut usize).write(kv.1) }
            });
            info!("relocate plt done")
        }
        if let Ok(kvs) = elf.relocate_dyn(bias) {
            kvs.into_iter().for_each(|kv| {
                trace!("relocate: {:#x} -> {:#x}", kv.0, kv.1);
                let (addr, ..) = address_space.query(VirtAddr::from(kv.0)).unwrap();
                unsafe { (addr.as_usize() as *mut usize).write(kv.1) }
            });
            info!("relocate dyn done")
        }
    }
    Ok(ELFInfo {
        address_space,
        entry: elf.header.pt2.entry_point() as usize + bias,
        stack_top: top - FRAME_SIZE,
        heap_bottom,
        ph_num: elf.header.pt2.ph_count() as usize,
        ph_entry_size: elf.header.pt2.ph_entry_size() as usize,
        ph_drift: res as usize + bias,
        tls: tls as usize + bias,
        bias,
        name: name.to_string(),
    })
}
