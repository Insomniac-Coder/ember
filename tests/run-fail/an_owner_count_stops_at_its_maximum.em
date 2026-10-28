#$ test: run-fail
#$ rules: RT-7, TST-31
#$ count_bits: 8
#$ panics: too many owners of a `Node` (255)
## [RT-7], [TST-31]: built with 8-bit counts, an owner count's maximum is 255, stored and compared as
## the 32-bit one is; the push that would give `n` its 256th owner stops the program before its +1.
class Node:
    v: int = 0

fn main():
    n = Node()
    keep: Array[Node] = []
    for _ in 0..300:
        keep.push(n)
    println(keep.len())
