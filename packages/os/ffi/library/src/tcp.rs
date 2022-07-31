use std::{error::Error, str, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net,
    sync::{RwLock, RwLockWriteGuard},
};

#[derive(Clone, Default)]
pub struct TcpListener {
    inner: ffi::Any,
}

impl TcpListener {
    pub fn new(listener: net::TcpListener) -> ffi::Arc<Self> {
        Self {
            inner: TcpListenerInner::new(listener).into(),
        }
        .into()
    }

    pub async fn lock(&self) -> RwLockWriteGuard<'_, net::TcpListener> {
        TryInto::<&TcpListenerInner>::try_into(&self.inner)
            .unwrap()
            .get_mut()
            .await
    }
}

#[ffi::any]
#[derive(Clone, Debug)]
pub struct TcpListenerInner {
    listener: Arc<RwLock<net::TcpListener>>,
}

impl TcpListenerInner {
    pub fn new(listener: net::TcpListener) -> Self {
        Self {
            listener: RwLock::new(listener).into(),
        }
    }

    pub async fn get_mut(&self) -> RwLockWriteGuard<'_, net::TcpListener> {
        self.listener.write().await
    }
}

#[derive(Clone, Default)]
pub struct TcpStream {
    inner: ffi::Any,
}

impl TcpStream {
    pub fn new(socket: net::TcpStream) -> ffi::Arc<Self> {
        Self {
            inner: TcpStreamInner::new(socket).into(),
        }
        .into()
    }

    pub async fn lock(&self) -> RwLockWriteGuard<'_, net::TcpStream> {
        TryInto::<&TcpStreamInner>::try_into(&self.inner)
            .unwrap()
            .get_mut()
            .await
    }
}

#[ffi::any]
#[derive(Clone, Debug)]
pub struct TcpStreamInner {
    socket: Arc<RwLock<net::TcpStream>>,
}

impl TcpStreamInner {
    pub fn new(socket: net::TcpStream) -> Self {
        Self {
            socket: RwLock::new(socket).into(),
        }
    }

    pub async fn get_mut(&self) -> RwLockWriteGuard<'_, net::TcpStream> {
        self.socket.write().await
    }
}

#[derive(Clone, Default)]
pub struct TcpAcceptedStream {
    pub stream: ffi::Arc<TcpStream>,
    pub address: ffi::ByteString,
}

#[ffi::bindgen]
async fn _pen_os_tcp_bind(
    address: ffi::ByteString,
) -> Result<ffi::Arc<TcpListener>, Box<dyn Error>> {
    Ok(TcpListener::new(
        net::TcpListener::bind(str::from_utf8(address.as_slice())?).await?,
    ))
}

#[ffi::bindgen]
async fn _pen_os_tcp_connect(
    address: ffi::ByteString,
) -> Result<ffi::Arc<TcpStream>, Box<dyn Error>> {
    Ok(TcpStream::new(
        net::TcpStream::connect(str::from_utf8(address.as_slice())?).await?,
    ))
}

#[ffi::bindgen]
async fn _pen_os_tcp_accept(
    listener: ffi::Arc<TcpListener>,
) -> Result<ffi::Arc<TcpAcceptedStream>, Box<dyn Error>> {
    let (stream, address) = listener.lock().await.accept().await?;

    Ok(TcpAcceptedStream {
        stream: TcpStream::new(stream),
        address: address.to_string().into(),
    }
    .into())
}

#[ffi::bindgen]
async fn _pen_os_tcp_receive(
    socket: ffi::Arc<TcpStream>,
    limit: ffi::Number,
) -> Result<ffi::ByteString, Box<dyn Error>> {
    let mut buffer = vec![0; f64::from(limit) as usize];
    let size = socket.lock().await.read(&mut buffer).await?;

    buffer.truncate(size);

    Ok(buffer.into())
}

#[ffi::bindgen]
async fn _pen_os_tcp_send(
    socket: ffi::Arc<TcpStream>,
    data: ffi::ByteString,
) -> Result<ffi::Number, Box<dyn Error>> {
    Ok((socket.lock().await.write(data.as_slice()).await? as f64).into())
}
