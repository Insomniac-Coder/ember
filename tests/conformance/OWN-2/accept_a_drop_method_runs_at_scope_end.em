#$ test: run-pass
#$ rules: OWN-2, DRP-1
# "When the owner goes out of scope (block end, reverse declaration order) …
# the value is **dropped**: its `drop` method (if any) runs, then its fields
# are dropped in reverse declaration order."
#
# The `drop` call was missing from the backend entirely. A struct whose only
# claim on `needs_drop` was its own `drop` method produced no lines at all, so
# the `Drop` statement lowered to nothing and **a declared destructor never
# ran** — silently, with no diagnostic and correct-looking output.

struct R:
    pub n: i32

    fn drop(mut self):
        println(self.n)

fn main():
    if true:
        r = R(7)
        println(r.n + 100)
    println(2)
#$ stdout: 107
#$ 7
#$ 2
