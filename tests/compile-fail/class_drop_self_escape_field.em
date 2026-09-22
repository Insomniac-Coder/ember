#$ test: compile-fail
#$ rules: CLS-7a, OBJ-5
#$ profiles: debug, release, shipping

# A destructor may not assign its handle into another object's field either.
class Holder:
    saved: Token

class Token:
    fn drop(mut self):
        holder = Holder(Token())
        holder.saved = self #$ error[E3016]: `self` escapes its own drop

fn main():
    token = Token()
