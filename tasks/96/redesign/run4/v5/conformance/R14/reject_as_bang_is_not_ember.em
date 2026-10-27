#$ test: compile-fail
#$ rules: R14
## R14 and section 3's removed words: `as!` is `cast x as D` (AN-67, the vocabulary's `as!` row). Split
## from step 3's `reject_as_question_and_as_bang.em`, which had two faults (step 3's re-check, wrong 2);
## the nearest allowed program is `accept_the_kind_test_and_the_sure_cast_replace_the_old_forms.em`.

interface Shape:
    fn size() -> int

struct Circle implements Shape:
    r: int

    fn size() -> int:
        return this.r

fn sure(s: dyn Shape) -> int:
    a = s as! Circle    #$ error[E1010]: `as!` is not Ember: to stop the program when `s` is not a `Circle`, write `a = cast s as Circle` in place of line 17
    return a.r

fn main():
    println(sure(Circle(r=2)))
