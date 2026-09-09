#$ test: run-pass
#$ rules: OWN-5, OWN-2, DRP-1
# "Overwriting a place that holds a live value drops the old value first."
#
# The other case in this directory tests only the parenthetical — that the new
# value is evaluated before the store — and it moves the old value into the
# call that builds the new one, so nothing is ever live to drop. The main
# clause went untested, and unimplemented: an overwrite ran no destructor at
# all, which for a struct owning an `Array[T]` is a leak (D-035).
#
# `R`'s drop prints, so the order is observable: the `1` must appear before
# the `0` that follows the assignment, and the `2` at scope end.

struct R:
    pub n: i32

    fn drop(mut self):
        println(self.n)

fn main():
    r = R(1)
    r = R(2)
    println(0)
#$ stdout: 1
#$ 0
#$ 2
