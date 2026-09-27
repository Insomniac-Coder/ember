#$ test: run-pass
#$ rules: DRP-7, BRW-1
# Without an owning destructor, the borrowed field has no later read at scope
# exit; its source may end before the outer Option value.

struct Plain[T]:
    borrowed: ref i32
    marker: T

fn main():
    _outer: Option[Plain[i32]] = None
    if true:
        value: i32 = 5
        _outer = Some(Plain(ref value, 0))
    println(1)
#$ stdout: 1
