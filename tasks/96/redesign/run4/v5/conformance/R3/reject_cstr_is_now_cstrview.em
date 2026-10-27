#$ test: compile-fail
#$ rules: MOD-3
#$ help: write `name: CStrView`
## design rules: R3
## R3 and the removed words of `V4/rules.md` section 3: `cstr` is written `CStrView` (E1010; the `cstr` row of
## `V4/vocabulary.md`; completeness gap G8).

struct Label:
    name: cstr                 #$ error[E1010]: `cstr` is now `CStrView`, a view of a C string
    width: int

fn main():
    println(0)
