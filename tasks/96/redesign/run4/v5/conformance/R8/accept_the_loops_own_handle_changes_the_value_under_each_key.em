#$ test: run-pass
#$ rules: BRW-18, CTL-2
#$ stdout: 2 4
## design rules: R8, R3
## R8 (c), verdict 95: walking a map's keys and values, `for k, ref mut v in counts:`, the loop's own
## handle may change the value it is on, so every count doubles. The nearest program to
## `reject_a_value_under_a_key_written_while_a_loop_walks_the_maps_keys.em` that is allowed, with the
## lines its message gives.

from std.collections import Map

fn main():
    counts = Map[String, int]()
    counts.insert(String("a"), 1)
    counts.insert(String("b"), 2)
    for k, ref mut v in counts:
        v *= 2
    println(counts["a"], counts["b"])
