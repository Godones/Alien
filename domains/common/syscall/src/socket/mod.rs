use alloc::sync::Arc;
use core::net::{Ipv4Addr, SocketAddrV4};

use basic::{
    constants::net::{
        Domain, ShutdownFlag, SocketAddrIn, SocketAddrInRaw, SocketLevel, SocketOption, SocketType,
        TcpSocketOption, SOCKET_TYPE_MASK,
    },
    AlienError, AlienResult,
};
use interface::{NetDomain, SchedulerDomain, SocketArgTuple, TaskDomain, VfsDomain};
use log::error;
use rref::{RRef, RRefVec};

pub fn sys_socket(
    task_domain: &Arc<dyn TaskDomain>,
    vfs_domain: &Arc<dyn VfsDomain>,
    net_stack_domain: &Arc<dyn NetDomain>,
    domain: usize,
    s_type: usize,
    protocol: usize,
) -> AlienResult<isize> {
    let domain = Domain::try_from(domain).map_err(|_| AlienError::EAFNOSUPPORT)?;
    let socket_type =
        SocketType::try_from(s_type & SOCKET_TYPE_MASK as usize).map_err(|_| AlienError::EINVAL)?;
    error!(
        "<sys_socket> socket domain: {:?}, type: {:?}",
        domain, socket_type
    );
    let socket_id = net_stack_domain.socket(domain, socket_type, protocol)?;
    let inode_id = vfs_domain.do_socket(socket_id)?;
    let fd = task_domain.add_fd(inode_id)?;
    Ok(fd as isize)
}

pub fn sys_socket_pair(
    task_domain: &Arc<dyn TaskDomain>,
    vfs_domain: &Arc<dyn VfsDomain>,
    net_stack_domain: &Arc<dyn NetDomain>,
    domain: usize,
    s_type: usize,
    protocol: usize,
    sv: usize,
) -> AlienResult<isize> {
    let domain = Domain::try_from(domain).map_err(|_| AlienError::EAFNOSUPPORT)?;
    let socket_type =
        SocketType::try_from(s_type & SOCKET_TYPE_MASK as usize).map_err(|_| AlienError::EINVAL)?;
    error!(
        "<sys_socket_pair> socket domain: {:?}, type: {:?}",
        domain, socket_type
    );
    if protocol != 0 || sv == 0 {
        return Err(AlienError::EINVAL);
    }
    let (id1, id2) = net_stack_domain.socket_pair(domain, socket_type)?;
    let inode_id1 = vfs_domain.do_socket(id1)?;
    let inode_id2 = vfs_domain.do_socket(id2)?;
    let fd1 = task_domain.add_fd(inode_id1)?;
    let fd2 = task_domain.add_fd(inode_id2)?;
    task_domain.write_val_to_user(sv, &[fd1 as u32, fd2 as u32])?;
    Ok(0)
}

pub fn sys_bind(
    task_domain: &Arc<dyn TaskDomain>,
    vfs_domain: &Arc<dyn VfsDomain>,
    net_stack_domain: &Arc<dyn NetDomain>,
    fd: usize,
    addr: usize,
    addr_len: usize,
) -> AlienResult<isize> {
    error!(
        "<sys_bind> fd: {}, addr: {:#x}, addr_len: {}",
        fd, addr, addr_len
    );
    let inode_id = task_domain.get_fd(fd)?;
    let socket_id = vfs_domain.socket_id(inode_id)?;
    let addr_raw = task_domain.read_val_from_user::<SocketAddrInRaw>(addr)?;
    let addr = SocketAddrIn::from(addr_raw);
    error!("<sys_bind> addr: {:?}", addr);
    let domain = Domain::try_from(addr.family as usize).map_err(|_| AlienError::EAFNOSUPPORT)?;
    if domain != Domain::AF_INET {
        return Err(AlienError::EAFNOSUPPORT);
    }
    let reuse_socket_id = net_stack_domain.bind(socket_id, &RRef::new(addr))?;
    if reuse_socket_id.is_some() {
        panic!("now we don't support reuse socket")
    }
    Ok(0)
}

pub fn sys_listen(
    task_domain: &Arc<dyn TaskDomain>,
    vfs_domain: &Arc<dyn VfsDomain>,
    net_stack_domain: &Arc<dyn NetDomain>,
    fd: usize,
    backlog: usize,
) -> AlienResult<isize> {
    error!("<sys_listen> fd: {}, backlog: {}", fd, backlog);
    let inode_id = task_domain.get_fd(fd)?;
    let socket_id = vfs_domain.socket_id(inode_id)?;
    net_stack_domain.listen(socket_id, backlog)?;
    Ok(0)
}

