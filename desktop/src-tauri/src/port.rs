//! Let the OS assign a free loopback port.
//!
//! There is a small race window between closing the probe socket and the
//! backend binding the same port; sidecar startup retries cover it.

use std::io;
use std::net::TcpListener;

pub fn pick_free_port() -> io::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assigns_nonzero_port() {
        let port = pick_free_port().unwrap();
        assert_ne!(port, 0);
    }
}
