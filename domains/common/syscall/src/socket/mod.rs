use alloc::sync::Arc;

use constants::{
    net::{Domain, SocketType, SOCKET_TYPE_MASK},
    AlienError, AlienResult,
};
use interface::TaskDomain;
use log::info;

pub fn sys_socket(
    _task_domain: &Arc<dyn TaskDomain>,
    domain: usize,
    s_type: usize,
    _protocol: usize,
) -> AlienResult<isize> {
    let domain = Domain::try_from(domain).map_err(|_| AlienError::EAFNOSUPPORT)?;
    let socket_type =
        SocketType::try_from(s_type & SOCKET_TYPE_MASK as usize).map_err(|_| AlienError::EINVAL)?;
    info!("socket domain: {:?}, type: {:?}", domain, socket_type);
    // panic!("sys_socket not implemented");
    Err(AlienError::ENOSYS)
}
