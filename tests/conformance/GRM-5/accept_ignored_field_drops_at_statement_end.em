#$ test: run-pass
#$ rules: GRM-5, OWN-2, DRP-3
#$ stdout: 2
#$ stdout: 0
#$ stdout: 1
#$ stdout: 1

# The unbound second field remains owned by the compiler's aggregate
# temporary. `_` must not leak it or extend it to block scope: R(2) drops at
# the end of this statement, before the continuation marker. R(1) moves into
# `kept` and drops at scope end.

struct R:
    pub value: i32

    fn drop(mut self):
        println(self.value)

fn make() -> (R, R):
    return (R(1), R(2))

fn main():
    kept, _ = make()
    println(0)
    println(kept.value)
