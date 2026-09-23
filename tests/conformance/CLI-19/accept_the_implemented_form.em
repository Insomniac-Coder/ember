#$ test: run-pass
#$ rules: CLI-19, GRM-25
#$ profiles: debug, release, shipping
#$ stdout: true
# The form the `E0900` help names compiles.

fn next() -> int:
    return 5

fn main():
    n = next()
    println(0 < n < 10)
