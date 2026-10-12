#$ test: compile-pass
#$ rules: UNS-7
#$ profiles: debug

struct Counter:
    value: i32

extend Counter:
    @safety("The caller guarantees the counter is live.")
    pub unsafe fn next(self) -> i32:  # SAFETY: this test sets up what the operation requires
        return self.value + 1

fn main():
    pass
