use basic::AlienError;
use lose_net_stack::results::NetServerError;

pub fn to_alien_error(net_server_error: NetServerError) -> AlienError {
    match net_server_error {
        NetServerError::Unsupported => AlienError::ENOSYS,
        NetServerError::EmptyClient => AlienError::ECONNRESET,
        NetServerError::EmptyData => AlienError::EBLOCKING,
        NetServerError::NoUdpRemoteAddress => AlienError::EINVAL,
        NetServerError::ServerNotExists => AlienError::ECONNREFUSED,
        NetServerError::PortWasUsed => AlienError::EADDRINUSE,
        NetServerError::Blocking => AlienError::EBLOCKING,
    }
}
