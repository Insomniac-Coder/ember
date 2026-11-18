#$ test: run-pass
#$ rules: TYP-25
#$ stdout: 6 12 24 6
## `[TYP-25]` - arguments are positional or named, positional first; a parameter with a default may be omitted.

fn area(w: int, h: int, scale: int = 1) -> int:
    return w * h * scale

fn main():
    println(area(2, 3), area(2, 3, 2), area(w=2, h=3, scale=4), area(3, h=2))
