#$ test: compile-fail
#$ rules: R14
## R14 and section 3's removed words: `as?` is a kind test, `as!` is `cast x as D` (AN-67, the
## vocabulary's `as?` and `as!` rows). Two mistakes, two messages.

interface Shape:
    fn size() -> int

struct Circle implements Shape:
    r: int

    fn size() -> int:
        return this.r

fn maybe(s: dyn Shape) -> int:
    a = s as? Circle    #$ error[E1010]: a downcast that may fail is a kind test: instead of `a = s as? Circle` and `if a exists as c:`, write `if s is Circle as c:`, keeping its block
    if a exists as c:
        return c.r
    return 0

fn sure(s: dyn Shape) -> int:
    a = s as! Circle    #$ error[E1010]: `as!` is not Ember: to stop the program when `s` is not a `Circle`, write `a = cast s as Circle`
    return a.r

fn main():
    println(maybe(Circle(r=1)), sure(Circle(r=2)))
