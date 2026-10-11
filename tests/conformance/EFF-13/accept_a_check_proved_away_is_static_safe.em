#$ test: run-pass
#$ rules: EFF-13, EXC-3, EXC-8
#$ profiles: debug, release, shipping
#$ stdout: 3

# `[EFF-13]` — what counts is the check left after the proofs: one handle
# local the loop does not reassign holds its access for the loop (`[EXC-8]`)
# or needs none (`[EXC-3]`), and the function is `@static_safe`.

class Counter:
    value: i32

    fn bump(mut self):
        self.value = self.value + 1

@static_safe
fn three(mut c: Counter):
    for i in 0..3:
        c.bump()

fn main():
    c = Counter(0)
    three(c)
    println(c.value)
