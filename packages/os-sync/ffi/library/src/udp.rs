use crate::error::OsError;
use std::{
    error::Error,
    net, str,
    sync::{Arc, RwLock, RwLockWriteGuard},
};

const MAX_UDP_PAYLOAD_SIZE: usize = 512;

#[ffi::into_any(into_fn = "_pen_os_udp_socket_to_any")]
#[repr(C)]
#[derive(Clone)]
struct UdpSocket(ffi::Arc<ffi::Any>);

#[ffi::any]
#[derive(Clone, Debug)]
struct UdpSocketInner(Arc<RwLock<net::UdpSocket>>);

impl UdpSocket {
    pub fn new(socket: net::UdpSocket) -> Self {
        Self(ffi::Arc::new(
            UdpSocketInner(RwLock::new(socket).into()).into(),
        ))
    }

    pub fn lock(&self) -> Result<RwLockWriteGuard<net::UdpSocket>, OsError> {
        Ok(TryInto::<&UdpSocketInner>::try_into(&*self.0)
            .unwrap()
            .0
            .write()?)
    }
}

#[ffi::into_any(into_fn = "_pen_os_udp_datagram_to_any")]
#[repr(C)]
#[derive(Clone)]
struct UdpDatagram(ffi::Arc<UdpDatagramInner>);

#[repr(C)]
#[derive(Clone)]
struct UdpDatagramInner {
    data: ffi::ByteString,
    address: ffi::ByteString,
}

impl UdpDatagram {
    pub fn new(data: ffi::ByteString, address: ffi::ByteString) -> Self {
        Self(ffi::Arc::new(UdpDatagramInner { data, address }))
    }
}

#[ffi::bindgen]
fn _pen_os_udp_bind(address: ffi::ByteString) -> Result<UdpSocket, Box<dyn Error>> {
    Ok(UdpSocket::new(net::UdpSocket::bind(str::from_utf8(
        address.as_slice(),
    )?)?))
}

#[ffi::bindgen]
fn _pen_os_udp_connect(socket: UdpSocket, address: ffi::ByteString) -> Result<(), Box<dyn Error>> {
    socket
        .lock()?
        .connect(str::from_utf8(address.as_slice())?)?;

    Ok(())
}

#[ffi::bindgen]
fn _pen_os_udp_receive(socket: UdpSocket) -> Result<ffi::ByteString, Box<dyn Error>> {
    let mut buffer = vec![0; MAX_UDP_PAYLOAD_SIZE];
    let size = socket.lock()?.recv(&mut buffer)?;

    buffer.truncate(size);

    Ok(buffer.into())
}

#[ffi::bindgen]
fn _pen_os_udp_receive_from(socket: UdpSocket) -> Result<UdpDatagram, Box<dyn Error>> {
    let mut buffer = vec![0; MAX_UDP_PAYLOAD_SIZE];
    let (size, address) = socket.lock()?.recv_from(&mut buffer)?;

    buffer.truncate(size);

    Ok(UdpDatagram::new(buffer.into(), address.to_string().into()))
}

#[ffi::bindgen]
fn _pen_os_udp_send(
    socket: UdpSocket,
    data: ffi::ByteString,
) -> Result<ffi::Number, Box<dyn Error>> {
    let size = socket.lock()?.send(data.as_slice())?;

    Ok((size as f64).into())
}

#[ffi::bindgen]
fn _pen_os_udp_send_to(
    socket: UdpSocket,
    data: ffi::ByteString,
    address: ffi::ByteString,
) -> Result<ffi::Number, Box<dyn Error>> {
    let size = socket
        .lock()?
        .send_to(data.as_slice(), str::from_utf8(address.as_slice())?)?;

    Ok((size as f64).into())
}
