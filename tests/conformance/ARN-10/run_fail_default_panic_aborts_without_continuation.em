#$ test: run-fail
#$ rules: ARN-3, ARN-10, PAN-1, TST-23
#$ panics: division by zero
#$ stdout: before default panic
#$ profiles: debug, release, shipping

from std.core import Default

struct Pixel:
    value: i32

extend Pixel implements Default:
    fn default() -> Pixel:
        println("before default panic")
        numerator: i32 = 1
        denominator: i32 = 0
        value = numerator / denominator
        println("continuation after default panic")
        return Pixel(value)

fn main():
    arena = Arena.with_capacity(64)
    values = arena.alloc_array[Pixel](2)
    println(values.len())
