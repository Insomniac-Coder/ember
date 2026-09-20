#$ test: run-fail
#$ rules: HEAP-4, HEAP-5, HEAP-6, EXC-1, RC-5
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: overlapping access

struct Counter:
    value: i32

fn borrow_counter(mut owner: Shared[Counter]) -> ref Counter:
    return owner.get()

fn main():
    shared = Shared(Counter(1))
    alias = shared
    reader = borrow_counter(shared)
    writer = alias.get_mut()
    println(reader.value + writer.value)
