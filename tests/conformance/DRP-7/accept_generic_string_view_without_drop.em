#$ test: run-pass
#$ rules: DRP-7, LT-3
# The source ends after the Plain value's last ordinary use; Plain has no drop.

struct Plain[T]:
    borrowed: str
    marker: T

fn main():
    _outer: Option[Plain[i32]] = None
    if true:
        value: String = "drop"
        _outer = Some(Plain(value.as_str(), 0))
    println(1)
#$ stdout: 1
