use crate::error::OsError;
use std::{str, sync::Arc};
use tokio::{
    net,
    sync::{RwLock, RwLockWriteGuard},
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

    pub async fn lock(&self) -> RwLockWriteGuard<'_, net::UdpSocket> {
        TryInto::<&UdpSocketInner>::try_into(&self.inner)
            .unwrap()
            .get_mut()
            .await
    }
}

#[ffi::any]
#[derive(Clone, Debug)]
pub struct UdpSocketInner {
    socket: Arc<RwLock<net::UdpSocket>>,
}

impl UdpSocketInner {
    pub fn new(socket: net::UdpSocket) -> Self {
        Self {
            socket: RwLock::new(socket).into(),
        }
    }

    pub async fn get_mut(&self) -> RwLockWriteGuard<'_, net::UdpSocket> {
        self.socket.write().await
    }
}

#[derive(Clone, Debug, Default)]
pub struct UdpDatagram {
    pub data: ffi::ByteString,
    pub address: ffi::ByteString,
}

#[ffi::bindgen]
async fn _pen_os_udp_bind(address: ffi::ByteString) -> Result<ffi::Arc<UdpSocket>, OsError> {
    Ok(UdpSocket::new(
        net::UdpSocket::bind(str::from_utf8(address.as_slice())?).await?,
    ))
}

#[ffi::bindgen]
async fn _pen_os_udp_connect(
    socket: ffi::Arc<UdpSocket>,
    address: ffi::ByteString,
) -> Result<(), OsError> {
    socket
        .lock()
        .await
        .connect(str::from_utf8(address.as_slice())?)
        .await?;

    Ok(())
}

#[ffi::bindgen]
async fn _pen_os_udp_receive(socket: ffi::Arc<UdpSocket>) -> Result<ffi::ByteString, OsError> {
    let mut buffer = vec![0; MAX_UDP_PAYLOAD_SIZE];
    let size = socket.lock().await.recv(&mut buffer).await?;

    buffer.truncate(size);

    Ok(buffer.into())
}

#[ffi::bindgen]
async fn _pen_os_udp_receive_from(
    socket: ffi::Arc<UdpSocket>,
) -> Result<ffi::Arc<UdpDatagram>, OsError> {
    let mut buffer = vec![0; MAX_UDP_PAYLOAD_SIZE];
    let (size, address) = socket.lock().await.recv_from(&mut buffer).await?;

    buffer.truncate(size);

    Ok(UdpDatagram {
        data: buffer.into(),
        address: address.to_string().into(),
    }
    .into())
}

#[ffi::bindgen]
async fn _pen_os_udp_send(
    socket: ffi::Arc<UdpSocket>,
    data: ffi::ByteString,
) -> Result<ffi::Number, OsError> {
    let size = socket.lock().await.send(data.as_slice()).await?;

    Ok((size as f64).into())
}

#[ffi::bindgen]
async fn _pen_os_udp_send_to(
    socket: ffi::Arc<UdpSocket>,
    data: ffi::ByteString,
    address: ffi::ByteString,
) -> Result<ffi::Number, OsError> {
    let size = socket
        .lock()
        .await
        .send_to(data.as_slice(), str::from_utf8(address.as_slice())?)
        .await?;

    Ok((size as f64).into())
}
