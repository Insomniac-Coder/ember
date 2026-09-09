#$ test: compile-fail
#$ rules: OWN-4, DIA-7a
# "A loop body that moves a value declared outside the loop is `E3041` unless
# the value is reassigned before the next iteration on every path."
#
# This was reported as `E3040` — shape O1, "use after move" — whose help is
# "clone it, or borrow it instead". Correct advice for a different program.
# `[DIA-7a]` keys `E3041` to shape O3, "move in a loop", and its help is about
# the *next iteration*, which is where the problem actually is.
#
# The two are told apart by what the analysis knows: a move and a use in one
# iteration leaves the local definitely moved (O1), while a move that reaches
# its own use round a back edge leaves it *maybe* moved at the loop head,
# because the entry state joins "not yet moved" with "moved last time".

fn consume(owned xs: Array[i32]) -> i32:
    return xs[0]

fn main():
    a: Array[i32] = Array[i32]()
    a.push(1)
    i: i32 = 0
    while i < 2:
        println(consume(a))        #$ error[E3041]: `a` is moved in a loop
        i = i + 1
