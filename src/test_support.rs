//! Shared `unix`-only test helpers. Not compiled into the published crate
//! (`#[cfg(all(test, unix))]`).

use std::io::{Read, Write};
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Force a TCP RST on close by setting `SO_LINGER` to 0. `std`'s
/// `TcpStream::set_linger` is still unstable, so go through `setsockopt`
/// directly. A reset, not a graceful FIN, is the stale keep-alive symptom #63
/// targets (hyper surfaces it as a `ConnectionReset` `io::Error`, distinct from
/// an `IncompleteMessage`).
fn force_rst_on_close(fd: i32) {
    #[repr(C)]
    struct Linger {
        l_onoff: i32,
        l_linger: i32,
    }
    extern "C" {
        fn setsockopt(
            s: i32,
            level: i32,
            name: i32,
            val: *const core::ffi::c_void,
            len: u32,
        ) -> i32;
    }
    #[cfg(target_os = "linux")]
    let (sol_socket, so_linger) = (1i32, 13i32);
    #[cfg(not(target_os = "linux"))]
    let (sol_socket, so_linger) = (0xffffi32, 0x0080i32); // macOS / BSD
    let l = Linger {
        l_onoff: 1,
        l_linger: 0,
    };
    unsafe {
        setsockopt(
            fd,
            sol_socket,
            so_linger,
            &l as *const _ as *const core::ffi::c_void,
            std::mem::size_of::<Linger>() as u32,
        );
    }
}

/// Spawn a bare TCP server that resets the first `reset_count` connections with
/// a TCP RST before any response (the stale keep-alive symptom from #63), then
/// replies `200 OK` with `body`. Returns the base URL and a counter of accepted
/// connections so a test can assert how many attempts reached the wire.
pub(crate) fn reset_then_ok_server(reset_count: usize, body: String) -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let addr = listener.local_addr().expect("local addr");
    let conns = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&conns);
    std::thread::spawn(move || {
        let mut i = 0usize;
        for stream in listener.incoming() {
            let Ok(mut s) = stream else { continue };
            counter.fetch_add(1, Ordering::SeqCst);
            // Drain the client's request bytes so it finishes writing before we
            // act (otherwise the RST can race the request send).
            let mut buf = [0u8; 8192];
            let _ = s.read(&mut buf);
            if i < reset_count {
                force_rst_on_close(s.as_raw_fd());
                drop(s);
            } else {
                let head = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\
                     content-length: {}\r\nconnection: close\r\n\r\n",
                    body.len()
                );
                let _ = s.write_all(head.as_bytes());
                let _ = s.write_all(body.as_bytes());
                let _ = s.flush();
            }
            i += 1;
        }
    });
    (format!("http://{addr}"), conns)
}
