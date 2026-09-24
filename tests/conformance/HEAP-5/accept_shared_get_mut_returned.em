#$ test: run-pass
#$ rules: HEAP-3, HEAP-5, HEAP-6, RC-5, EXC-1, EXC-2
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(ember_access_begin_write)
#$ assert-c: contains(ember_access_end_write)

struct Counter:
    value: i32

@borrows(owner)
fn borrow_counter(mut owner: Shared[Counter]) -> ref mut Counter:
    return owner.get_mut()

fn main():
    shared = Shared(Counter(40))
    alias = shared
    first = borrow_counter(shared)
    first.value = first.value + 1
    # The returned reference's dynamic access transfers to this caller and
    # ends at `first`'s last use, so the aliased handle can be borrowed next.
    second = alias.get_mut()
    second.value = second.value + 1
    println(second.value)
