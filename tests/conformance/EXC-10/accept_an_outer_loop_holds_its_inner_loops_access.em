#$ test: run-pass
#$ rules: EXC-8, EXC-9, EXC-10, EXC-11, EXC-12
#$ profiles: debug, development, release
#$ stdout: 12
#$ 15
#$ 39
#$ assert-c-count: contains("ember_object_begin_write") == 3
#$ assert-c-count: contains("ember_object_end_write") == 3

# `[EXC-10]` — the outer loops only count, so each nest holds its inner loop's
# access from the first inner turn to the nest's end: one check per nest, not
# one per outer turn. An inner loop that runs no turn (the first `r`) begins
# nothing, and three loops deep the access moves out twice. The driver test
# checks the `inner_loop_reused` proof.

class Counter:
    value: i32
    log: Array[int]

    fn bump(mut self):
        self.value = self.value + 1

fn main():
    counter = Counter(0, [])
    alias = counter
    for r in 0..3:
        for i in 0..4:
            counter.bump()
    println(alias.value)
    for r in 0..3:
        for i in 0..r:
            counter.bump()
    println(alias.value)
    for a in 0..2:
        for b in 0..3:
            for c in 0..4:
                counter.bump()
    println(alias.value)
