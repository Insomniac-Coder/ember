# Clause BRW-8.6: "The choice depends only on the declared first line, never on the body; it is an
# ABI decision and cannot be observed."
#$ test: run-pass
#$ rules: BRW-8, FN-1, OWN-7
#$ stdout: 1
#$ stdout: 1
# PENDING Q1 (INDEX.md): `Pair` fits in two pointers and goes in registers, `Quad` does not and is
# passed by address. Since the choice "cannot be observed", both functions must print the same;
# this file expects the value at the call (`[OWN-7]`: a `Copy` value is duplicated on use; `[FN-1]`:
# "a number is copied in"), the proposed ruling of Q1. `[FN-1]`'s "shows through `a`" read for every
# plain parameter would expect 9 for both, which no register passing can give.

@derive(Copy)
struct Pair:
    a: int
    b: int

@derive(Copy)
struct Quad:
    a: int
    b: int
    c: int
    d: int

static mut P: Pair = Pair(a=1, b=0)
static mut Q: Quad = Quad(a=1, b=0, c=0, d=0)

fn show_pair(p: Pair):
    P.a = 9
    println(p.a)

fn show_quad(q: Quad):
    Q.a = 9
    println(q.a)

fn main():
    show_pair(P)
    show_quad(Q)