pub fn sys_accept(
    task_domain: &Arc<dyn TaskDomain>,
    vfs_domain: &Arc<dyn VfsDomain>,
    net_stack_domain: &Arc<dyn NetDomain>,
    scheduler_domain: &Arc<dyn SchedulerDomain>,
    fd: usize,
    addr: usize,
    addr_len: usize,
) -> AlienResult<isize> {
    error!(
        "<sys_accept> fd: {}, addr: {:#x}, addr_len: {}",
        fd, addr, addr_len
    );
    let inode_id = task_domain.get_fd(fd)?;
    let socket_id = vfs_domain.socket_id(inode_id)?;
    loop {
        let new_socket_id = net_stack_domain.accept(socket_id);
        match new_socket_id {
            Ok(new_socket_id) => {
                if addr != 0 {
                    let remote_addr = RRef::new(SocketAddrIn::default());
                    let remote_addr = net_stack_domain.remote_addr(new_socket_id, remote_addr)?;
                    let raw = SocketAddrInRaw::from(*remote_addr);
                    task_domain.write_val_to_user(addr, &raw)?;
                    let len = core::mem::size_of::<SocketAddrInRaw>();
                    task_domain.write_val_to_user(addr_len, &len)?;
                }
                let inode_id = vfs_domain.do_socket(new_socket_id)?;
                let fd = task_domain.add_fd(inode_id)?;
                return Ok(fd as isize);
            }
            Err(AlienError::EBLOCKING) => scheduler_domain.yield_now()?,
            Err(err) => return Err(err),
        }
        // check if there is a EINTR signal
    }
}

pub fn sys_connect(
    task_domain: &Arc<dyn TaskDomain>,
    vfs_domain: &Arc<dyn VfsDomain>,
    net_stack_domain: &Arc<dyn NetDomain>,
    scheduler_domain: &Arc<dyn SchedulerDomain>,
    fd: usize,
    addr: usize,
    addr_len: usize,
) -> AlienResult<isize> {
    error!(
        "<sys_connect> fd: {}, addr: {:#x}, addr_len: {}",
        fd, addr, addr_len
    );
    let inode_id = task_domain.get_fd(fd)?;
    let socket_id = vfs_domain.socket_id(inode_id)?;
    let addr_raw = task_domain.read_val_from_user::<SocketAddrInRaw>(addr)?;
    let addr = SocketAddrIn::from(addr_raw);
    let domain = Domain::try_from(addr.family as usize).map_err(|_| AlienError::EAFNOSUPPORT)?;
    if domain != Domain::AF_INET {
        return Err(AlienError::EAFNOSUPPORT);
    }
    let socket_addr = SocketAddrV4::new(addr.addr, addr.in_port.to_be());
    let socket_addr = RRef::new(socket_addr);

    loop {
        let res = net_stack_domain.connect(socket_id, &socket_addr);
        match res {
            Ok(_) => return Ok(0),
            Err(AlienError::EBLOCKING) => {}
            Err(err) => return Err(err),
        }
        scheduler_domain.yield_now()?;
        // check if there is a EINTR signal
    }
}

