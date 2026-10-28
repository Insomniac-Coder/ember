#$ test: compile-fail
#$ rules: R8
#$ help: ks = sorted(m.keys().copy_each())
## R8 (b): a list of views a call made from a map's keys ends when the map may reshape; the way out
## keeps strings of your own (`.copy_each()`, verdict 33). The E3060 message of `V3/rules.md` section 3.

from std.collections import Map

fn main():
    m = Map[String, int]()
    m.insert(String("a"), 1)
    ks = sorted(m.keys())
    m.insert(String("z"), 1)
    println(ks)    #$ error[E3060]: `m.insert(String("z"), 1)` (line 13) changes `m`, whose keys `ks` (line 12) still views on line 14; use `ks` before line 13, or keep your own strings: write `ks = sorted(m.keys().copy_each())` in place of line 12
