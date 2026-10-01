#$ test: run-pass
#$ rules: TYP-18, TYP-15
#$ stdout: ['a', 'b'] [7] 0
# `[TYP-18]` — a constructor's explicit type arguments fix the instantiation,
# whatever else the context says: `Array[str]()` holds `str` with nothing to
# annotate (a container at a view type is a type like any other, ODR-069),
# `Array[i32]()` holds `i32` (F-029), and `String()` takes none (D-456).

fn main():
    words = Array[str]()
    words.push("a")
    words.push("b")
    xs = Array[i32]()
    xs.push(7)
    s = String()
    println(words, xs, s.len())
