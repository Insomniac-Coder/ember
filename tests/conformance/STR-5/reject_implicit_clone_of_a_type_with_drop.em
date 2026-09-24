#$ test: compile-fail
#$ rules: STR-5, OWN-8, DRP-1
#$ help: write `fn clone(self) -> Guard`
# ODR-026 — a type with its own `drop` manages something a field-wise copy
# would release twice, so it is `Clone` only when it says so.

struct Guard:
    id: int

    fn drop(mut self):
        pass

fn main():
    g = Guard(id=1)
    h = g.clone()    #$ error[E1010]: `Guard` has no method named `clone`
