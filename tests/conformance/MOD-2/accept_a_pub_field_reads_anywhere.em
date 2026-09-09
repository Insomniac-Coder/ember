#$ test: run-pass
#$ rules: MOD-2
## A `pub` field reads from anywhere; a private one is reached through a
## function the declaring module exports.

from support.shapes import make, height_of

fn main():
    b = make(3, 4)
    println(b.width)
    println(height_of(b))
#$ stdout: 3
#$ 4
