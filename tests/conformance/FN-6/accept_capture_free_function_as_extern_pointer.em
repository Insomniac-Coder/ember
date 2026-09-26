#$ test: run-pass
#$ rules: FN-6, FFI-9
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: 42
#$ stdout: 42

fn increase(value: i32) -> i32:
    return value + 1

fn main():
    callback: extern "C" fn(i32) -> i32 = increase
    println(callback(41))
    inline: extern "C" fn(i32) -> i32 = fn(value: i32) -> i32 => value + 1
    println(inline(41))
    native: fn(i32) -> i32 = increase
    forwarded: extern "C" fn(i32) -> i32 = native
    println(forwarded(41))
