#$ test: compile-fail
#$ rules: TYP-12
#$ error[E5015]: `Color` has no foreign representation
## `[TYP-12]` - an enum crossing a foreign boundary MUST carry `@repr`: one that does not is refused at an export.

enum Color:
    Red
    Green

extern "C" fn paint(c: Color) -> int:
    return 1

fn main():
    println(1)
