#$ test: run-pass
#$ rules: R12
#$ stdout: 0
## R12 (rules.md lines 662-670, 752-756): a lambda that only reads the real `score` changes nothing, as
## the compiler works out from its body; it is kept in a list of `fn()` like any function, and no line
## says anything about what it changes.

fn main():
    score = 0
    hs: Array[fn()] = []
    hs.push(fn(): println(score))
    for ref mut h in hs:
        h()
