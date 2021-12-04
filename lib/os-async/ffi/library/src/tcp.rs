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

#[derive(Clone, Debug)]
pub struct TcpListenerInner {
    listener: Arc<RwLock<net::TcpListener>>,
}

ffi::type_information!(tcp_listener_inner, crate::tcp::TcpListenerInner);

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

#[derive(Clone, Debug)]
pub struct TcpStreamInner {
    socket: Arc<RwLock<net::TcpStream>>,
}

ffi::type_information!(ffi_tcp_socket, crate::tcp::TcpStreamInner);

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

#[no_mangle]
extern "C" fn _pen_os_tcp_bind(
    address: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Arc<TcpListener>>> {
    todo!()
}

#[no_mangle]
extern "C" fn _pen_os_tcp_connect(
    address: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Arc<TcpStream>>> {
    todo!()
}

#[no_mangle]
extern "C" fn _pen_os_tcp_accept(
    listener: ffi::Arc<TcpListener>,
) -> ffi::Arc<FfiResult<ffi::Arc<TcpAcceptedStream>>> {
    todo!()
}

#[no_mangle]
extern "C" fn _pen_os_tcp_receive(
    socket: ffi::Arc<TcpStream>,
    limit: ffi::Number,
) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    todo!()
}

#[no_mangle]
extern "C" fn _pen_os_tcp_send(
    socket: ffi::Arc<TcpStream>,
    data: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Number>> {
    todo!()
}
