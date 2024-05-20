use alloc::collections::BTreeMap;

use basic::{constants::DeviceId, sync::Mutex};
use spin::Lazy;
use vfscore::utils::VfsNodeType;

struct DeviceIdManager {
    map: BTreeMap<u32, u32>,
}

impl DeviceIdManager {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }
    pub fn alloc(&mut self, inode_type: VfsNodeType) -> DeviceId {
        assert!(matches!(
            inode_type,
            VfsNodeType::CharDevice | VfsNodeType::BlockDevice
        ));
        let id = self.map.entry(inode_type.to_u32()).or_insert(0);
        *id += 1;
        DeviceId::new(inode_type.to_u32(), *id)
    }
}

pub trait InodeType2u32 {
    fn to_u32(&self) -> u32;
}

impl InodeType2u32 for VfsNodeType {
    fn to_u32(&self) -> u32 {
        match self {
            VfsNodeType::CharDevice => 2,
            VfsNodeType::BlockDevice => 3,
            _ => 0,
        }
    }
}

static DEVICE_ID_MANAGER: Lazy<Mutex<DeviceIdManager>> =
    Lazy::new(|| Mutex::new(DeviceIdManager::new()));

pub fn alloc_device_id(inode_type: VfsNodeType) -> DeviceId {
    DEVICE_ID_MANAGER.lock().alloc(inode_type)
}
