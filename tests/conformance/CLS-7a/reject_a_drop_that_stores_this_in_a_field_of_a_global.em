#$ test: compile-fail
#$ rules: CLS-7a, OBJ-5
#$ help: on line 15, hand over a copy of what you need where it has `this`: `copy of this.id`
# `[CLS-7a]` — inside `drop`, `this` MUST NOT be stored anywhere that outlives the call: `E3016`,
# with `[OBJ-5]`'s message, naming the class.

class Token:
    id: int

    fn init(id: int):
        this.id = id

    fn drop():
        keep: Array[Token] = Array[Token]()
        keep.push(owns this)        #$ error[E3016]: `drop` ends this `Token`; an owner made from it would point at a freed object; keep a copy of what you need

fn main():
    _token = Token(1)
