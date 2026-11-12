#$ test: compile-fail
#$ rules: CLS-7a, OBJ-5
#$ profiles: debug, development, release

# A destructor must not publish its own counted handle into an owning
# container. The container could outlive the in-progress deinitialisation and
# resurrect the object after the runtime's pre-drop count check.
class Token:
    fn drop():
        retained: Array[Token] = Array[Token]()
        retained.push(owns this) #$ error[E3016]: `drop` ends this `Token`; an owner made from it would point at a freed object; keep a copy of what you need

fn main():
    _token = Token()
