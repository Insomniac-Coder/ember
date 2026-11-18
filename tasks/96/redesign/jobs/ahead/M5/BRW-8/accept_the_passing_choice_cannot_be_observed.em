# Clause BRW-8.6: "The choice depends only on the declared first line, never on the body; it is an
# ABI decision and cannot be observed."
#$ test: run-pass
#$ rules: BRW-8, FN-1, BRW-21
#$ stdout: 9
#$ stdout: 9
# Q1, ruled by the lead (INDEX.md): a plain parameter reads the caller's place, and a number another
# name writes across the call shows through it (`[FN-1]`'s own words; `decisions.md` §4, area 2 Q4).
# `Pair` fits in two pointers, `Quad` does not; both print the number `P.a = 9` and `Q.a = 9` wrote,
# since a copy in registers is the compiler's choice only where nothing can observe it. (How that sits with
# "depends only on the declared first line" is Q14.)

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
