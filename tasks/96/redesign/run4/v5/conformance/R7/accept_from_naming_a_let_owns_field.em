#$ test: run-pass
#$ rules: R7, R3
#$ stdout: x
## R7: `from` may name a `let` field of type `owns T`, whose value lives in a block of its own, so moving
## the object never moves it (`V5/changes.md` section 8, item 9). The nearest program to
## `reject_from_naming_an_inline_struct_field.em` that is allowed, with only the line its message gives
## changed: `init` hands its `pair` to the `owns` field by an ordinary `=` (R3: no `owns` before a name
## that already exists).

struct Pair:
    a: String
    b: String

class Entry:
    let pair: owns Pair
    first: StrView from pair

    fn init(owns pair: Pair):
        this.pair = pair
        this.first = this.pair.a

fn main():
    e = Entry(Pair(a=String("x"), b=String("y")))
    println(e.first)
