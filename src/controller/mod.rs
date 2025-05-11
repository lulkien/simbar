use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::process;
use std::thread;
use std::time::Duration;

use crate::common::{LOG_DIR, SOCKET_PATH};

pub fn controller() {
    log::info!("Controller: Starting...");

    let listener = match UnixListener::bind(SOCKET_PATH) {
        Ok(sock) => {
            log::info!("Controller: Socket created at {}", SOCKET_PATH);
            sock
        }
        Err(e) => {
            log::error!("Controller: Failed to bind socket: {}", e);
            process::exit(1);
        }
    };

    log::info!("Controller: Waiting for worker to connect...");

    match listener.accept() {
        Ok((mut socket, _addr)) => {
            log::info!("Controller: Worker connected");

            for i in 1..=5 {
                let message = format!("Message {} from controller", i);
                match socket.write_all(message.as_bytes()) {
                    Ok(_) => log::info!("Controller: Sent: {}", message),
                    Err(e) => log::error!("Controller: Failed to send message: {}", e),
                }

                // Wait for response
                let mut response = vec![0; 1024];
                match socket.read(&mut response) {
                    Ok(n) => {
                        let response_str = String::from_utf8_lossy(&response[..n]);
                        log::info!("Controller: Received: {}", response_str);
                    }
                    Err(e) => log::error!("Controller: Failed to read response: {}", e),
                }

                thread::sleep(Duration::from_secs(1));
            }

            let _ = socket.write_all(b"exit");
            log::info!("Controller: Sent exit command to worker");
        }
        Err(e) => {
            log::error!("Controller: Failed to accept connection: {}", e);
            process::exit(1);
        }
    }

    let _ = fs::remove_file(SOCKET_PATH);
    log::info!("Controller: Shutting down...");
}

pub fn init_controller_logging() -> Result<(), fern::InitError> {
    fs::create_dir_all(LOG_DIR)?;

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
        .chain(fern::log_file(format!("{}/controller.log", LOG_DIR))?)
        .apply()?;
    Ok(())
}
