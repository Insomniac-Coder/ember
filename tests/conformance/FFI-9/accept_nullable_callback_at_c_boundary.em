#$ test: run-pass
#$ rules: FFI-9, TYP-13
#$ profiles: debug, release, shipping
#$ stdout: 42 0

pub extern "C" fn increase(value: i32) -> i32:
    return value + 1

pub extern "C" fn apply(callback: Option[extern "C" fn(i32) -> i32], value: i32) -> i32:
    match callback:
        Some(function):
            return function(value)
        None:
            return 0

fn main():
    println(apply(Some(increase), 41), apply(None, 41))
