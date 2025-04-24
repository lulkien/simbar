use nix::unistd::{ForkResult, fork};
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::process;
use std::thread;
use std::time::Duration;

const SOCKET_PATH: &str = "/tmp/controller_worker.sock";
const LOG_DIR: &str = "/tmp/myapp";

fn init_controller_logging() -> Result<(), fern::InitError> {
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

fn init_worker_logging() -> Result<(), fern::InitError> {
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

fn main() {
    let _ = fs::remove_file(SOCKET_PATH);

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child: _, .. }) => {
            if let Err(e) = init_controller_logging() {
                eprintln!("Failed to initialize controller logging: {}", e);
                process::exit(1);
            }
            controller();
        }
        Ok(ForkResult::Child) => {
            if let Err(e) = init_worker_logging() {
                eprintln!("Failed to initialize worker logging: {}", e);
                process::exit(1);
            }
            thread::sleep(Duration::from_secs(2));
            worker();
        }
        Err(_) => {
            eprintln!("Failed to fork process");
            process::exit(1);
        }
    }
}

fn controller() {
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
    log::info!("Controller: Shutting down");
}

fn worker() {
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
