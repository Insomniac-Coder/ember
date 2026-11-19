#$ test: compile-fail
#$ rules: OWN-3, BRW-17

fn main():
    s = String("a")
    v = ref s
    t = s
    println(v, t)    #$ error[E3060]
