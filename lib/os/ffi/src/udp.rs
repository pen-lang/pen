use crate::{error::OsError, result::FfiResult};
use ffi::AnyLike;
use std::{
    net, str,
    sync::{Arc, LockResult, RwLock, RwLockWriteGuard},
};

#[derive(Clone, Default)]
pub struct UdpSocket {
    inner: ffi::Any,
}

impl UdpSocket {
    pub fn new(socket: net::UdpSocket) -> ffi::Arc<Self> {
        Self {
            inner: UdpSocketInner::new(socket).into_any(),
        }
        .into()
    }

    pub fn lock(&self) -> Result<RwLockWriteGuard<net::UdpSocket>, OsError> {
        Ok(UdpSocketInner::as_inner(&self.inner).unwrap().get_mut()?)
    }
}

#[derive(Clone, Debug)]
pub struct UdpSocketInner {
    file: Arc<RwLock<net::UdpSocket>>,
}

ffi::type_information!(ffi_file, crate::udp::UdpSocketInner);

impl UdpSocketInner {
    pub fn new(socket: net::UdpSocket) -> Self {
        Self {
            file: RwLock::new(socket).into(),
        }
    }

    pub fn get_mut(&self) -> LockResult<RwLockWriteGuard<'_, net::UdpSocket>> {
        self.file.write()
    }
}

#[no_mangle]
extern "C" fn _pen_os_udp_bind(
    address: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Arc<UdpSocket>>> {
    ffi::Arc::new(bind(address).into())
}

fn bind(address: ffi::ByteString) -> Result<ffi::Arc<UdpSocket>, OsError> {
    Ok(UdpSocket::new(net::UdpSocket::bind(str::from_utf8(address.as_slice())?)?).into())
}

#[no_mangle]
extern "C" fn _pen_os_udp_connect(
    socket: ffi::Arc<UdpSocket>,
    address: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::None>> {
    ffi::Arc::new(connect(socket, address).into())
}

fn connect(socket: ffi::Arc<UdpSocket>, address: ffi::ByteString) -> Result<(), OsError> {
    socket
        .lock()?
        .connect(str::from_utf8(address.as_slice())?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_socket() {
        bind("127.0.0.1:8080".into()).unwrap();
    }

    #[test]
    fn connect_socket() {
        let socket = bind("127.0.0.1:8080".into()).unwrap();
        connect(socket, "127.0.0.1:8081".into()).unwrap();
    }
}
