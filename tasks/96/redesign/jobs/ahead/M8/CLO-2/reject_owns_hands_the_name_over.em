# Clause CLO-2.2: "`owns` handed over, for every type, numbers included (after `fn() owns count`, the
# function that made the lambda no longer uses `count`, `E3040`"
#$ test: compile-fail
#$ rules: CLO-2, OWN-3
# No text in the spec for this program: code only.

fn main():
    count = 5
    f = fn() owns count => count + 1
    println(f())
    println(count)    #$ error[E3040]
