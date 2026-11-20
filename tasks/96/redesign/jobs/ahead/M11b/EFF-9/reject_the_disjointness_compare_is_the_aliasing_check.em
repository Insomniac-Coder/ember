# Clause EFF-9.1: "`Aliasing` covers the compare of `assert_disjoint` and `check_disjoint`
# (`[DSJ-1]`) (...), the two run-time alias checks there are"
#$ test: compile-fail
#$ rules: EFF-9, EFF-12, DSJ-1
# `check_disjoint` compares two spans of one map while running: an `Aliasing` check, which
# `@no_alias_checks` forbids. `assert_disjoint` with its message is EFF-12's file. No text in the
# spec for `check_disjoint`: code only.

from std.mem import check_disjoint

@no_alias_checks
fn same(ref mut tracks: Map[String, Array[float]], a: StrView, b: StrView) -> bool:
    d = ref mut tracks[a].as_mut_span()
    s = tracks[b].as_span()
    return check_disjoint(d, s)    #$ error[E4030]

fn main():
    tracks: Map[String, Array[float]] = Map()
    tracks.insert(owns String("a"), owns [1.0])
    tracks.insert(owns String("b"), owns [2.0])
    println(same(ref mut tracks, "a", "b"))
