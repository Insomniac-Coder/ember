#$ test: compile-fail
#$ rules: GRM-41, FN-4
## `[GRM-41]` — a method does not list its object among its parameters (`self` anywhere is `E1010`); `this`
## where there is no object is `E1010`; `static fn` is a function of a type body is `E2115`.

struct Counter:
    n: int = 0

    fn bump(self):    #$ error[E1010]: inside a method its object is `this`
        this.n += 1

    static fn make() -> Counter:
        return Counter(n=this.n)    #$ error[E1010]: `this` can only be used in a method

static fn helper() -> int:    #$ error[E2115]: `static fn` is a function of a type body
    return 1

fn free() -> int:
    return this.n    #$ error[E1010]: `this` can only be used in a method

fn main():
    println(helper(), free())
