use libc::{c_int, ioctl};
use nix::unistd;
use nix::sys::stat;
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::fs::{MetadataExt, FileTypeExt};
use std::{
    fs::OpenOptions,
    io::{Error, ErrorKind},
};
use hex::FromHex;

//
// A dynamic version of the rand_pool_info struct.
// We'll allocate enough bytes for the user-supplied entropy.
//
#[repr(C)]
struct RandPoolInfoDynamic {
    entropy_count: c_int,  // in bits
    buf_size: c_int,       // in bytes
    buf: Vec<u8>,          // user-provided random bytes
}

// The standard #define for RNDADDENTROPY in Linux (with dynamic size).
// The 0x5203 is the "function number", the rest is determined by _IOW('R', 0x03, <type>).
// We'll build the code at runtime based on the size. But for simplicity, here's a helper:
fn rndaddentropy_ioctl_code() -> u64 {
    // This is the raw code  _IOW('R', 0x03, int) but that doesn't encode dynamic size automatically.
    // Typically user-space libraries do an #ifdef or so. We'll just pick the function ID 0x03,
    // but the kernel rarely checks the size for RNDADDENTROPY if it's at least big enough.
    // We'll proceed with 0x40085203, but note on some arches you might need 0x40485203 for 40 bytes, etc.
    //
    // Because we are using a dynamic buffer, the kernel doesn't strictly enforce the size. 
    // Some more modern kernels do additional checks, so we might need a direct call or a macros approach.
    //
    // For a truly robust approach, you can do an ioctl macro from the 'nix' crate:
    //   ioctl_write_ptr_bad!(rndaddentropy, b'R', 0x03, RandPoolInfoDynamic);
    // For brevity, let's keep it direct:
    0x4008_5203
}

// If you want to forcibly reseed the CRNG (kernel >= 4.14), you can do:
const RNDRESEEDCRNG: u64 = 0x5207; // _IO('R', 0x07)

/// The core logic: decode hex, open /dev/urandom, run RNDADDENTROPY, optionally RNDRESEEDCRNG.
pub fn reset_entropy_with_bytes(hex_string: &str) -> std::io::Result<()> {
    eprintln!("-- reset_entropy_with_bytes called --");

    // 0. Print out some environment debug info
    log_caps_and_id();
    log_dev_stats("/dev/random");
    log_dev_stats("/dev/urandom");

    // 1. Decode the hex string into raw bytes
    let raw_bytes = match Vec::from_hex(hex_string) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Hex decode error for {hex_string}: {e}");
            return Err(Error::new(ErrorKind::InvalidData, format!("Hex decode error: {e}")));
        }
    };

    eprintln!("Decoded {} bytes from hex string", raw_bytes.len());
    if raw_bytes.is_empty() {
        return Err(Error::new(ErrorKind::InvalidData, "Empty hex string provided"));
    }

    // 2. Open /dev/urandom in read/write mode
    eprintln!("Opening /dev/urandom in read/write mode...");
    let file = match OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/urandom") {
            Ok(f) => {
                eprintln!("Successfully opened /dev/urandom");
                f
            },
            Err(e) => {
                eprintln!("Error opening /dev/urandom: {e}");
                return Err(e);
            }
        };
    let fd: RawFd = file.as_raw_fd();
    eprintln!("Got file descriptor = {fd}");

    // 3. Build the RandPoolInfoDynamic struct
    //    The kernel expects:
    //      entropy_count (bits)
    //      buf_size (bytes)
    //      buf (the actual random data).
    //
    //    We'll assume the raw_bytes are truly random. 
    //    You can do any logic you want to set 'entropy_count'.
    let entropy_bits = (raw_bytes.len() * 8) as c_int;
    let info = RandPoolInfoDynamic {
        entropy_count: entropy_bits,
        buf_size: raw_bytes.len() as c_int,
        buf: raw_bytes,
    };

    // 4. Issue the RNDADDENTROPY ioctl
    eprintln!("Issuing RNDADDENTROPY ioctl with {} bits of entropy...", entropy_bits);
    let ret = unsafe {
        ioctl(
            fd,
            rndaddentropy_ioctl_code(),
            &info as *const RandPoolInfoDynamic,
        )
    };

    if ret < 0 {
        let err = Error::last_os_error();
        eprintln!("RNDADDENTROPY ioctl failed. FD={fd}, error={err}");
        return Err(err);
    }
    eprintln!("RNDADDENTROPY ioctl succeeded (ret={ret})");

    // 5. (Optional) RNDRESEEDCRNG 
    eprintln!("Issuing RNDRESEEDCRNG ioctl...");
    let ret = unsafe { ioctl(fd, RNDRESEEDCRNG) };
    if ret < 0 {
        let err = Error::last_os_error();
         eprintln!("RNDRESEEDCRNG ioctl failed. FD={fd}, error={err}");
         return Err(err);
    }
    eprintln!("RNDRESEEDCRNG ioctl succeeded (ret={ret})");

    eprintln!("-- reset_entropy_with_bytes completed successfully --");
    Ok(())
}

// Debug helper: print out capabilities and UID
fn log_caps_and_id() {
    eprintln!("UID: {}", unistd::geteuid());
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("Cap") {
                eprintln!("{}", line);
            }
        }
    } else {
        eprintln!("Could not read /proc/self/status");
    }
}

// Debug helper: device stats
fn log_dev_stats(path: &str) {
    match std::fs::metadata(path) {
        Ok(meta) => {
            eprintln!("Stats for {path}:");
            eprintln!("  File type: char? {}", meta.file_type().is_char_device());
            eprintln!("  Mode (octal): {:o}", meta.mode());
            eprintln!(
                "  rdev (major, minor): ({}, {})",
                stat::major(meta.rdev()),
                stat::minor(meta.rdev())
            );
        }
        Err(e) => eprintln!("Cannot stat {path}: {e}"),
    }
}
