#$ test: compile-fail
#$ rules: TYP-24, TYP-21
# D-428 — an operator whose operand fits two instances equally (an `i32`
# widens to both `i64` and `i128`) is `E2070`, as the method's call is. It
# took whichever instance was registered last, with no diagnostic.

struct V:
    x: int

extend V implements Add[i64]:
    type Output = int
    fn add(self, other: i64) -> int:
        return 64

extend V implements Add[i128]:
    type Output = int
    fn add(self, other: i128) -> int:
        return 128

fn main():
    v = V(x = 0)
    s: i32 = 3
    println(v + s)       #$ error[E2070]: `+` is offered by both `std.core.Add[i64]` and `std.core.Add[i128]`
    println(v.add(s))    #$ error[E2070]: `add` is offered by both `std.core.Add[i64]` and `std.core.Add[i128]`
