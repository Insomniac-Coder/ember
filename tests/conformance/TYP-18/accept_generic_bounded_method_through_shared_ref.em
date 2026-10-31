#$ test: run-pass
#$ rules: TYP-14, TYP-17, TYP-18
#$ profiles: debug, development, release
#$ stdout: 47

# A bound supplies the method, while the caller supplies a shared reference.
# Method resolution must read through `ref T` before consulting T's bounds.
interface Scored:
    fn score() -> i32

struct Good:
    value: i32

extend Good implements Scored:
    fn score() -> i32:
        return this.value

[T: needs Scored]
fn read_score(value: ref T) -> i32:
    return value.score()

fn main():
    good = Good(47)
    println(read_score(ref good))
