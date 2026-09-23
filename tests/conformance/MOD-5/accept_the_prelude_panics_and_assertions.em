#$ test: run-pass
#$ rules: MOD-5
#$ profiles: debug, release, shipping
#$ stdout: ok
#$ 5 0
# The prelude's assertion functions, its `mem` module, and standard error.

fn main():
    x = 3
    assert(x == 3)
    assert(x > 0, "x is positive")
    assert_eq(x + 2, 5)
    assert_ne("a", "b")
    debug_assert(x < 10)
    eprintln("this line goes to standard error, not the expected output")
    println("ok")
    total = 5
    old = mem.replace(total, 0)
    println(old, total)
