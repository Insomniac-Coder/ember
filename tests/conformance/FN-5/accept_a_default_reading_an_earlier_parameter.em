#$ test: run-pass
#$ rules: FN-5
#$ stdout: 9
#$ 8
# A default reads the already-bound earlier argument and runs only when omitted.

fn reads(n: int, m: int = n * 2) -> int:
    return n + m

fn main():
    println(reads(3))
    println(reads(m=6, n=2))
