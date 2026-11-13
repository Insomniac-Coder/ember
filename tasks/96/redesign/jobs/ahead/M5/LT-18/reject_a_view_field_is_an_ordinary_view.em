# Clause LT-18.1: "Every view field is an ordinary view under `[BRW-1]` to `[BRW-22]`"
#$ test: compile-fail
#$ rules: LT-18, BRW-1, BRW-17, LT-17, LT-46
# Each field is checked as a view of its own would be: a change of what it reads (`E3021`, at the
# change), a reshape of what it points into (`E3060`, at its later use), and another name for what
# a `ref mut` field names (`E3021`, at the read). Codes only.

@view
struct Pair:
    left: Span[int]
    right: Span[int]

struct Cursor:
    at: ref mut int

fn changed():
    a = [1]
    b = [2]
    pair = Pair(left=a.as_span(), right=b.as_span())
    a[0] = 9    #$ error[E3021]
    println(pair.left[0], pair.right[0])

fn reshaped():
    a = [1]
    b = [2]
    pair = Pair(left=a.as_span(), right=b.as_span())
    a.push(owns 3)
    println(pair.left[0])    #$ error[E3060]
    println(pair.right[0])

fn read_beside_a_ref_mut_field():
    x = 1
    c = Cursor(at=ref mut x)
    println(x)    #$ error[E3021]
    c.at += 1

fn main():
    changed()
    reshaped()
    read_beside_a_ref_mut_field()
