#$ test: compile-fail
#$ rules: CLS-7a, OBJ-5
#$ profiles: debug, release, shipping

# A destructor must not publish its own counted handle into an owning
# container. The container could outlive the in-progress deinitialisation and
# resurrect the object after the runtime's pre-drop count check.
class Token:
    fn drop(mut self):
        retained: Array[Token] = Array[Token]()
        retained.push(self) #$ error[E3016]: `self` escapes its own drop

fn main():
    token = Token()
