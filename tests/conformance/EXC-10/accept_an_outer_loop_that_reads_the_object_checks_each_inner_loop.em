#$ test: run-pass
#$ rules: EXC-9, EXC-10, EXC-19
#$ profiles: debug, development, release
#$ stdout: [2, 4, 6]

# `[EXC-10]`, `[EXC-9]` — the outer loop writes the counter's `log` between its
# inner loops, so it cannot hold the inner loop's whole-object write: each
# inner loop begins and ends its own, and the push in between finds none held.

class Counter:
    value: i32
    log: Array[int]

    fn bump(mut self):
        self.value = self.value + 1

fn main():
    counter = Counter(0, [])
    alias = counter
    for r in 0..3:
        for i in 0..2:
            counter.bump()
        alias.log.push(alias.value)
    println(alias.log)
