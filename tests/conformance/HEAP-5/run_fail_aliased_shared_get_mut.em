#$ test: run-fail
#$ rules: HEAP-3, HEAP-5, HEAP-6, EXC-1, RC-5
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: overlapping access

struct Counter:
    value: i32

fn main():
    shared = Shared(Counter(1))
    alias = shared
    first = shared.get_mut()

    # The ordinary static borrow roots differ (`shared` versus `alias`), but
    # both handles designate one allocation. The second long-term mutation
    # must therefore trip the runtime exclusivity state.
    second = alias.get_mut()
    println(first.value + second.value)
