#![no_std]
#![forbid(unsafe_code)]
mod area;

extern crate alloc;

use alloc::collections::btree_map::Values;
use alloc::collections::BTreeMap;
pub use area::{VmArea, VmAreaEqual, VmAreaType};
use config::FRAME_SIZE;
use core::fmt::{Debug, Formatter};
use core::ops::Range;
use memory_addr::{PhysAddr, VirtAddr};
use page_table::{
    MappingFlags, PageSize, PagingError, PagingIf, PagingResult, Rv64PTE, Sv39PageTable,
};

pub trait PhysPage: Debug + Send + Sync {
    fn phys_addr(&self) -> PhysAddr;
    fn as_slice(&self) -> &[u8];
    fn as_mut_slice(&mut self) -> &mut [u8];
}

pub struct VmSpace<T: PagingIf<Rv64PTE>> {
    table: Sv39PageTable<T>,
    areas: BTreeMap<usize, VmAreaType>,
}

impl<T: PagingIf<Rv64PTE> + Debug> Debug for VmSpace<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VmSpace")
            .field("areas", &self.areas)
            .finish()
    }
}

impl<T: PagingIf<Rv64PTE>> VmSpace<T> {
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
        for (vaddr, phy_frame) in vm_area.mapper().iter() {
            let va = VirtAddr::from(*vaddr);
            let pa = phy_frame.phys_addr();
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
        for (vaddr, _) in vm_area.mapper().iter() {
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

    pub fn area_iter(&self) -> Values<usize, VmAreaType> {
        self.areas.values()
    }
}
