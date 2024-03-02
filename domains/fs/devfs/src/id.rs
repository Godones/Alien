use alloc::collections::BTreeMap;
use ksync::Mutex;
use spin::Lazy;
use vfscore::utils::VfsNodeType;

#[derive(Copy, Clone, Debug, Eq, PartialOrd, PartialEq, Hash, Ord)]
pub struct DeviceId {
    major: u32,
    minor: u32,
}

impl From<u64> for DeviceId {
    fn from(id: u64) -> Self {
        Self {
            major: (id >> 32) as u32,
            minor: (id & 0xffffffff) as u32,
        }
    }
}

pub struct DeviceIdManager {
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

static DEVICE_ID_MANAGER: Lazy<Mutex<DeviceIdManager>> =
    Lazy::new(|| Mutex::new(DeviceIdManager::new()));

pub fn alloc_device_id(inode_type: VfsNodeType) -> DeviceId {
    DEVICE_ID_MANAGER.lock().alloc(inode_type)
}
