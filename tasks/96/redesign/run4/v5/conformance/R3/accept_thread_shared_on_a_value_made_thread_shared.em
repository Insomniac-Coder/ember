#$ test: run-pass
#$ rules: OWN-8, THR-1
#$ stdout: 0
## design rules: R3, R15
## R3, R15: the nearest program that is allowed: the object is created shared between threads, and
## `thread_shared names` makes one more owner of it.

class Names:
    items: Array[String] = []

fn share_plain():
    names = thread_shared Names()
    keep = thread_shared names
    println(keep.items.len())

fn main():
    share_plain()
