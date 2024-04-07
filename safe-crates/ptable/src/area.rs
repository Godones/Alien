use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};
use core::{
    fmt::{Debug, Formatter},
    ops::Range,
};

use config::FRAME_SIZE;
use page_table::MappingFlags;

use crate::PhysPage;

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

    pub fn read_data(&self, addr: usize, data: &mut [u8]) {
        match self {
            VmAreaType::VmArea(vm_area) => vm_area.read_data(addr, data),
            VmAreaType::VmAreaEqual(_) => panic!("read_data on VmAreaEqual"),
        }
    }

    pub fn write_data(&mut self, addr: usize, data: &[u8]) {
        match self {
            VmAreaType::VmArea(vm_area) => vm_area.write_data(addr, data),
            VmAreaType::VmAreaEqual(_) => panic!("write_data on VmAreaEqual"),
        }
    }

    pub fn permission(&self) -> MappingFlags {
        match self {
            VmAreaType::VmArea(vm_area) => vm_area.permission(),
            VmAreaType::VmAreaEqual(vm_area_equal) => vm_area_equal.permission(),
        }
    }
}
pub struct VmArea {
    v_range: Range<usize>,
    permission: MappingFlags,
    map: BTreeMap<usize, Box<dyn PhysPage>>,
}

impl Debug for VmArea {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VmArea")
            .field("v_range", &self.v_range)
            .field("permission", &self.permission)
            .field("map", &self.map)
            .finish()
    }
}

impl VmArea {
    pub fn new(
        v_range: Range<usize>,
        permission: MappingFlags,
        phy_frames: Vec<Box<dyn PhysPage>>,
    ) -> Self {
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

    pub(super) fn mapper(&self) -> &BTreeMap<usize, Box<dyn PhysPage>> {
        &self.map
    }

    pub fn clone_with(&self, phy_frames: Vec<Box<dyn PhysPage>>) -> Self {
        assert_eq!(self.size() / FRAME_SIZE, phy_frames.len());
        let mut phy_frames_map = BTreeMap::new();
        for (i, mut phy_frame) in phy_frames.into_iter().enumerate() {
            let start = self.start() + i * FRAME_SIZE;
            let old_phy_frame = self.map.get(&start).unwrap();
            // copy data
            phy_frame
                .as_mut_bytes()
                .copy_from_slice(old_phy_frame.as_bytes());
            phy_frames_map.insert(start, phy_frame);
        }
        Self {
            v_range: self.v_range.clone(),
            permission: self.permission,
            map: phy_frames_map,
        }
    }

    pub fn write_data(&mut self, addr: usize, data: &[u8]) {
        let mut current_start_address = addr;
        let mut data = data;
        for (va, pa) in self.map.iter_mut() {
            if current_start_address >= *va && current_start_address < va + FRAME_SIZE {
                let offset = current_start_address - va;
                let phy_buf = pa.as_mut_bytes();
                let w_len = core::cmp::min(phy_buf.len() - offset, data.len());
                phy_buf[offset..offset + w_len].copy_from_slice(&data[..w_len]);
                if w_len == data.len() {
                    return;
                }
                data = &data[w_len..];
                current_start_address = va + FRAME_SIZE;
            }
        }
    }

    pub fn read_data(&self, addr: usize, data: &mut [u8]) {
        let mut start = addr;
        let mut data = data;
        for (va, pa) in self.map.iter() {
            if start >= *va && start < va + FRAME_SIZE {
                let offset = start - va;
                let phy_buf = pa.as_bytes();
                let r_len = core::cmp::min(phy_buf.len() - offset, data.len());
                data[..r_len].copy_from_slice(&phy_buf[offset..offset + r_len]);
                if r_len == data.len() {
                    return;
                }
                data = &mut data[r_len..];
                start = va + FRAME_SIZE;
            }
        }
    }
}

#[derive(Debug, Clone)]
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
