#$ test: run-pass
#$ rules: UNS-1, TIER-1
#$ profiles: debug, release, shipping
#$ stdout: 42

struct Counter:
    value: i32

extend Counter:
    unsafe fn next(self) -> i32:  # SAFETY: this test sets up what the operation requires
        return self.value + 1

fn main():
    counter = Counter(41)
    unsafe:  # SAFETY: this test sets up what the operation requires
        println(counter.next())
