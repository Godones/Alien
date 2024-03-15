#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use config::FRAME_SIZE;
use core::fmt::{Debug, Formatter};
use core::ops::{Deref, Range};
use page_table::riscv::Sv39PageTable;
use page_table::PageSize;

pub use page_table::{MappingFlags, PagingError, PagingIf, PagingResult};

pub use memory_addr::{PhysAddr, VirtAddr};

#[derive(Debug)]
pub struct PhyFrame {
    meta: Box<dyn PhyPageMeta>,
}

pub trait PhyPageMeta: Debug + Send + Sync {
    fn start_addr(&self) -> usize;
    fn size(&self) -> usize;
}

impl Deref for PhyFrame {
    type Target = Box<dyn PhyPageMeta>;

    fn deref(&self) -> &Self::Target {
        &self.meta
    }
}

impl PhyFrame {
    pub fn new(meta: Box<dyn PhyPageMeta>) -> Self {
        Self { meta }
    }
}

#[derive(Debug)]
pub struct VmArea {
    v_range: Range<usize>,
    permission: MappingFlags,
    map: BTreeMap<usize, PhyFrame>,
}

impl VmArea {
    pub fn new(v_range: Range<usize>, permission: MappingFlags, phy_frames: Vec<PhyFrame>) -> Self {
        assert_eq!(v_range.start % FRAME_SIZE, 0);
        assert_eq!(v_range.end % FRAME_SIZE, 0);
        assert_eq!((v_range.end - v_range.start) / FRAME_SIZE, phy_frames.len());
        let mut phy_frames_map = BTreeMap::new();
        for (i, phy_frame) in phy_frames.into_iter().enumerate() {
            phy_frames_map.insert(v_range.start + i * FRAME_SIZE, phy_frame);
        }
        Self {
            v_range,
            permission,
            map: phy_frames_map,
        }
    }
    pub fn range(&self) -> Range<usize> {
        self.v_range.clone()
    }
    pub fn permission(&self) -> MappingFlags {
        self.permission
    }

    pub fn size(&self) -> usize {
        self.v_range.end - self.v_range.start
    }

    pub fn start(&self) -> usize {
        self.v_range.start
    }
}

#[derive(Debug)]
pub struct VmAreaEqual {
    v_range: Range<usize>,
    permission: MappingFlags,
}

impl VmAreaEqual {
    pub fn new(v_range: Range<usize>, permission: MappingFlags) -> Self {
        Self {
            v_range,
            permission,
        }
    }
    pub fn range(&self) -> Range<usize> {
        self.v_range.clone()
    }
    pub fn permission(&self) -> MappingFlags {
        self.permission
    }

    pub fn start_addr(&self) -> usize {
        self.v_range.start
    }

    pub fn size(&self) -> usize {
        self.v_range.end - self.v_range.start
    }
}

#[derive(Debug)]
pub enum VmAreaType {
    VmArea(VmArea),
    VmAreaEqual(VmAreaEqual),
}

impl VmAreaType {
    pub fn size(&self) -> usize {
        match self {
            VmAreaType::VmArea(vm_area) => vm_area.size(),
            VmAreaType::VmAreaEqual(vm_area_equal) => vm_area_equal.size(),
        }
    }
}

pub struct VmSpace<T: PagingIf> {
    table: Sv39PageTable<T>,
    areas: BTreeMap<usize, VmAreaType>,
}

impl<T: PagingIf> Debug for VmSpace<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VmSpace")
            .field("areas", &self.areas)
            .finish()
    }
}

impl<T: PagingIf> VmSpace<T> {
    pub fn new() -> Self {
        Self {
            table: Sv39PageTable::try_new().unwrap(),
            areas: BTreeMap::new(),
        }
    }

    pub fn root_paddr(&self) -> usize {
        self.table.root_paddr().as_usize()
    }

    pub fn query(&self, v_addr: usize) -> PagingResult<(PhysAddr, MappingFlags, PageSize)> {
        self.table.query(VirtAddr::from(v_addr))
    }

    pub fn map(&mut self, vm_area: VmAreaType) -> PagingResult {
        match vm_area {
            VmAreaType::VmArea(vm_area) => self.map_vm_area(vm_area)?,
            VmAreaType::VmAreaEqual(vm_area_equal) => self.map_vm_area_equal(vm_area_equal)?,
        }
        Ok(())
    }

    fn map_vm_area_equal(&mut self, vm_area_equal: VmAreaEqual) -> PagingResult {
        let v_start = VirtAddr::from(vm_area_equal.start_addr());
        let p_start = PhysAddr::from(vm_area_equal.start_addr());
        self.table.map_region(
            v_start,
            p_start,
            vm_area_equal.size(),
            vm_area_equal.permission(),
            true,
        )?;
        self.areas.insert(
            vm_area_equal.start_addr(),
            VmAreaType::VmAreaEqual(vm_area_equal),
        );
        Ok(())
    }

    fn map_vm_area(&mut self, vm_area: VmArea) -> PagingResult {
        for (vaddr, phy_frame) in vm_area.map.iter() {
            let va = VirtAddr::from(*vaddr);
            let pa = PhysAddr::from(phy_frame.start_addr());
            self.table
                .map(va, pa, PageSize::Size4K, vm_area.permission())?;
        }
        self.areas
            .insert(vm_area.start(), VmAreaType::VmArea(vm_area));
        Ok(())
    }

    pub fn unmap(&mut self, v_addr: usize) -> PagingResult {
        assert_eq!(v_addr % FRAME_SIZE, 0);
        let ty = self.areas.remove(&v_addr).ok_or(PagingError::NotMapped)?;
        match ty {
            VmAreaType::VmArea(vm_area) => self.unmap_vm_area(vm_area)?,
            VmAreaType::VmAreaEqual(vm_area_equal) => self.unmap_vm_area_equal(vm_area_equal)?,
        }
        Ok(())
    }

    fn unmap_vm_area(&mut self, vm_area: VmArea) -> PagingResult {
        for (vaddr, _) in vm_area.map.iter() {
            self.table.unmap(VirtAddr::from(*vaddr))?;
        }
        Ok(())
    }

    fn unmap_vm_area_equal(&mut self, vm_area_equal: VmAreaEqual) -> PagingResult {
        let v_start = VirtAddr::from(vm_area_equal.start_addr());
        self.table.unmap_region(v_start, vm_area_equal.size())?;
        Ok(())
    }

    pub fn is_mapped(&self, v_addr: usize) -> bool {
        for (start, area) in self.areas.iter() {
            if v_addr >= *start && v_addr < (*start + area.size()) {
                return true;
            }
        }
        false
    }

    pub fn protect(&mut self, v_range: Range<usize>, permission: MappingFlags) -> PagingResult {
        let v_start = VirtAddr::from(v_range.start);
        let v_end = VirtAddr::from(v_range.end);
        assert!(v_start.is_aligned(FRAME_SIZE));
        assert!(v_end.is_aligned(FRAME_SIZE));
        self.table.update(v_start, None, Some(permission))?;
        Ok(())
    }
}