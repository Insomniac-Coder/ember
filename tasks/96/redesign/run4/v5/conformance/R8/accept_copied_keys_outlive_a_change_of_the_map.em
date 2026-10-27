#$ test: run-pass
#$ rules: BRW-16
#$ stdout: ['a'] 2
## design rules: R8
## R8 (b): `.copy_each()` makes strings of your own (verdict 33), which a change of the map does not end.

from std.collections import Map

fn main():
    m = Map[String, int]()
    m.insert(String("a"), 1)
    ks = sorted(m.keys().copy_each())
    m.insert(String("z"), 1)
    println(ks, m.len())
