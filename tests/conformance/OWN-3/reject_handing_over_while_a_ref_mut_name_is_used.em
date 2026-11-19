#$ test: compile-fail
#$ rules: OWN-3

fn main():
    s = String("a")
    m = ref mut s
    t = s            #$ error[E3022]
    m.push_str("b")
    println(t)
