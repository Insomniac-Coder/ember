#$ test: compile-fail
#$ rules: CLS-7a, OBJ-5
#$ profiles: debug, development, release

# An `owned fn` owns its captured class handle and can outlive the destructor.
fn keep(token: Token):
    pass

class Token:
    fn drop():
        copy = this
        _task = owned fn() => keep(copy) #$ error[E3016]: `drop` ends this `Token`; an owner made from it would point at a freed object; keep a copy of what you need

fn main():
    _token = Token()
