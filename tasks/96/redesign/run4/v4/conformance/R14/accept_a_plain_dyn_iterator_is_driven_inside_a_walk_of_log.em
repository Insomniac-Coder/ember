#$ test: run-pass
#$ rules: R12, R14, R8
#$ stdout: 33
## R12: "a `dyn` type with no `changes` changes nothing", so driving one inside a walk of `LOG` is
## allowed (R8 (c)).

static mut LOG: Array[int] = [1, 2]

fn add_each(owns it: owns dyn Iterator[Item = int]) -> int:
    total = 0
    for entry in LOG:
        if it.next() exists as v:
            total += v + entry
    return total

fn main():
    xs = [10, 20]
    println(add_each(xs.iter()))
