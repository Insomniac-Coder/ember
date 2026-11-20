# Clause BRW-11.4: "Where the compiler proves when compiling that two `ref mut` arguments, or a
# `ref mut` argument and a variable the callee uses by its own name (a module variable), are one
# whole variable, the call is allowed and compiled against a copy of the function in which the two
# names are one."
#$ test: run-pass
#$ rules: BRW-11, MONO-10, BRW-22, BCK-5
#$ stdout: 2
#$ stdout: 1
#$ stdout: renamed
#$ stdout: 1 1
# The spec's two programs: `add_both` prints 2, and `rename(ref mut LOG)` leaves `LOG` holding one
# string. Other calls use the ordinary body (`add_both(ref mut x, ref mut y)`).

static mut LOG: Array[String] = []

fn add_both(ref mut a: int, ref mut b: int):
    a += 1
    b += 1
    println(a)

fn rename(ref mut s: Array[String]):
    LOG.clear()
    s = [String("renamed")]

fn main():
    x = 0
    add_both(ref mut x, ref mut x)
    LOG.push(owns String("a"))
    LOG.push(owns String("b"))
    rename(ref mut LOG)
    println(LOG.len())
    println(LOG[0])
    p = 0
    q = 0
    add_both(ref mut p, ref mut q)
    println(p, q)
