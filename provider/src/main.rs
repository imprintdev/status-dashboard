use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::{SystemTime, UNIX_EPOCH};

fn iso8601() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    // Gregorian calendar from days since epoch
    let (y, mo, d) = days_to_ymd(days);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, d, h, m, s)
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut y = 1970u64;
    loop {
        let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
        let dy = if leap { 366 } else { 365 };
        if days < dy {
            break;
        }
        days -= dy;
        y += 1;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let months = [
        31u64,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut mo = 1u64;
    for dm in &months {
        if days < *dm {
            break;
        }
        days -= dm;
        mo += 1;
    }
    (y, mo, days + 1)
}

#[cfg(target_os = "windows")]
fn disk_free_gb() -> f64 {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    #[allow(non_camel_case_types)]
    type BOOL = i32;
    #[allow(non_camel_case_types)]
    type ULARGE_INTEGER = u64;
    #[allow(non_camel_case_types)]
    type LPCWSTR = *const u16;

    extern "system" {
        fn GetDiskFreeSpaceExW(
            lpDirectoryName: LPCWSTR,
            lpFreeBytesAvailableToCaller: *mut ULARGE_INTEGER,
            lpTotalNumberOfBytes: *mut ULARGE_INTEGER,
            lpTotalNumberOfFreeBytes: *mut ULARGE_INTEGER,
        ) -> BOOL;
    }

    let path: Vec<u16> = OsStr::new("C:\\").encode_wide().chain(Some(0)).collect();
    let mut free: u64 = 0;
    let mut _total: u64 = 0;
    let mut _total_free: u64 = 0;
    unsafe {
        GetDiskFreeSpaceExW(path.as_ptr(), &mut free, &mut _total, &mut _total_free);
    }
    free as f64 / 1_073_741_824.0
}

#[cfg(not(target_os = "windows"))]
fn disk_free_gb() -> f64 {
    #[repr(C)]
    struct Statvfs {
        f_bsize: u64,
        f_frsize: u64,
        f_blocks: u64,
        f_bfree: u64,
        f_bavail: u64,
        _pad: [u8; 80],
    }

    unsafe extern "C" {
        fn statvfs(path: *const u8, buf: *mut Statvfs) -> i32;
    }

    let mut st = Statvfs {
        f_bsize: 0,
        f_frsize: 0,
        f_blocks: 0,
        f_bfree: 0,
        f_bavail: 0,
        _pad: [0; 80],
    };
    unsafe {
        statvfs(b"/\0".as_ptr(), &mut st);
    }
    (st.f_bavail * st.f_frsize) as f64 / 1_073_741_824.0
}

fn handle(mut stream: std::net::TcpStream) {
    let mut buf = [0u8; 512];
    let _ = stream.read(&mut buf);

    // Only respond to GET requests
    if !buf.starts_with(b"GET ") {
        let _ = stream.write_all(b"HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\n\r\n");
        return;
    }

    let time = iso8601();
    let disk = disk_free_gb();
    let body = format!(
        "{{\"ok\":true,\"time\":\"{}\",\"disk_free_gb\":{:.2}}}",
        time, disk
    );
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
}

fn main() {
    let addr = "0.0.0.0:9000";
    let listener = TcpListener::bind(addr).expect("failed to bind");
    eprintln!("listening on http://{}", addr);
    for s in listener.incoming().flatten() {
        handle(s);
    }
}
