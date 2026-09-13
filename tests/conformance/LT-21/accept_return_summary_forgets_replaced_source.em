#$ test: run-pass
#$ rules: LT-20, LT-21, LT-22, LT-24, LT-35, LT-40, VERIFY-3, TST-17, TST-19
#$ stdout: 30

@view
struct Pair:
    selected: Span[i32]
    stable: Span[i32]

fn replace_selected(old: Span[i32], stable: Span[i32], replacement: Span[i32]) -> Pair:
    pair = Pair(old, stable)
    pair.selected = replacement
    return pair

fn main():
    old: Array[i32] = Array[i32]()
    old.push(10)
    stable: Array[i32] = Array[i32]()
    stable.push(20)
    replacement: Array[i32] = Array[i32]()
    replacement.push(30)
    pair = replace_selected(old.as_span(), stable.as_span(), replacement.as_span())
    old.push(11)
    println(pair.selected[0])
