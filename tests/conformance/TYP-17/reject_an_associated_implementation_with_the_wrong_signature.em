#$ test: compile-fail
#$ rules: TYP-17, IFC-1

from std.core import Default

struct Pixel:
    value: i32

extend Pixel implements Default:
    static fn default(seed: i32) -> Pixel:
        return Pixel(seed)

#$ error[E2040]: `Pixel.default` takes 1 parameter, and `Default.default` takes 0; write `static fn default() -> Pixel:`
fn main():
    pass
