#$ test: compile-fail
#$ rules: R8
#$ help: added: Array[String] = []
## R8 (c): one rule for every loop: a map does not change while a `for` loop walks it, here through
## its keys (R15 names lists, strings, maps and sets as what a `for` walks).

from std.collections import Map

fn main():
    counts = Map[String, int]()
    counts.insert(String("a"), 1)
    for k in counts.keys():
        counts.insert(String("b"), 2)    #$ error[E3020]: `counts.insert(String("b"), 2)` adds to `counts`, which this loop walks (line 12); collect the new keys in a list of your own: write `added: Array[String] = []` before the loop, `added.push(String("b"))` in place of this line, and `for k in owns added: counts.insert(k, 2)` after the loop
        println(k)
