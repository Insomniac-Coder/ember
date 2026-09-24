#$ test: run-fail
#$ rules: HEAP-3, HEAP-5, HEAP-6, EXC-1, RC-5
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: overlapping access

struct Counter:
    value: i32

@borrows(owner)
fn inner(mut owner: Shared[Counter]) -> ref mut Counter:
    return owner.get_mut()

@borrows(owner)
fn outer(mut owner: Shared[Counter]) -> ref mut Counter:
    return inner(owner)

fn main():
    shared = Shared(Counter(1))
    alias = shared
    first = outer(shared)
    second = alias.get_mut()
    println(first.value + second.value)
