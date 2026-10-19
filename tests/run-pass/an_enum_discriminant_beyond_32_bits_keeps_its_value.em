#$ test: run-pass
#$ rules: TYP-12
#$ stdout: 5000000000 -5000000000 7
#$ stdout: true

## D-720 — an enum with no `@repr` whose discriminant needs more than 32 bits gets a 64-bit
## discriminant type; it was given `uint32_t`, so `Big.Huge as i64` printed 705032704.

enum Big:
    Small = 7
    Huge = 5000000000

enum Low:
    Deep = -5000000000
    Zero = 0

fn main():
    println(Big.Huge as i64, Low.Deep as i64, Big.Small as i64)
    println(Big.Huge != Big.Small)
