use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::process;
use std::thread;
use std::time::Duration;

use crate::common::{LOG_DIR, SOCKET_PATH};

pub fn worker() {
    let pid = process::id();
    log::info!("Worker[{}]: Starting...", pid);

    thread::sleep(Duration::from_millis(500));

    let mut socket = match UnixStream::connect(SOCKET_PATH) {
        Ok(sock) => {
            log::info!("Worker[{}]: Connected to controller", pid);
            sock
        }
        Err(e) => {
            log::error!("Worker[{}]: Failed to connect to socket: {}", pid, e);
            process::exit(1);
        }
    };

    let mut buffer = vec![0; 1024];
    loop {
        match socket.read(&mut buffer) {
            Ok(0) => {
                log::info!("Worker[{}]: Connection closed by controller", pid);
                break;
            }
            Ok(n) => {
                let message = String::from_utf8_lossy(&buffer[..n]);
                log::info!("Worker[{}]: Received: {}", pid, message);

                if message == "exit" {
                    log::info!("Worker[{}]: Received exit command", pid);
                    break;
                }

                let response = format!("Worker[{}] processed: {}", pid, message);
                match socket.write_all(response.as_bytes()) {
                    Ok(_) => log::info!("Worker[{}]: Sent response", pid),
                    Err(e) => log::error!("Worker[{}]: Failed to send response: {}", pid, e),
                }
            }
            Err(e) => {
                log::error!("Worker[{}]: Failed to read from socket: {}", pid, e);
                break;
            }
        }
    }

    log::info!("Worker[{}]: Shutting down", pid);
}

pub fn init_worker_logging() -> Result<(), fern::InitError> {
    fs::create_dir_all(LOG_DIR)?;

    let pid = process::id();
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(fern::log_file(format!("{}/worker_{}.log", LOG_DIR, pid))?)
        .apply()?;
    Ok(())
}
