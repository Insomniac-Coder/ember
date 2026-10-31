#$ test: compile-fail
#$ rules: CLS-7a, OBJ-5
#$ profiles: debug, development, release

# A destructor must not publish its own counted handle into an owning
# container. The container could outlive the in-progress deinitialisation and
# resurrect the object after the runtime's pre-drop count check.
class Token:
    fn drop():
        retained: Array[Token] = Array[Token]()
        retained.push(this) #$ error[E3016]: `this` escapes its own `drop`

fn main():
    _token = Token()
