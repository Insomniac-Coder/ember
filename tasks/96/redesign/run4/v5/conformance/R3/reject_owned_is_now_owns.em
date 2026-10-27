#$ test: compile-fail
#$ rules: MOD-3
#$ help: write `fn take(owns x: String):`
## design rules: R3, R4
## R3 and the removed words of `V4/rules.md` section 3: the mode `owned` is written `owns` (E1010; the `owned`
## row of `V4/vocabulary.md`; completeness gap G8). The nearest program that is allowed is
## `R1/accept_an_object_read_before_it_is_handed_over`.

fn take(owned x: String):      #$ error[E1010]: `owned` is now `owns`
    println(x)

fn main():
    take(String("a"))
