#$ test: compile-fail
#$ rules: LT-17, LT-20, LT-24, LT-36, LT-38, TST-17

# In contrast, binding `pair` moves the whole payload. `[LT-36]` requires
# every carried region at that move, so a prior mutation of `right` remains
# rejected even though the arm body happens to project only `pair.left`.

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]

fn main():
    left: Array[i32] = Array[i32]()
    left.push(7)
    right: Array[i32] = Array[i32]()
    right.push(8)
    maybe: Option[Pair] = Some(Pair(left.as_span(), right.as_span()))
    right.push(9) #$ error[E3021]: `right` is borrowed here and mutably borrowed elsewhere
    match maybe:
        Some(pair):
            println(pair.left[0])
        None:
            pass
