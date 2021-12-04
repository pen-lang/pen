use crate::{error::OsError, result::FfiResult};
use std::{
    net,
    sync::{Arc, LockResult, RwLock, RwLockWriteGuard},
};

const MAX_UDP_PAYLOAD_SIZE: usize = 512;

#[derive(Clone, Default)]
pub struct UdpSocket {
    inner: ffi::Any,
}

impl UdpSocket {
    pub fn new(socket: net::UdpSocket) -> ffi::Arc<Self> {
        Self {
            inner: UdpSocketInner::new(socket).into(),
        }
        .into()
    }

    pub fn lock(&self) -> Result<RwLockWriteGuard<net::UdpSocket>, OsError> {
        Ok(TryInto::<&UdpSocketInner>::try_into(&self.inner)
            .unwrap()
            .get_mut()?)
    }
}

#[derive(Clone, Debug)]
pub struct UdpSocketInner {
    socket: Arc<RwLock<net::UdpSocket>>,
}

ffi::type_information!(udp_socket_inner, crate::udp::UdpSocketInner);

impl UdpSocketInner {
    pub fn new(socket: net::UdpSocket) -> Self {
        Self {
            socket: RwLock::new(socket).into(),
        }
    }

    pub fn get_mut(&self) -> LockResult<RwLockWriteGuard<'_, net::UdpSocket>> {
        self.socket.write()
    }
}

#[derive(Clone, Debug, Default)]
pub struct UdpDatagram {
    pub data: ffi::ByteString,
    pub address: ffi::ByteString,
}

#[no_mangle]
extern "C" fn _pen_os_udp_bind(
    address: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Arc<UdpSocket>>> {
    todo!()
}

#[no_mangle]
extern "C" fn _pen_os_udp_connect(
    socket: ffi::Arc<UdpSocket>,
    address: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::None>> {
    todo!()
}

#[no_mangle]
extern "C" fn _pen_os_udp_receive(
    socket: ffi::Arc<UdpSocket>,
) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    todo!()
}

#[no_mangle]
extern "C" fn _pen_os_udp_receive_from(
    socket: ffi::Arc<UdpSocket>,
) -> ffi::Arc<FfiResult<ffi::Arc<UdpDatagram>>> {
    todo!()
}

#[no_mangle]
extern "C" fn _pen_os_udp_send(
    socket: ffi::Arc<UdpSocket>,
    data: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Number>> {
    todo!()
}

#[no_mangle]
extern "C" fn _pen_os_udp_send_to(
    socket: ffi::Arc<UdpSocket>,
    data: ffi::ByteString,
    address: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Number>> {
    todo!()
}
