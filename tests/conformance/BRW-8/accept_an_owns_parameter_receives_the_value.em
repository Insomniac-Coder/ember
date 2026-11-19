# Clause BRW-8.4: "An `owns` parameter receives the value."
#$ test: run-pass
#$ rules: BRW-8, FN-1, OWN-9
#$ stdout: 4
#$ stdout: 2 1
# The callee owns what it receives and may change it; the caller keeps its own only by giving a copy.

fn grow(owns xs: Array[int]) -> int:
    xs.push(owns 4)
    return xs.len()

fn main():
    mine = [1, 2, 3]
    println(grow(owns mine))
    kept = [5]
    println(grow(owns copy of kept), kept.len())
