#$ test: run-pass
#$ rules: LT-17, LT-20, LT-21, LT-24, VERIFY-3, TST-17, TST-18
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
    first: Array[i32] = Array[i32]()
    first.push(30)
    second: Array[i32] = Array[i32]()
    second.push(40)
    pair = Pair(old.as_span(), stable.as_span())
    use_first = true
    if use_first:
        pair.selected = first.as_span()
    else:
        pair.selected = second.as_span()
    old.push(11)
    println(pair.selected[0])
