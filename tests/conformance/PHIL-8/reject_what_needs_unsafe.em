#$ test: compile-fail
#$ rules: PHIL-8, UNS-1
#$ error[E3100]: `alloc` needs an `unsafe` block
#$ help: wrap the call in `unsafe:`
# `[PHIL-8]` — an operation that neither a static proof nor a run-time check can make safe requires
# `unsafe`, and the diagnostic names the way to write it.

fn main():
    p: *mut i64 = alloc[i64](1)