pub fn sys_recvfrom(
    task_domain: &Arc<dyn TaskDomain>,
    vfs_domain: &Arc<dyn VfsDomain>,
    net_stack_domain: &Arc<dyn NetDomain>,
    scheduler_domain: &Arc<dyn SchedulerDomain>,
    fd: usize,
    buf: usize,
    len: usize,
    flags: usize,
    addr: usize,
    addr_len: usize,
) -> AlienResult<isize> {
    error!(
        "<recv_from> fd: {}, buf: {:#x}, len: {}, flags: {}, addr: {:#x}, addr_len: {}",
        fd, buf, len, flags, addr, addr_len
    );
    let inode_id = task_domain.get_fd(fd)?;
    let socket_id = vfs_domain.socket_id(inode_id)?;

    let remote_addr = RRef::new(SocketAddrIn::default());
    let mut tmp_arg_tuple = RRef::new(SocketArgTuple {
        buf: RRefVec::new(0, len),
        addr: remote_addr,
        len: 0,
    });
    loop {
        let res = net_stack_domain.recv_from(socket_id, tmp_arg_tuple);
        match res {
            Ok(arg_tuple) => {
                if arg_tuple.len != 0 {
                    task_domain.copy_to_user(buf, &arg_tuple.buf.as_slice()[..arg_tuple.len])?;
                    if addr != 0 {
                        let raw = SocketAddrInRaw::from(*arg_tuple.addr);
                        task_domain.write_val_to_user(addr, &raw)?;
                        let len = core::mem::size_of::<SocketAddrInRaw>();
                        task_domain.write_val_to_user(addr_len, &len)?;
                    }
                    return Ok(arg_tuple.len as isize);
                } else {
                    tmp_arg_tuple = arg_tuple;
                    scheduler_domain.yield_now()?
                }
            }
            // Err(AlienError::EBLOCKING) => {
            //     // check if it is a non-blocking socket
            //     // if file.flags.lock().contains(OpenFlags::O_NONBLOCK) {
            //     //     return Err(LinuxError::EAGAIN);
            //     // }
            //     scheduler_domain.yield_now()?
            // }
            Err(err) => return Err(err),
        }
        // check if there is a EINTR signal
    }
}
pub fn sys_sendto(
    task_domain: &Arc<dyn TaskDomain>,
    vfs_domain: &Arc<dyn VfsDomain>,
    net_stack_domain: &Arc<dyn NetDomain>,
    fd: usize,
    buf: usize,
    len: usize,
    flags: usize,
    addr: usize,
    addr_len: usize,
) -> AlienResult<isize> {
    error!(
        "<sys_sendto> fd: {}, buf: {:#x}, len: {}, flags: {}, addr: {:#x}, addr_len: {:#x}",
        fd, buf, len, flags, addr, addr_len
    );
    let inode_id = task_domain.get_fd(fd)?;
    let socket_id = vfs_domain.socket_id(inode_id)?;
    let mut data = RRefVec::new(0, len);
    task_domain.copy_from_user(buf, data.as_mut_slice())?;

    let remote_addr = if addr != 0 {
        let addr_raw = task_domain.read_val_from_user::<SocketAddrInRaw>(addr)?;
        let socket_addr =
            SocketAddrV4::new(Ipv4Addr::from(addr_raw.addr), addr_raw.in_port.to_be());
        error!("<sys_sendto> remote_addr: {:?}", socket_addr);
        let socket_addr = RRef::new(socket_addr);
        Some(socket_addr)
    } else {
        None
    };
    let remote_addr = remote_addr.as_ref();
    net_stack_domain
        .sendto(socket_id, &data, remote_addr)
        .map(|len| len as isize)
}
pub fn sys_getsockname(
    task_domain: &Arc<dyn TaskDomain>,
    vfs_domain: &Arc<dyn VfsDomain>,
    net_stack_domain: &Arc<dyn NetDomain>,
    fd: usize,
    addr: usize,
    addr_len: usize,
) -> AlienResult<isize> {
    error!(
        "<sys_getsockname> fd: {}, addr: {:#x}, addr_len: {}",
        fd, addr, addr_len
    );
    let inode_id = task_domain.get_fd(fd)?;
    let socket_id = vfs_domain.socket_id(inode_id)?;
    let local_addr = RRef::new(SocketAddrIn::default());
    let local_addr = net_stack_domain.local_addr(socket_id, local_addr)?;
    let raw = SocketAddrInRaw::from(*local_addr);
    task_domain.write_val_to_user(addr, &raw)?;
    let len = core::mem::size_of::<SocketAddrInRaw>();
    task_domain.write_val_to_user(addr_len, &len)?;
    Ok(0)
}

pub fn sys_getpeername(
    task_domain: &Arc<dyn TaskDomain>,
    vfs_domain: &Arc<dyn VfsDomain>,
    net_stack_domain: &Arc<dyn NetDomain>,
    fd: usize,
    addr: usize,
    addr_len: usize,
) -> AlienResult<isize> {
    error!(
        "<sys_getpeername> fd: {}, addr: {:#x}, addr_len: {}",
        fd, addr, addr_len
    );
    let inode_id = task_domain.get_fd(fd)?;
    let socket_id = vfs_domain.socket_id(inode_id)?;
    let remote_addr = RRef::new(SocketAddrIn::default());
    let remote_addr = net_stack_domain.remote_addr(socket_id, remote_addr)?;
    let raw = SocketAddrInRaw::from(*remote_addr);
    task_domain.write_val_to_user(addr, &raw)?;
    let len = core::mem::size_of::<SocketAddrInRaw>();
    task_domain.write_val_to_user(addr_len, &len)?;
    Ok(0)
}

