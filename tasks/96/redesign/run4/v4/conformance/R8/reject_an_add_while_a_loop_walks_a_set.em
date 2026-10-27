#$ test: compile-fail
#$ rules: R8
#$ help: added: Array[String] = []
## R8 (c), verdict 72: the loop rule covers everything a loop goes through, a set included; the change is
## refused where it is made (E3020), with the list's message.

from std.collections import Set

fn main():
    seen = Set[String]()
    seen.add(String("a"))
    for name in seen:
        seen.add(String("b"))    #$ error[E3020]: `seen.add(String("b"))` (line 13) adds to `seen`, which this loop walks (line 12); collect the new names in a list of your own: write `added: Array[String] = []` before the loop, `added.push(String("b"))` in place of this line, and `for n in owns added: seen.add(n)` after the loop
        println(name)
