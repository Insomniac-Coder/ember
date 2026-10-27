#$ test: compile-fail
#$ rules: BRW-18, CTL-2
#$ help: added: Array[String] = []
## design rules: R8
## R8 (c), verdict 72: the loop rule covers everything a loop goes through, a map's keys included; the
## change is refused where it is made (E3020), with the list's message, not at the next use of `k`.

from std.collections import Map

fn main():
    counts = Map[String, int]()
    counts.insert(String("a"), 1)
    for k in counts.keys():
        counts.insert(String("b"), 2)    #$ error[E3020]: `counts.insert(String("b"), 2)` (line 13) adds to `counts`, whose keys this loop walks (line 12); collect the new keys in a list of your own: write `added: Array[String] = []` before the loop, `added.push(String("b"))` in place of this line, and `for k in owns added: counts.insert(k, 2)` after the loop
        println(k)
