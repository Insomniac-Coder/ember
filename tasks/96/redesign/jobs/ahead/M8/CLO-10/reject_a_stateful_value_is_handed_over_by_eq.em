# Clause CLO-10.2: "a function value with state is not, and `=` hands it over"
#$ test: compile-fail
#$ rules: CLO-10, OWN-3
# No text in the spec for this program: code only.

fn main():
    k = 3
    h = fn() copy of k => k + 1
    h2 = h
    println(h2())
    println(h())    #$ error[E3040]
