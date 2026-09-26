#$ test: run-pass
#$ rules: FFI-10, FFI-11, MOD-2
#$ profiles: debug, release, shipping
#$ stdout: 5

from modules.ffi_counted import counted

fn main():
    data: Array[u8] = Array[u8]()
    data.push(4)
    println(counted(data.as_span()))
