#$ test: run-pass
#$ rules: OWN-8
#$ stdout: 3
#$ 7
## design rules: R3
## R3: the nearest program that is allowed: `ref b` says `b`'s default (a read-only view) and may be
## written on any declared name.

fn main():
    pairs = [(1, 2), (3, 4)]
    for (ref mut a, ref b) in pairs:
        a += b
    for (a, b) in pairs:
        println(a)
