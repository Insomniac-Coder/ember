#$ test: run-fail
#$ rules: FN-6, FFI-21, CLO-10
#$ profiles: debug, development, release
#$ stdout: 2
#$ panics: a callable value holding captured state has no C entry

# ODR-102 — a stored callable value gives C its C entry; one made from a
# closure with captured state has none, and giving it to C panics where the
# conversion is, instead of handing C a null function.

fn add_one(value: i32) -> i32:
    return value + 1

fn main():
    base: i32 = 10
    f: fn(i32) -> i32 = add_one
    c: extern "C" fn(i32) -> i32 = f
    println(c(1))
    g: fn(i32) -> i32 = owned fn(x: i32) -> i32:
        return x + base
    d: extern "C" fn(i32) -> i32 = g
    println(d(1))
