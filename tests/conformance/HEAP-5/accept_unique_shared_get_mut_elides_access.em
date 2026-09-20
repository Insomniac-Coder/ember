#$ test: run-pass
#$ rules: HEAP-3, HEAP-5, HEAP-6, EXC-3, EXC-3a
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: !contains(ember_access_begin_write)
#$ assert-c: !contains(ember_access_end_write)

struct Counter:
    value: i32

fn main():
    shared = Shared(Counter(40))
    value = shared.get_mut()
    value.value = value.value + 2
    println(value.value)
