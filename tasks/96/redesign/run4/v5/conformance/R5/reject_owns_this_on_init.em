#$ test: compile-fail
#$ rules: CLS-2
#$ help: write `fn init():`
## design rules: R5
## R5: `init` builds its object, so it cannot take it over: `owns this` on `init` is refused (E2108,
## `V5/rules.md` section 3's message; step 3's re-check: a refusal with no test). The nearest program
## that is allowed is `accept_init_sets_its_fields_with_no_word`.

class Door:
    angle: float

    fn init() owns this:       #$ error[E2108]: `init` builds the object, so it cannot take it over
        this.angle = 0.0

fn main():
    d = Door()
    println(d.angle)
