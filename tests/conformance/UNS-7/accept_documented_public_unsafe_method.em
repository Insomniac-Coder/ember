#$ test: compile-pass
#$ rules: UNS-7
#$ profiles: debug

struct Counter:
    value: i32

extend Counter:
    @safety("The caller guarantees the counter is live.")
    pub unsafe fn next() -> i32:
        return this.value + 1

fn main():
    pass
