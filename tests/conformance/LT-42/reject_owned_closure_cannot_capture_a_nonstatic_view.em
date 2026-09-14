#$ test: compile-fail
#$ rules: CLO-1, CLO-2, LT-42, TYP-15, LT-3, TST-19

# Owning a local Span in an `owned fn` would permit the closure to outlive its
# Array source. This is the ordinary all-slots static storage boundary, not a
# special exemption for compiler-generated closure environments.

@view
struct Pair:
    values: Span[i32]

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    pair = Pair(values.as_span())
    task = owned fn() => pair.values[0] #$ error[E3063]: `owned fn` captures `Pair` by value, but that view is not static
    println(task())
