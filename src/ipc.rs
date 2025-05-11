use super::common::{DBUS_INTERFACE, DBUS_SERVICE_NAME, Result};
use dbus::Message;
use dbus::blocking::{BlockingSender, Connection};
use std::time::Duration;

const DBUS_TIMEOUT: Duration = Duration::from_secs(2);

pub fn send_dbus_message(path: &str, method: &str, message: &str) -> Result<()> {
    let conn = Connection::new_session()?;
    let msg =
        Message::new_method_call(DBUS_SERVICE_NAME, path, DBUS_INTERFACE, method)?.append1(message);

    conn.send_with_reply_and_block(msg, DBUS_TIMEOUT)?;
    Ok(())
}

pub fn setup_dbus_receiver(_path: &str) -> Result<dbus::blocking::Connection> {
    let conn = Connection::new_session()?;
    conn.request_name(DBUS_SERVICE_NAME, false, true, false)?;

    Ok(conn)
}
