#$ test: run-fail
#$ rules: HEAP-4, HEAP-5, HEAP-6, EXC-1, RC-5
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: overlapping access

struct Counter:
    value: i32

fn main():
    shared = Shared(Counter(1))
    alias = shared
    reader = shared.get()
    # The reader is live below. A strong-handle alias must therefore fail the
    # same reader/writer dynamic-exclusivity check used by class access.
    writer = alias.get_mut()
    println(reader.value + writer.value)
