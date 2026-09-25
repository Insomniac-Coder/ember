#$ test: run-pass
#$ rules: STR-7
#$ stdout: 3 none
#$ stdout: 2 7
# `[STR-7]` — inside a generic struct's body, `Self` names the struct with its
# own parameters wherever it is written, inside another type too:
# `Option[Self]` and `Array[Self]` (D-333: only a bare `Self` was replaced, so
# `Option[Self]` stayed unresolved).

struct Cell2[T]:
    v: T

    fn maybe(v: T, keep: bool) -> Option[Self]:
        if keep:
            return Some(Cell2(v))
        return None

    fn pair(self, other: Self) -> Array[Self]:
        out: Array[Self] = Array()
        out.push(self)
        out.push(other)
        return out

fn main():
    kept = Cell2[i64].maybe(3, true)
    dropped = Cell2[i64].maybe(4, false)
    match dropped:
        Some(_):
            println(kept.unwrap().v, "some")
        None:
            println(kept.unwrap().v, "none")
    both = Cell2(1).pair(Cell2(7))
    println(len(both), both[1].v)
