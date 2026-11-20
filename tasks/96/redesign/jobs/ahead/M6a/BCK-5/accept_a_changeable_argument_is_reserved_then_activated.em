# Clause BCK-5.1: "A `ref mut` argument, and the object of a method that may change it, is
# *reserved* while the call's other arguments are evaluated and *activated* when the call starts;
# during reservation it conflicts only with a change of its place (`[BRW-3]`), so `v.push(owns
# v.len())` is accepted."
#$ test: run-pass
#$ rules: BCK-5, BRW-3
#$ stdout: 3
#$ stdout: 4
#$ stdout: 2
# The place is read while the other arguments are evaluated: the spec's `v.push(owns v.len())`, a
# `ref mut` argument beside a number read from it, and a method's object beside a value computed
# from it.

struct Counter:
    n: int = 0

    fn add(k: int):
        this.n += k

fn add_to(ref mut xs: Array[int], x: int):
    xs.push(owns x)

fn main():
    v = [1, 2]
    v.push(owns v.len())
    println(v.len())
    add_to(ref mut v, v[0])
    println(v.len())
    c = Counter()
    c.add(1)
    c.add(c.n)
    println(c.n)
