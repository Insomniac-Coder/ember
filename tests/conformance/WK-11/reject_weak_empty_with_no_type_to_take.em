#$ test: compile-fail
#$ rules: WK-11
# D-401 — where nothing says which `Weak` it is, `Weak.empty()` asks for its
# type.

fn main():
    w = Weak.empty()    #$ error[E2060]: cannot infer which `Weak` this is
