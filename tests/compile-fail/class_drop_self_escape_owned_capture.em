#$ test: compile-fail
#$ rules: CLS-7a, OBJ-5
#$ profiles: debug, release, shipping

# An `owned fn` owns its captured class handle and can outlive the destructor.
fn keep(token: Token):
    pass

class Token:
    fn drop(mut self):
        copy = self
        task = owned fn() => keep(copy) #$ error[E3016]: `self` escapes its own drop

fn main():
    token = Token()
