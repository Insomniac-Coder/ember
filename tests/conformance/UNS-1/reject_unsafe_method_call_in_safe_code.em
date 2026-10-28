#$ test: compile-fail
#$ rules: UNS-1, TIER-1
#$ profiles: debug
#$ error[E3100]: `counter.next()` (line 16) calls an `unsafe fn`, whose callers must make true what its `@safety` says

struct Counter:
    value: i32

extend Counter:
    @safety("this test sets up what the operation requires")
    unsafe fn next(self) -> i32:
        return self.value + 1

fn main():
    counter = Counter(41)
    println(counter.next())
