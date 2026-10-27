#$ test: compile-fail
#$ rules: MOD-3
#$ help: write `fn drop():` in place of line 12 and `println("end", this.name)` in place of line 13
## design rules: R5
## R5 and the removed words of `V5/rules.md` section 3: `fn drop(mut self)` is written `fn drop():`; a
## `drop` reaches the object it ends as `this`, with no word (E1010; the `mut self` row of
## `V5/vocabulary.md`, `[DRP-1]`; step 3's re-check: a refusal with no test). The nearest program that
## is allowed is `accept_a_drop_that_changes_nothing_outside_writes_no_changes`.

class Res:
    name: String

    fn drop(mut self):         #$ error[E1010]: `mut self` is not Ember: a `drop` reaches the object it ends as `this`, with no word
        println("end", self.name)

fn main():
    r = Res(name=String("a"))
    println("made")
