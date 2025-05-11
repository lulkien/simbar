pub const SOCKET_PATH: &str = "/tmp/controller_worker.sock";
pub const LOG_DIR: &str = "/tmp/myapp";

pub const DBUS_SERVICE_NAME: &str = "com.example.ControllerWorker";
pub const DBUS_CONTROLLER_PATH: &str = "/com/example/Controller";
pub const DBUS_WORKER_PATH: &str = "/com/example/Worker";
pub const DBUS_INTERFACE: &str = "com.example.ControllerWorker1";

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
