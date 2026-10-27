#$ test: compile-fail
#$ rules: MOD-3
#$ help: write `fn first(s: StrView) -> StrView:`; for text of your own write `String`
## design rules: R3
## R3 and the removed words of `V4/rules.md` section 3: `str` is written `StrView`, a view into someone
## else's text (E1010; the `str` row of `V4/vocabulary.md`; completeness gap G8). The nearest program
## that is allowed is `R2/accept_a_view_of_your_own_made_from_a_parameter`.

fn first(s: str) -> str:       #$ error[E1010]: `str` is now `StrView`, a view into someone else's text
    return s.trim()

fn main():
    println(first(" a "))
