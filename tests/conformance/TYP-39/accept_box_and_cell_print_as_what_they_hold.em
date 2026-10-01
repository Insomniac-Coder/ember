#$ test: run-pass
#$ rules: TYP-39, STD-9
#$ stdout: 5
#$ stdout: 7
#$ stdout: 5 and 7
#$ stdout: hi
#$ stdout: [1, 2]
#$ stdout: 3    3
# ODR-094 — a `Box[T]` and a `Cell[T]` show as the `T` they hold, alone, in an
# f-string with a spec, inside a collection, and one inside the other.

fn main():
    b = Box(5)
    c = Cell(7)
    println(b)
    println(c)
    println(f"{b} and {c}")
    s = Box("hi".to_string())
    println(s)
    xs = [Box(1), Box(2)]
    println(xs)
    nested = Box(Cell(3))
    println(nested, f"{nested:>4}")
