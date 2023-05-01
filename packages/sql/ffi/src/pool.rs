use crate::error::SqlError;
use futures::{pin_mut, StreamExt};
use sqlx::{Executor, Row, ValueRef};
use std::{error::Error, str, time::Duration};

type AnyPool = sqlx::Pool<sqlx::Any>;

type AnyQuery<'a> =
    sqlx::query::Query<'a, sqlx::Any, <sqlx::Any as sqlx::database::HasArguments<'a>>::Arguments>;

#[repr(C)]
struct PoolOptions {
    min_connections: ffi::Number,
    max_connections: ffi::Number,
    connect_timeout: ffi::Number,
}

#[ffi::into_any(into_fn = "_pen_sql_pool_to_any")]
#[repr(C)]
struct Pool(ffi::Arc<ffi::Any>);

impl Pool {
    pub fn new(pool: AnyPool) -> Self {
        Self(ffi::Arc::new(PoolInner { pool }.into()))
    }

    pub fn as_inner(&self) -> &AnyPool {
        let pool: &PoolInner = TryFrom::try_from(&*self.0).unwrap();

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
) -> Result<Pool, Box<dyn Error>> {
    Ok(Pool::new(
        sqlx::any::AnyPoolOptions::new()
            .min_connections(f64::from(options.min_connections) as u32)
            .max_connections(f64::from(options.max_connections) as u32)
            .acquire_timeout(Duration::from_millis(
                f64::from(options.connect_timeout) as u64
            ))
            .connect(str::from_utf8(uri.as_slice())?)
            .await?,
    ))
}

#[ffi::bindgen]
async fn _pen_sql_pool_query(
    pool: Pool,
    query: ffi::ByteString,
    arguments: ffi::List,
) -> Result<ffi::List, Box<dyn Error>> {
    let mut rows = ffi::List::new();

    for row in pool
        .as_inner()
        .fetch_all(build_query(&query, arguments).await?)
        .await?
        .iter()
        .rev()
    {
        let mut columns = ffi::List::new();

        for index in (0..row.columns().len()).rev() {
            columns = ffi::List::prepend(
                columns,
                if row.try_get_raw(index)?.is_null() {
                    ffi::None::default().into()
                } else if let Ok(number) = row.try_get::<f64, _>(index) {
                    ffi::Number::from(number).into()
                } else if let Ok(number) = row.try_get::<f32, _>(index) {
                    ffi::Number::from(number as f64).into()
                } else if let Ok(number) = row.try_get::<i64, _>(index) {
                    ffi::Number::from(number as f64).into()
                } else if let Ok(number) = row.try_get::<i32, _>(index) {
                    ffi::Number::from(number as f64).into()
                } else if let Ok(boolean) = row.try_get::<bool, _>(index) {
                    ffi::Boolean::from(boolean).into()
                } else if let Ok(string) = row.try_get::<&str, _>(index) {
                    ffi::Any::from(ffi::ByteString::from(string))
                } else {
                    return Err(SqlError::TypeNotSupported.into());
                },
            );
        }

        rows = ffi::List::prepend(rows, columns);
    }

    Ok(rows)
}

#[ffi::bindgen]
async fn _pen_sql_pool_execute(
    pool: Pool,
    query: ffi::ByteString,
    arguments: ffi::List,
) -> Result<ffi::Number, Box<dyn Error>> {
    Ok((pool
        .as_inner()
        .execute(build_query(&query, arguments).await?)
        .await?
        .rows_affected() as f64)
        .into())
}

async fn build_query(
    query: &ffi::ByteString,
    arguments: ffi::List,
) -> Result<AnyQuery<'_>, Box<dyn Error>> {
    let mut query = sqlx::query::<sqlx::Any>(str::from_utf8(query.as_slice())?);
    let arguments = ffi::future::stream::from_list(arguments);

    pin_mut!(arguments);

    while let Some(argument) = arguments.next().await {
        if argument.is_none() {
            query = query.bind(None::<f64>);
        } else if let Ok(boolean) = ffi::Boolean::try_from(argument.clone()) {
            query = query.bind(bool::from(boolean));
        } else if let Ok(number) = ffi::Number::try_from(argument.clone()) {
            query = query.bind(f64::from(number));
        } else if let Ok(string) = ffi::ByteString::try_from(argument.clone()) {
            query = query.bind(str::from_utf8(string.as_slice())?.to_owned());
        } else {
            return Err(SqlError::TypeNotSupported.into());
        }
    }

    Ok(query)
}
