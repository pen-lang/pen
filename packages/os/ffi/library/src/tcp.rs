use std::{error::Error, str, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net,
    sync::{RwLock, RwLockWriteGuard},
};

#[ffi::into_any(into_fn = "_pen_os_tcp_listener_to_any")]
#[repr(C)]
#[derive(Clone)]
struct TcpListener(ffi::Arc<ffi::Any>);

#[ffi::any]
#[derive(Clone, Debug)]
struct TcpListenerInner(Arc<RwLock<net::TcpListener>>);

impl TcpListener {
    pub fn new(listener: net::TcpListener) -> Self {
        Self(ffi::Arc::new(
            TcpListenerInner(RwLock::new(listener).into()).into(),
        ))
    }

    pub async fn lock(&self) -> RwLockWriteGuard<'_, net::TcpListener> {
        TryInto::<&TcpListenerInner>::try_into(&*self.0)
            .unwrap()
            .0
            .write()
            .await
    }
}

#[ffi::into_any(into_fn = "_pen_os_tcp_stream_to_any")]
#[repr(C)]
#[derive(Clone)]
struct TcpStream(ffi::Arc<ffi::Any>);

#[ffi::any]
#[derive(Clone, Debug)]
struct TcpStreamInner(Arc<RwLock<net::TcpStream>>);

impl TcpStream {
    pub fn new(socket: net::TcpStream) -> Self {
        Self(ffi::Arc::new(
            TcpStreamInner(RwLock::new(socket).into()).into(),
        ))
    }

    pub async fn lock(&self) -> RwLockWriteGuard<'_, net::TcpStream> {
        TryInto::<&TcpStreamInner>::try_into(&*self.0)
            .unwrap()
            .0
            .write()
            .await
    }
}

#[ffi::into_any(into_fn = "_pen_os_tcp_accepted_stream_to_any")]
#[repr(C)]
#[derive(Clone)]
struct TcpAcceptedStream(ffi::Arc<TcpAcceptedStreamInner>);

#[repr(C)]
struct TcpAcceptedStreamInner {
    stream: TcpStream,
    address: ffi::ByteString,
}

impl TcpAcceptedStream {
    pub fn new(stream: TcpStream, address: ffi::ByteString) -> Self {
        Self(ffi::Arc::new(TcpAcceptedStreamInner { stream, address }))
    }
}

#[ffi::bindgen]
async fn _pen_os_tcp_bind(address: ffi::ByteString) -> Result<TcpListener, Box<dyn Error>> {
    Ok(TcpListener::new(
        net::TcpListener::bind(str::from_utf8(address.as_slice())?).await?,
    ))
}

#[ffi::bindgen]
async fn _pen_os_tcp_connect(address: ffi::ByteString) -> Result<TcpStream, Box<dyn Error>> {
    Ok(TcpStream::new(
        net::TcpStream::connect(str::from_utf8(address.as_slice())?).await?,
    ))
}

#[ffi::bindgen]
async fn _pen_os_tcp_accept(listener: TcpListener) -> Result<TcpAcceptedStream, Box<dyn Error>> {
    let (stream, address) = listener.lock().await.accept().await?;

    Ok(TcpAcceptedStream::new(
        TcpStream::new(stream),
        address.to_string().into(),
    ))
}

#[ffi::bindgen]
async fn _pen_os_tcp_receive(
    socket: TcpStream,
    limit: ffi::Number,
) -> Result<ffi::ByteString, Box<dyn Error>> {
    let mut buffer = vec![0; f64::from(limit) as usize];
    let size = socket.lock().await.read(&mut buffer).await?;

    buffer.truncate(size);

    Ok(buffer.into())
}

#[ffi::bindgen]
async fn _pen_os_tcp_send(
    socket: TcpStream,
    data: ffi::ByteString,
) -> Result<ffi::Number, Box<dyn Error>> {
    Ok((socket.lock().await.write(data.as_slice()).await? as f64).into())
}
