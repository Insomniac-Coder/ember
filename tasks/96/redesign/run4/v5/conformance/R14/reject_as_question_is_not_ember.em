#$ test: compile-fail
#$ rules: MOD-3
## design rules: R14
## R14 and section 3's removed words: `as?` is a kind test (AN-67, the vocabulary's `as?` row). Split
## from step 3's `reject_as_question_and_as_bang.em`, which had two faults (step 3's re-check, wrong 2);
## the nearest allowed program is `accept_the_kind_test_and_the_sure_cast_replace_the_old_forms.em`.

interface Shape:
    fn size() -> int

struct Circle implements Shape:
    r: int

    fn size() -> int:
        return this.r

fn maybe(s: dyn Shape) -> int:
    a = s as? Circle    #$ error[E1010]: a downcast that may fail is a kind test: instead of `a = s as? Circle` and `if a exists as c:` (lines 17 and 18), write `if s is Circle as c:`, keeping its block
    if a exists as c:
        return c.r
    return 0

fn main():
    println(maybe(Circle(r=1)))
