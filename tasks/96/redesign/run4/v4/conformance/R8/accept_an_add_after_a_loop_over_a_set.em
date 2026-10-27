#$ test: run-pass
#$ rules: R8
#$ stdout:
#$ a
#$ 2
## R8 (c), verdict 72: the way out E3020's message gives: the new names are collected during the walk of
## the set and added after the loop. The nearest program to `reject_an_add_while_a_loop_walks_a_set.em`
## that is allowed.

from std.collections import Set

fn main():
    seen = Set[String]()
    seen.add(String("a"))
    added: Array[String] = []
    for name in seen:
        added.push(String("b"))
        println(name)
    for n in owns added:
        seen.add(n)
    println(seen.len())
