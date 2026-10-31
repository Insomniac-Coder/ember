#$ test: run-pass
#$ rules: MNG-1, MOD-5
#$ profiles: development, release
#$ stdout: 7 3 2
#$ assert-c: contains("em_3std4core4char13is_alphabetic(")
#$ assert-c: contains("int64_t em_7StrView13is_alphabetic(")
# `[MNG-1]` (ODR-215) — a built-in type's path is under `std.core`: the built-in `char`'s method
# `is_alphabetic` is `std.core.char.is_alphabetic` (`em_3std4core4char13is_alphabetic`), and a
# program's own `struct StrView` (`[MOD-5]`: the program's type is what the name means in it) has
# `StrView.is_alphabetic` (`em_7StrView13is_alphabetic`), the root package having no component of its
# own. A built-in's methods were `em_4char…` and `em_3str…`, and the program's own type of a prelude
# name was kept apart only by a `root` component its path does not have.

struct StrView:
    n: int

    fn is_alphabetic() -> int:
        return this.n + 1

fn letters(text: Array[char]) -> int:
    n = 0
    for c in text:
        if c.is_alphabetic():
            n += 1
    return n

fn main():
    mine = StrView(6)
    println(mine.is_alphabetic(), letters(['a', 'b', '1', 'c']), letters(['x', '2', 'y']))
