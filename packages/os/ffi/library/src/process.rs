use futures::{StreamExt, pin_mut};
use std::{error::Error, process::exit, str, time::Duration};
use tokio::{process::Command, time::sleep};

#[ffi::bindgen]
async fn _pen_os_exit(code: ffi::Number) -> ffi::None {
    // HACK Wait for all I/O buffers to be flushed (hopefully.)
    sleep(Duration::from_millis(50)).await;

    // Resolve a main function immediately with an exit code.
    exit(f64::from(code) as i32)
}

#[ffi::bindgen]
async fn _pen_os_run_command(
    command: ffi::ByteString,
    arguments: ffi::List,
) -> Result<(), Box<dyn Error>> {
    let arguments = ffi::future::stream::from_list(arguments);

    pin_mut!(arguments);

    let mut args = vec![];

    while let Some(argument) = arguments.next().await {
        args.push(ffi::ByteString::try_from(argument).unwrap());
    }

    Command::new(str::from_utf8(command.as_slice())?)
        .args(
            args.iter()
                .map(|string| str::from_utf8(string.as_slice()))
                .collect::<Result<Vec<_>, _>>()?,
        )
        .output()
        .await?;

    Ok(())
}
