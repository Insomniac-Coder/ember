#$ test: run-pass
#$ rules: STR-5, ENM-3
#$ stdout: Red Red true 1
## `[STR-5]` - a unit-only enum is `Copy`, `Ord` and `Hash` automatically (`[ENM-3]`), and `Eq` and `Debug`
## as every struct or enum is when its parts are.

enum Color:
    Red
    Green

fn main():
    c = Color.Red
    d = c
    cm: Map[Color, int] = {}
    cm[Color.Red] = 1
    println(c, d, c < Color.Green and c == d, cm.len())
