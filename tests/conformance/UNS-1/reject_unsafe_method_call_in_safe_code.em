#$ test: compile-fail
#$ rules: UNS-1, TIER-1
#$ profiles: debug
#$ error[E3100]: `counter.next()` (line 15) calls an `unsafe fn`, whose callers must make true what its `@safety` says

struct Counter:
    value: i32

extend Counter:
    unsafe fn next(self) -> i32:  # SAFETY: this test sets up what the operation requires
        return self.value + 1

fn main():
    counter = Counter(41)
    println(counter.next())
