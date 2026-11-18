#$ test: run-pass
#$ rules: TYP-21, TXT-11
#$ stdout: xy
#$ abc
#$ 7
## `[TYP-21]` - `String` implements `Add[StrView, Output = String]` and `AddAssign[StrView]`, so it meets `T: needs
## Add[StrView, Output = T]`; `a + b` on it makes a new `String` and `+=` adds to the one it is written on; and an
## operator method on a number is the operator (`3.add(4)` is `3 + 4`).

[T: needs Add[StrView, Output = T]]
fn cat(a: T, b: StrView) -> T:
    return a + b

[T: needs AddAssign[StrView]]
fn append(ref mut a: T, b: StrView):
    a += b

fn main():
    println(cat(String("x"), "y"))
    s = String("a")
    append(ref mut s, "bc")
    println(s)
    println(3.add(4))
