#$ test: run-pass
#$ rules: ARN-3, ARN-10, ARN-12, TST-23

from std.core import Default

struct Pixel:
    value: i32

extend Pixel implements Default:
    fn default() -> Pixel:
        return Pixel(23)

fn main():
    arena = Arena.with_capacity(64)
    values: MutSpan[Pixel] = arena.alloc_array[Pixel](3)
    println(values.len())
    println(values[0].value)
    println(values[1].value)
    println(values[2].value)
#$ stdout: 3
#$ 23
#$ 23
#$ 23
