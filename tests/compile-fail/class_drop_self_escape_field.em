#$ test: compile-fail
#$ rules: CLS-7a, OBJ-5
#$ profiles: debug, development, release

# A destructor may not assign its handle into another object's field either.
class Holder:
    saved: Token

class Token:
    fn drop():
        _holder = Holder(Token())
        _holder.saved = this #$ error[E3016]: `this` escapes its own `drop`

fn main():
    _token = Token()
