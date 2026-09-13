#$ test: run-pass
#$ rules: LT-20, LT-21, LT-24, VERIFY-3, TST-17
#$ stdout: 30

@view
struct Pair:
    selected: Span[i32]
    stable: Span[i32]

fn main():
    old: Array[i32] = Array[i32]()
    old.push(10)
    stable: Array[i32] = Array[i32]()
    stable.push(20)
    replacement: Array[i32] = Array[i32]()
    replacement.push(30)
    pair = Pair(old.as_span(), stable.as_span())
    pair.selected = replacement.as_span()
    old.push(11)
    println(pair.selected[0])
