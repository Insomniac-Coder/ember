#$ test: compile-fail
#$ rules: LT-20, LT-21, LT-24, BRW-1, VERIFY-3, TST-17, TST-18

@view
struct Pair:
    selected: Span[i32]
    stable: Span[i32]

fn main():
    old: Array[i32] = Array[i32]()
    stable: Array[i32] = Array[i32]()
    replacement: Array[i32] = Array[i32]()
    pair = Pair(old.as_span(), stable.as_span())
    pair.selected = replacement.as_span()
    replacement.push(1) #$ error[E3021]: `replacement` is borrowed here and mutably borrowed elsewhere
    println(pair.selected.len())
