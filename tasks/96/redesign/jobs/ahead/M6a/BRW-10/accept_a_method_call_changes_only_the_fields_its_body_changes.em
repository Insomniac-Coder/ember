# Clause BRW-10.1: "What a method does to its object is worked out from its body, for every method,
# private or `pub` (`[EXC-15]`): a call `w.bump()` changes only the fields `bump`'s body changes, and
# those its callees' bodies change, so it may be called while a loop walks `w.names` if `bump`
# touches only `w.count`."
#$ test: run-pass
#$ rules: BRW-10, EXC-15, EXC-19, BRW-18
#$ stdout: a
#$ stdout: bb
#$ stdout: 6
# `bump` touches `count` through its callee `tick`, and the `pub` method `bump_pub` touches only
# `count`: both may be called while the loop walks `w.names`.

struct Words:
    names: Array[String]
    count: int = 0

    fn tick():
        this.count += 1

    fn bump():
        this.tick()

    pub fn bump_pub():
        this.count += 2

fn main():
    w = Words([String("a"), String("bb")])
    for name in w.names:
        w.bump()
        w.bump_pub()
        println(name)
    println(w.count)
