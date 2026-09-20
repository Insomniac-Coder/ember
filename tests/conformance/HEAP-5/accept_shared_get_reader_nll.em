#$ test: run-pass
#$ rules: HEAP-4, HEAP-5, HEAP-6, RC-5, EXC-1, EXC-2
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(ember_access_begin_read)
#$ assert-c: contains(ember_access_end_read)
#$ assert-c: contains(ember_access_begin_write)

struct Counter:
    value: i32

fn main():
    shared = Shared(Counter(40))
    alias = shared
    reader = shared.get()
    seen = reader.value
    # `reader` is dead after `seen`, so the reader interval ends before this
    # writer interval begins through the aliased strong handle.
    writer = alias.get_mut()
    writer.value = seen + 2
    println(writer.value)
