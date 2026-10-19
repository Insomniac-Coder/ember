#$ test: compile-fail
#$ rules: LAY-2
#$ profiles: debug
# `[LAY-2]` — a derived `hash` reads each field of a `@packed` type from an aligned copy; a field
# passed by address (a `String`) cannot be copied, and borrowing it in place would take a
# reference to a field that may be unaligned, so it is `E2170` at the field.

@packed
@derive(Hash)
struct Key:
    id: u8
    count: u64
    name: String  #$ error[E2170]: `@derive(Hash)` borrows field `name` of a `@packed` type

fn main():
    k = Key(id = 1, count = 2, name = "k")
    println(k.id)
