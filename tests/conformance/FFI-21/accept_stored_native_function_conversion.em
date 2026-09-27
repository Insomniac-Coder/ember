#$ test: run-pass
#$ rules: FN-6, CLO-3, FFI-21
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: 42
#$ stdout: 41
#$ stdout: 42
#$ stdout: 42

struct Holder:
    callback: fn(i32) -> i32

fn add_one(value: i32) -> i32:
    return value + 1

fn add_two(value: i32) -> i32:
    return value + 2

fn forward[T](owned value: T) -> T:
    return value

fn main():
    holder = Holder(callback = add_one)
    holder.callback = add_two
    native: fn(i32) -> i32 = holder.callback
    println(native(40))
    foreign: extern "C" fn(i32) -> i32 = holder.callback
    println(foreign(40))
    callbacks = [add_one, add_two]
    for callback in callbacks:
        converted: extern "C" fn(i32) -> i32 = callback
        println(converted(40))
    forwarded = forward(add_two)
    converted: extern "C" fn(i32) -> i32 = forwarded
    println(converted(40))
