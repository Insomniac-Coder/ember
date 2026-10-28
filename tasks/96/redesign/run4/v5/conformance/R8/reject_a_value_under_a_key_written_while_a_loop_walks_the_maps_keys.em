#$ test: compile-fail
#$ rules: BRW-18, CTL-2
#$ help: for k, ref mut v in counts:
## design rules: R8
## R8 (c), verdict 95 (which reverses verdict 90): while a loop walks a map's keys, nothing changes the
## map through its own name, a value under a key it already has included, as Rust refuses it; the
## loop's own handle changes the value it is on. AN-12; rules section 3's E3020 message, in this
## program's lines.

from std.collections import Map

fn main():
    counts = Map[String, int]()
    counts.insert(String("a"), 1)
    counts.insert(String("b"), 2)
    for k in counts.keys():
        counts[k] *= 2    #$ error[E3020]: `counts[k] *= 2` (line 16) changes `counts`, whose keys this loop walks (line 15); change each value through the loop's own handle: write `for k, ref mut v in counts:` in place of line 15 and `v *= 2` in place of line 16
    println(counts["a"], counts["b"])
