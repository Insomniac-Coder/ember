# Clause BCK-4.1: "A place whose storage ends (...) while a view of it is live is `E3060` at the
# view's later use" -- with no later use there is no error.
#$ test: run-pass
#$ rules: BCK-4, BRW-2
#$ stdout: abc
#$ stdout: xyz
#$ stdout: 5
#$ stdout: a

fn main():
    s = String("abc")
    w: StrView = s
    println(w)
    s = String("xyz")
    println(s)
    if true:
        x = 5
        r = ref x
        println(r)
    a = [String("a")]
    v = a[0]
    println(v)
    b = a
    println(b[0])
