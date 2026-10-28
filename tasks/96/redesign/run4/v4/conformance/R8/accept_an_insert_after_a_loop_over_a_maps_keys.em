#$ test: run-pass
#$ rules: R8
#$ stdout:
#$ a
#$ 2
## R8 (c), verdict 72: the way out E3020's message gives: the new keys are collected during the walk
## and inserted after the loop.

from std.collections import Map

fn main():
    counts = Map[String, int]()
    counts.insert(String("a"), 1)
    added: Array[String] = []
    for k in counts.keys():
        added.push(String("b"))
        println(k)
    for k in owns added:
        counts.insert(k, 2)
    println(counts.len())
