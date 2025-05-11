use common::SOCKET_PATH;
use nix::unistd::{ForkResult, fork};
use std::fs;
use std::process;

mod common;
mod controller;
mod ipc;
mod worker;

fn main() {
    let _ = fs::remove_file(SOCKET_PATH);

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child: _ }) => {
            if let Err(e) = controller::init_controller_logging() {
                eprintln!("Failed to initialize controller logging: {}", e);
                process::exit(0);
            }
            controller::controller();
        }
        Ok(ForkResult::Child) => {
            if let Err(e) = worker::init_worker_logging() {
                eprintln!("Failed to initialize worker logging: {}", e);
                process::exit(0);
            }
            worker::worker();
        }
        Err(_) => {
            eprintln!("Failed to fork process");
            process::exit(1);
        }
    }
}
