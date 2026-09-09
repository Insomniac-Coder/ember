#$ test: run-pass
#$ rules: TYP-5, RNG-2
## "A value of a range type `T` over representation `R` coerces to `R`" — at
## an assignment, an argument, a return, and a field initialiser.

type Percent = u8 in 0 ..= 100

struct Holder:
    value: u8

fn takes(x: u8) -> u8: return x

fn gives(p: Percent) -> u8:
    return p

fn main():
    p: Percent = 7
    a: u8 = p
    h = Holder(p)
    print(a)
    print(takes(p))
    print(gives(p))
    print(h.value)
#$ stdout: 7777
