#$ test: run-pass
#$ rules: TYP-47, MONO-1, MNG-1, MNG-6
#$ profiles: debug, development, release
#$ stdout: 2 2 3 0
#$ int x
#$ int x
#$ x int
#$ assert-c: contains("em_5namesGdc93d5271667ecfd6638e250d3f5fb2d(int64_t")
#$ assert-c: contains("em_5namesG2e31469970f7765ff2dbb214f24bc68d(const unsigned char*")
#$ assert-c: contains("em_5countGd53d18c23212ea7b6300594bb89bce60(")
# `[TYP-47]`, `[MONO-1]` — each use of a function over a pack is made for its exact types, as any
# generic is: one instance per distinct list of types (two calls with `(int, StrView)` share one,
# and `(StrView, int)` is another), each named by `[MNG-1]` from the canonical spelling of its
# arguments, the pack's types each in its place (`[MNG-6]`): `[i64, StrView]`, `[StrView, i64]`
# and `[]` hash to the three symbols asserted. Nothing is looked up while the program runs.

[Ts...]
fn count(values: Ts...) -> int:
    return values.len()

extend int implements Named:
    fn named() -> StrView:
        return "int"

extend StrView implements Named:
    fn named() -> StrView:
        return "x"

interface Named:
    fn named() -> StrView

[Ts...: needs Named]
fn names(values: Ts...):
    first = true
    for v in values:
        if not first:
            print(" ", end="")
        print(v.named(), end="")
        first = false
    println()

fn main():
    println(count(1, "a"), count(2, "b"), count(1, 2, 3), count())
    names(1, "a")
    names(2, "b")
    names("c", 3)
