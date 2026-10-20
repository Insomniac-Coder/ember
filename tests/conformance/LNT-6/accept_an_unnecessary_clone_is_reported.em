#$ test: run-pass
#$ command: lint
#$ rules: LNT-6
#$ stdout: 5 3 3
## `[LNT-6]` — `L2001`: a clone of a value the function owns and never uses again; moving it
## costs nothing. A clone of a value used afterwards, or of a borrowed parameter, is no lint.

fn take(owned s: String) -> int:
    return s.len()

fn measure(s: String) -> int:
    copy = s.clone()
    return take(copy)

fn main():
    first = String.from("hello")
    second = first.clone()  #$ warning[L2001]: `first` is cloned, then never used again
    kept = String.from("abc")
    other = kept.clone()
    println(take(second), measure(other), kept.len())
