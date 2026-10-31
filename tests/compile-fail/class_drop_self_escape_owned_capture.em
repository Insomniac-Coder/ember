#$ test: compile-fail
#$ rules: CLS-7a, OBJ-5
#$ profiles: debug, development, release

# An `owned fn` owns its captured class handle and can outlive the destructor.
fn keep(token: Token):
    pass

class Token:
    fn drop():
        copy = this
        _task = owned fn() => keep(copy) #$ error[E3016]: `this` escapes its own `drop`

fn main():
    _token = Token()
