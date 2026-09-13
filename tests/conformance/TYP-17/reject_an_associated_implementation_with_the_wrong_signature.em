#$ test: compile-fail
#$ rules: TYP-17, IFC-1

from std.core import Default

struct Pixel:
    value: i32

extend Pixel implements Default:
    fn default(seed: i32) -> Pixel:
        return Pixel(seed)

#$ error[E2040]: `Pixel.default` does not match the signature required by `std.core.Default`
fn main():
    pass
