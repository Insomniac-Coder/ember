#$ test: run-pass
#$ rules: TYP-13, FFI-9
#$ profiles: debug, release, shipping
#$ stdout: 8 8
#$ stdout: 42
#$ stdout: none

pub extern "C" fn increase(value: i32) -> i32:
    return value + 1

fn main():
    println(mem.size_of[Option[extern "C" fn(i32) -> i32]](),
            mem.size_of[extern "C" fn(i32) -> i32]())
    present: Option[extern "C" fn(i32) -> i32] = Some(increase)
    match present:
        Some(callback):
            println(callback(41))
        None:
            println("unexpected")
    absent: Option[extern "C" fn(i32) -> i32] = None
    match absent:
        Some(_):
            println("unexpected")
        None:
            println("none")
