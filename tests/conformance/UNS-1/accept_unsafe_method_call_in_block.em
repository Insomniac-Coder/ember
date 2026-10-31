#$ test: run-pass
#$ rules: UNS-1, TIER-1
#$ profiles: debug, development, release
#$ stdout: 42

struct Counter:
    value: i32

extend Counter:
    @safety("this test sets up what the operation requires")
    unsafe fn next() -> i32:
        return this.value + 1

fn main():
    counter = Counter(41)
    unsafe:  # SAFETY: this test sets up what the operation requires
        println(counter.next())
