# Clause THR-2.3: "A hand-over to another thread is a reshape of what is handed over (§VII.1): no view
# into it, a `sharedref mut` link into a list of struct values included, is used after it (`E3060`)"
#$ test: compile-fail
#$ rules: THR-2, BRW-17
# No text in the spec for this program: code only.

import std.thread

class Names:
    items: Array[String] = []

fn main():
    names = Names()
    names.items.push(owns String("a"))
    first = names.items[0]
    h = thread.spawn(owns fn() owns names: names.items.len())
    println(first)    #$ error[E3060]
    println(h.join())
