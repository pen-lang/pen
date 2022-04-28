use std::{error::Error, str, time::Duration};

#[repr(C)]
struct PoolOptions {
    min_connections: ffi::Number,
    max_connections: ffi::Number,
    connect_timeout: ffi::Number,
}

#[repr(C)]
#[derive(Default)]
struct Pool {
    inner: ffi::Any,
}

#[ffi::any]
#[repr(C)]
#[derive(Clone)]
struct PoolInner {
    pool: sqlx::Pool<sqlx::Any>,
}

#[ffi::bindgen]
async fn _pen_sql_create_pool(
    uri: ffi::ByteString,
    options: ffi::Arc<PoolOptions>,
) -> Result<ffi::Arc<Pool>, Box<dyn Error>> {
    Ok(Pool {
        inner: PoolInner {
            pool: sqlx::any::AnyPoolOptions::new()
                .min_connections(f64::from(options.min_connections) as u32)
                .max_connections(f64::from(options.max_connections) as u32)
                .connect_timeout(Duration::from_millis(
                    f64::from(options.connect_timeout) as u64
                ))
                .connect(str::from_utf8(uri.as_slice())?)
                .await?,
        }
        .into(),
    }
    .into())
}
