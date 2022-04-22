use crate::{error::OsError, result::FfiResult};
use std::{
    io::{Read, Write},
    net, str,
    sync::{Arc, LockResult, RwLock, RwLockWriteGuard},
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

    pub fn lock(&self) -> Result<RwLockWriteGuard<net::TcpListener>, OsError> {
        Ok(TryInto::<&TcpListenerInner>::try_into(&self.inner)
            .unwrap()
            .get_mut()?)
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

    pub fn get_mut(&self) -> LockResult<RwLockWriteGuard<'_, net::TcpListener>> {
        self.listener.write()
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

    pub fn lock(&self) -> Result<RwLockWriteGuard<net::TcpStream>, OsError> {
        Ok(TryInto::<&TcpStreamInner>::try_into(&self.inner)
            .unwrap()
            .get_mut()?)
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

    pub fn get_mut(&self) -> LockResult<RwLockWriteGuard<'_, net::TcpStream>> {
        self.socket.write()
    }
}

#[derive(Clone, Default)]
pub struct TcpAcceptedStream {
    pub stream: ffi::Arc<TcpStream>,
    pub address: ffi::ByteString,
}

#[ffi::bindgen]
fn _pen_os_tcp_bind(address: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Arc<TcpListener>>> {
    ffi::Arc::new(bind(address).into())
}

fn bind(address: ffi::ByteString) -> Result<ffi::Arc<TcpListener>, OsError> {
    Ok(TcpListener::new(net::TcpListener::bind(str::from_utf8(
        address.as_slice(),
    )?)?))
}

#[ffi::bindgen]
fn _pen_os_tcp_connect(address: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Arc<TcpStream>>> {
    ffi::Arc::new(connect(address).into())
}

fn connect(address: ffi::ByteString) -> Result<ffi::Arc<TcpStream>, OsError> {
    Ok(TcpStream::new(net::TcpStream::connect(str::from_utf8(
        address.as_slice(),
    )?)?))
}

#[ffi::bindgen]
fn _pen_os_tcp_accept(
    listener: ffi::Arc<TcpListener>,
) -> ffi::Arc<FfiResult<ffi::Arc<TcpAcceptedStream>>> {
    ffi::Arc::new(accept(listener).into())
}

fn accept(listener: ffi::Arc<TcpListener>) -> Result<ffi::Arc<TcpAcceptedStream>, OsError> {
    let (stream, address) = listener.lock()?.accept()?;

    Ok(TcpAcceptedStream {
        stream: TcpStream::new(stream),
        address: address.to_string().into(),
    }
    .into())
}

#[ffi::bindgen]
fn _pen_os_tcp_receive(
    socket: ffi::Arc<TcpStream>,
    limit: ffi::Number,
) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    ffi::Arc::new(receive(socket, limit).into())
}

fn receive(socket: ffi::Arc<TcpStream>, limit: ffi::Number) -> Result<ffi::ByteString, OsError> {
    let mut buffer = vec![0; f64::from(limit) as usize];
    let size = socket.lock()?.read(&mut buffer)?;

    buffer.truncate(size);

    Ok(buffer.into())
}

#[ffi::bindgen]
fn _pen_os_tcp_send(
    socket: ffi::Arc<TcpStream>,
    data: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Number>> {
    ffi::Arc::new(send(socket, data).into())
}

fn send(socket: ffi::Arc<TcpStream>, data: ffi::ByteString) -> Result<ffi::Number, OsError> {
    Ok((socket.lock()?.write(data.as_slice())? as f64).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_socket() {
        bind("127.0.0.1:8082".into()).unwrap();
    }
}
