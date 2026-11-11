#$ test: run-pass
#$ rules: STR-7
#$ stdout: 3 none
#$ stdout: 2 7
# `[STR-7]` — inside a generic struct's body, `Self` names the struct with its
# own parameters wherever it is written, inside another type too:
# `Option[Self]` and `Array[Self]` (D-333: only a bare `Self` was replaced, so
# `Option[Self]` stayed unresolved). `pair` takes its object and `other` over (`owns this`,
# `owns other`): a method that only reads them may not hand them to `push` (`[OWN-11]`, `E3013`).

[T]
struct Cell2:
    v: T

    static fn maybe(v: T, keep: bool) -> Option[Self]:
        if keep:
            return Some(Cell2(v))
        return None

    fn pair(owns other: Self) owns this -> Array[Self]:
        out: Array[Self] = Array()
        out.push(owns this)
        out.push(owns other)
        return out

fn main():
    kept = Cell2[i64].maybe(3, true)
    dropped = Cell2[i64].maybe(4, false)
    match dropped:
        Some(_):
            println(kept.unwrap().v, "some")
        None:
            println(kept.unwrap().v, "none")
    both = Cell2(1).pair(owns Cell2(7))
    println(len(both), both[1].v)
