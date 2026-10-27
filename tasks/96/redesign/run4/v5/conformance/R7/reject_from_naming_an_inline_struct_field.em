#$ test: compile-fail
#$ rules: R7
#$ help: let pair: owns Pair
## R7: `from` names a `let` field whose contents live in their own heap block; a struct stored inline
## moves with its object, so a `from` naming one is refused (E2118), and the message gives the field
## with a block of its own (rules section 3's E2118 message word for word; settled in step 4,
## `V5/changes.md` section 8, item 9). The allowed neighbour is `accept_from_naming_a_let_owns_field.em`.

struct Pair:
    a: String
    b: String

class Entry:
    let pair: Pair
    first: StrView from pair    #$ error[E2118]: `pair` is stored inside the object, so moving the object moves it; keep it in a block of its own: write the field as `let pair: owns Pair`

    fn init(owns pair: Pair):
        this.pair = pair
        this.first = this.pair.a

fn main():
    e = Entry(Pair(a=String("x"), b=String("y")))
    println(e.first)