pub fn sys_set_socket_opt(
    task_domain: &Arc<dyn TaskDomain>,
    vfs_domain: &Arc<dyn VfsDomain>,
    _net_stack_domain: &Arc<dyn NetDomain>,
    fd: usize,
    level: usize,
    opt_name: usize,
    opt_value: usize,
    opt_len: usize,
) -> AlienResult<isize> {
    error!(
        "<sys_set_socket_opt> fd: {}, level: {}, opt_name: {}, opt_value: {:#x}, opt_len: {}",
        fd, level, opt_name, opt_value, opt_len
    );
    let inode_id = task_domain.get_fd(fd)?;
    let _socket_id = vfs_domain.socket_id(inode_id)?;
    let level = SocketLevel::try_from(level).map_err(|_| AlienError::EINVAL)?;
    match level {
        SocketLevel::Ip => {}
        SocketLevel::Socket => {
            let opt_name = SocketOption::try_from(opt_name).map_err(|_| AlienError::EINVAL)?;
            error!(
                "<sys_set_socket_opt> level: {:?}, opt_name: {:?}",
                level, opt_name
            );
        }
        SocketLevel::Tcp => {
            let opt_name = TcpSocketOption::try_from(opt_name).map_err(|_| AlienError::EINVAL)?;
            error!(
                "<sys_set_socket_opt> level: {:?}, opt_name: {:?}",
                level, opt_name
            );
        }
    }
    Ok(0)
}

pub fn sys_get_socket_opt(
    task_domain: &Arc<dyn TaskDomain>,
    vfs_domain: &Arc<dyn VfsDomain>,
    _net_stack_domain: &Arc<dyn NetDomain>,
    fd: usize,
    level: usize,
    opt_name: usize,
    opt_value: usize,
    opt_len: usize,
) -> AlienResult<isize> {
    error!(
        "<sys_get_socket_opt> fd: {}, level: {}, opt_name: {}, opt_value: {:#x}, opt_len: {}",
        fd, level, opt_name, opt_value, opt_len
    );
    let inode_id = task_domain.get_fd(fd)?;
    let _socket_id = vfs_domain.socket_id(inode_id)?;
    let level = SocketLevel::try_from(level).map_err(|_| AlienError::EINVAL)?;
    match level {
        SocketLevel::Ip => {}
        SocketLevel::Socket => {
            let opt_name = SocketOption::try_from(opt_name).map_err(|_| AlienError::EINVAL)?;
            error!(
                "<sys_get_socket_opt> level: {:?}, opt_name: {:?}",
                level, opt_name
            );
            match opt_name {
                SocketOption::SO_RCVBUF => {
                    task_domain.write_val_to_user(opt_value, &32000u32)?;
                }
                SocketOption::SO_SNDBUF => {
                    task_domain.write_val_to_user(opt_value, &32000u32)?;
                }
                SocketOption::SO_ERROR => {
                    task_domain.write_val_to_user(opt_value, &0u32)?;
                }
                _ => {}
            }
        }
        SocketLevel::Tcp => {
            let opt_name = TcpSocketOption::try_from(opt_name).map_err(|_| AlienError::EINVAL)?;
            error!(
                "<sys_get_socket_opt> level: {:?}, opt_name: {:?}",
                level, opt_name
            );
            match opt_name {
                TcpSocketOption::TCP_MAXSEG => {
                    task_domain.write_val_to_user(opt_value, &2000u32)?;
                }
                TcpSocketOption::TCP_NODELAY => {
                    task_domain.write_val_to_user(opt_value, &0u32)?;
                }
                _ => {}
            }
        }
    }
    task_domain.write_val_to_user(opt_len, &core::mem::size_of::<u32>())?;
    Ok(0)
}

pub fn sys_shutdown(
    task_domain: &Arc<dyn TaskDomain>,
    vfs_domain: &Arc<dyn VfsDomain>,
    net_stack_domain: &Arc<dyn NetDomain>,
    fd: usize,
    how: usize,
) -> AlienResult<isize> {
    error!("<sys_shutdown> fd: {}, how: {}", fd, how);
    let inode_id = task_domain.get_fd(fd)?;
    let socket_id = vfs_domain.socket_id(inode_id)?;
    let flag = ShutdownFlag::try_from(how).map_err(|_| AlienError::EINVAL)?;
    net_stack_domain.shutdown(socket_id, flag)?;
    Ok(0)
}
