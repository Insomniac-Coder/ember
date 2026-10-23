//! `[BLD-7]` — the number of physical cores, the front end's default number of threads (`-j`).
//!
//! A core's second hardware thread shares its execution units, and the compiler's analyses keep them
//! busy: one thread per core is the default the rule names. Where the count cannot be read, the
//! logical processors the system reports.

#![allow(unsafe_code)] // the processor topology is the system's to report

/// The machine's physical cores, at least one.
pub fn physical_cores() -> usize {
    physical().filter(|&cores| cores > 0).unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |count| count.get()))
}

#[cfg(windows)]
fn physical() -> Option<usize> {
    unsafe extern "system" {
        fn GetLogicalProcessorInformationEx(relationship: u32, buffer: *mut u8, length: *mut u32) -> i32;
    }
    const RELATION_PROCESSOR_CORE: u32 = 0;
    let mut length: u32 = 0;
    // SAFETY(ffi): a null buffer of length 0 asks for the length the information needs.
    unsafe { GetLogicalProcessorInformationEx(RELATION_PROCESSOR_CORE, std::ptr::null_mut(), &mut length) };
    if length == 0 {
        return None;
    }
    let mut buffer = vec![0u8; length as usize];
    // SAFETY(ffi): `buffer` holds `length` bytes, which the call fills with records.
    if unsafe { GetLogicalProcessorInformationEx(RELATION_PROCESSOR_CORE, buffer.as_mut_ptr(), &mut length) } == 0 {
        return None;
    }
    // Each record: its relationship (u32) and its size (u32), then its body; one per core.
    let mut cores = 0;
    let mut at = 0usize;
    while at + 8 <= length as usize {
        let size = u32::from_le_bytes(buffer[at + 4..at + 8].try_into().ok()?) as usize;
        if size == 0 {
            break;
        }
        cores += 1;
        at += size;
    }
    Some(cores)
}

#[cfg(target_os = "linux")]
fn physical() -> Option<usize> {
    // Each (package, core) pair once.
    let text = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    let mut cores = std::collections::BTreeSet::new();
    let mut package = None;
    for line in text.lines() {
        let Some((key, value)) = line.split_once(':') else { continue };
        match key.trim() {
            "physical id" => package = value.trim().parse::<u32>().ok(),
            "core id" => {
                cores.insert((package, value.trim().parse::<u32>().ok()?));
            }
            _ => {}
        }
    }
    (!cores.is_empty()).then_some(cores.len())
}

#[cfg(target_os = "macos")]
fn physical() -> Option<usize> {
    unsafe extern "C" {
        fn sysctlbyname(name: *const std::ffi::c_char, value: *mut std::ffi::c_void, length: *mut usize, new: *mut std::ffi::c_void, new_length: usize) -> i32;
    }
    let mut cores: i32 = 0;
    let mut length = std::mem::size_of::<i32>();
    // SAFETY(ffi): a NUL-terminated name and a buffer of `length` bytes for its integer value.
    let read = unsafe {
        sysctlbyname(c"hw.physicalcpu".as_ptr(), (&mut cores as *mut i32).cast(), &mut length, std::ptr::null_mut(), 0)
    };
    (read == 0 && cores > 0).then_some(cores as usize)
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
fn physical() -> Option<usize> {
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn there_is_a_core_and_no_more_than_the_logical_processors() {
        let cores = super::physical_cores();
        let logical = std::thread::available_parallelism().map_or(1, |count| count.get());
        assert!(cores >= 1 && cores <= logical, "{cores} physical, {logical} logical");
    }
}
