use crate::error::SqlError;
use futures::{pin_mut, StreamExt};
use sqlx::{Column, Executor, Row, ValueRef};
use std::{error::Error, str, time::Duration};

type AnyPool = sqlx::Pool<sqlx::Any>;

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

impl Pool {
    pub fn new(pool: AnyPool) -> Self {
        Self {
            inner: PoolInner { pool }.into(),
        }
    }

    pub fn as_inner(&self) -> &AnyPool {
        let pool: &PoolInner = TryFrom::try_from(&self.inner).unwrap();

        &pool.pool
    }
}

#[ffi::any]
#[repr(C)]
#[derive(Clone)]
struct PoolInner {
    pool: AnyPool,
}

#[ffi::bindgen]
async fn _pen_sql_pool_create(
    uri: ffi::ByteString,
    options: ffi::Arc<PoolOptions>,
) -> Result<ffi::Arc<Pool>, Box<dyn Error>> {
    Ok(Pool::new(
        sqlx::any::AnyPoolOptions::new()
            .min_connections(f64::from(options.min_connections) as u32)
            .max_connections(f64::from(options.max_connections) as u32)
            .connect_timeout(Duration::from_millis(
                f64::from(options.connect_timeout) as u64
            ))
            .connect(str::from_utf8(uri.as_slice())?)
            .await?,
    )
    .into())
}

#[ffi::bindgen]
async fn _pen_sql_pool_query(
    pool: ffi::Arc<Pool>,
    query: ffi::ByteString,
    arguments: ffi::Arc<ffi::List>,
) -> Result<ffi::Arc<ffi::List>, Box<dyn Error>> {
    let mut query = sqlx::query::<sqlx::Any>(str::from_utf8(query.as_slice())?);
    let arguments = ffi::future::stream::from_list(arguments);

    pin_mut!(arguments);

    while let Some(argument) = arguments.next().await {
        if let Ok(string) = ffi::ByteString::try_from(argument.clone()) {
            query = query.bind(str::from_utf8(string.as_slice())?.to_owned());
        } else if let Ok(number) = ffi::Number::try_from(argument.clone()) {
            query = query.bind(f64::from(number));
        } else if argument.is_none() {
            query = query.bind(None::<f64>);
        } else {
            return Err(SqlError::TypeNotSupported.into());
        }
    }

    let mut rows = ffi::List::new();

    for row in pool.as_inner().fetch_all(query).await? {
        let mut columns = ffi::List::new();

        for column in row.columns() {
            columns = ffi::List::prepend(
                columns,
                if let Ok(string) = row.try_get::<String, _>(column.name()) {
                    ffi::Any::from(ffi::ByteString::from(string))
                } else if let Ok(number) = row.try_get::<f64, _>(column.name()) {
                    ffi::Number::from(number).into()
                } else if row.try_get_raw(column.name())?.is_null() {
                    ffi::None::default().into()
                } else {
                    return Err(SqlError::TypeNotSupported.into());
                },
            );
        }

        rows = ffi::List::prepend(rows, columns);
    }

    Ok(rows)
}
