#$ test: run-pass
#$ rules: R2
#$ stdout: 20
## R2: the nearest program that is allowed: `=` on a `const ref mut` name writes the place it names
## (verdict 42: `const` says the name itself stays as it is, not the value).

fn main():
    a = 1
    r = const ref mut a
    r = 20
    println(a)
