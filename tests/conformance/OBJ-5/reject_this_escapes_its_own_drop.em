#$ test: compile-fail
#$ rules: OBJ-5, CLS-7a
#$ help: on line 17, hand over a copy of what you need where it has `this`: `copy of this.name`
# `[OBJ-5]` — an ended object is never brought back: storing `this` anywhere that outlives the
# call, inside `drop`, is `E3016`, and the message is the rule's: "`drop` ends this `Token`; an
# owner made from it would point at a freed object; keep a copy of what you need", with the class's
# name, and the line it is on.

class Token:
    name: String

    fn init(owns name: String):
        this.name = name

    fn drop():
        retained: Array[Token] = Array[Token]()
        retained.push(owns this)    #$ error[E3016]: `drop` ends this `Token`; an owner made from it would point at a freed object; keep a copy of what you need

fn main():
    _token = Token("a")
