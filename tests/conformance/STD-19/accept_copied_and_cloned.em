#$ test: run-pass
#$ rules: STD-19
#$ profiles: debug, development, release
#$ stdout:
#$ [4, 9, 2, 1] [9, 2]
#$ ['a', 'b'] 2
#$ [5]
# `[STD-19]` — `copied()` and `cloned()` over an iterator of references give
# the values those reach, so the items are values a program may keep: a
# list's copied items make a new array it can push to. On an `Iterable` they
# borrow it through `iter()` like the other adapters.

fn main():
    xs: Array[int] = [4, 9, 2]
    ys = xs.iter().copy_each().to_array()
    ys.push(owns 1)
    println(ys, xs.copy_each().skip(1).to_array())
    words: Array[String] = [String.from("a"), String.from("b")]
    ws = words.iter().copy_each().to_array()
    println(ws, words.copy_each().count())
    m: Map[StrView, int] = {"k": 5}
    println(m.values().copy_each().to_array())
