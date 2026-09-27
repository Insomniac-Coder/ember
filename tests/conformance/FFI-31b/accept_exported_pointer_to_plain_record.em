#$ test: run-pass
#$ rules: FFI-31b
#$ profiles: debug
#$ stdout: 1

struct Plain:
    value: i32

pub extern "C" fn accepts_plain(value: *Plain) -> i32:
    return 1

fn main():
    println(accepts_plain(null[*Plain]()))
