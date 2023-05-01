use std::{error::Error, str, sync::Arc};
use tokio::{
    net,
    sync::{RwLock, RwLockWriteGuard},
};

const MAX_UDP_PAYLOAD_SIZE: usize = 512;

#[ffi::into_any(into_fn = "_pen_os_udp_socket_to_any")]
#[repr(C)]
#[derive(Clone)]
pub struct UdpSocket(ffi::Arc<ffi::Any>);

#[ffi::any]
#[derive(Clone, Debug)]
struct UdpSocketInner {
    socket: Arc<RwLock<net::UdpSocket>>,
}

impl UdpSocket {
    pub fn new(socket: net::UdpSocket) -> Self {
        Self(ffi::Arc::new(
            UdpSocketInner {
                socket: RwLock::new(socket).into(),
            }
            .into(),
        ))
    }

    pub async fn lock(&self) -> RwLockWriteGuard<'_, net::UdpSocket> {
        TryInto::<&UdpSocketInner>::try_into(&*self.0)
            .unwrap()
            .socket
            .write()
            .await
    }
}

#[ffi::into_any(into_fn = "_pen_os_udp_datagram_to_any")]
#[repr(C)]
#[derive(Clone, Debug)]
pub struct UdpDatagram(ffi::Arc<UdpDatagramInner>);

#[repr(C)]
#[derive(Clone, Debug)]
struct UdpDatagramInner {
    data: ffi::ByteString,
    address: ffi::ByteString,
}

impl UdpDatagram {
    pub fn new(data: ffi::ByteString, address: ffi::ByteString) -> Self {
        Self(UdpDatagramInner { data, address }.into())
    }

    pub fn data(&self) -> &ffi::ByteString {
        &self.0.data
    }

    pub fn address(&self) -> &ffi::ByteString {
        &self.0.address
    }
}

#[ffi::bindgen]
async fn _pen_os_udp_bind(address: ffi::ByteString) -> Result<UdpSocket, Box<dyn Error>> {
    Ok(UdpSocket::new(
        net::UdpSocket::bind(str::from_utf8(address.as_slice())?).await?,
    ))
}

#[ffi::bindgen]
async fn _pen_os_udp_connect(
    socket: UdpSocket,
    address: ffi::ByteString,
) -> Result<(), Box<dyn Error>> {
    socket
        .lock()
        .await
        .connect(str::from_utf8(address.as_slice())?)
        .await?;

    Ok(())
}

#[ffi::bindgen]
async fn _pen_os_udp_receive(socket: UdpSocket) -> Result<ffi::ByteString, Box<dyn Error>> {
    let mut buffer = vec![0; MAX_UDP_PAYLOAD_SIZE];
    let size = socket.lock().await.recv(&mut buffer).await?;

    buffer.truncate(size);

    Ok(buffer.into())
}

#[ffi::bindgen]
async fn _pen_os_udp_receive_from(socket: UdpSocket) -> Result<UdpDatagram, Box<dyn Error>> {
    let mut buffer = vec![0; MAX_UDP_PAYLOAD_SIZE];
    let (size, address) = socket.lock().await.recv_from(&mut buffer).await?;

    buffer.truncate(size);

    Ok(UdpDatagram::new(buffer.into(), address.to_string().into()))
}

#[ffi::bindgen]
async fn _pen_os_udp_send(
    socket: UdpSocket,
    data: ffi::ByteString,
) -> Result<ffi::Number, Box<dyn Error>> {
    let size = socket.lock().await.send(data.as_slice()).await?;

    Ok((size as f64).into())
}

#[ffi::bindgen]
async fn _pen_os_udp_send_to(
    socket: UdpSocket,
    data: ffi::ByteString,
    address: ffi::ByteString,
) -> Result<ffi::Number, Box<dyn Error>> {
    let size = socket
        .lock()
        .await
        .send_to(data.as_slice(), str::from_utf8(address.as_slice())?)
        .await?;

    Ok((size as f64).into())
}
