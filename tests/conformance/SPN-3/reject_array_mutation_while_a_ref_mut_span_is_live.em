#$ test: compile-fail
#$ rules: SPN-1, SPN-3, BRW-1, BRW-2
#$ error[E3022]: `values` is already mutably borrowed

# `[SPN-3]`, `[BRW-1]` — a `ref mut` name of a span made from a list keeps the list's provenance: the
# list is not changed through another name while the span is used.

fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 1)
    child = ref mut values.as_mut_span()
    values.push(owns 2)
    println(child[0])
