#$ test: run-pass
#$ rules: TYP-17, IFC-1
# A receiver-less interface member is an associated function on the
# implementing type, not a value method with an implicit receiver.

from std.core import Default

struct Pixel:
    value: i32

extend Pixel implements Default:
    static fn default() -> Pixel:
        return Pixel(17)

[T: needs Default]
fn make() -> T:
    return T.default()

fn main():
    direct = Pixel.default()
    println(direct.value)
    pixel = make[Pixel]()
    println(pixel.value)
#$ stdout: 17
#$ stdout: 17
