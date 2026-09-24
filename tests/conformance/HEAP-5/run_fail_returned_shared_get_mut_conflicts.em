#$ test: run-fail
#$ rules: HEAP-3, HEAP-5, HEAP-6, EXC-1, RC-5
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: overlapping access

struct Counter:
    value: i32

@borrows(owner)
fn borrow_counter(mut owner: Shared[Counter]) -> ref mut Counter:
    return owner.get_mut()

fn main():
    shared = Shared(Counter(1))
    alias = shared
    first = borrow_counter(shared)
    # `first` remains live below, so this starts a conflicting dynamic write
    # access through a distinct strong handle to the same allocation.
    second = alias.get_mut()
    println(first.value + second.value)
