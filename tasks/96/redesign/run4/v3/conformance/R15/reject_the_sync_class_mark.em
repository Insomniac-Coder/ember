#$ test: compile-fail
#$ rules: R15
## R15: what may go to another thread is inferred from the words; the class mark `@sync` is gone
## (AN-80; section 3's removed words, E0104).

from std.sync import Atomic

@sync    #$ error[E0104]: `@sync` is gone: a class needs no mark to be shared between threads, and `Stats`'s fields are `Atomic`, which threads may change as they are; remove `@sync` (line 8)
class Stats:
    words: Atomic[int] = Atomic(0)

fn main():
    st = Stats()
    println(st.words.load())
