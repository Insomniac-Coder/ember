#$ test: run-pass
#$ rules: FN-6, FFI-9, FFI-21
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: 42
#$ stdout: native-only

fn add_one(value: i32) -> i32:
    return value + 1

fn factory() -> extern "C" fn(i32) -> i32:
    return add_one

fn native_text(value: str) -> str:
    return value

fn main():
    make: extern "C" fn() -> extern "C" fn(i32) -> i32 = factory
    callback = make()
    println(callback(41))
    optional: Option[extern "C" fn(i32) -> i32] = Some(callback)
    match optional:
        Some(present):
            println(present(41))
        None:
            panic("missing callback")
    ordinary: fn(str) -> str = native_text
    println(ordinary("native-only"))
