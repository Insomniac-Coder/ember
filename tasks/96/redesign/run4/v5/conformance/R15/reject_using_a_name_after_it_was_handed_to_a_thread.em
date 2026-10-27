#$ test: compile-fail
#$ rules: R15, R1
## R15 and R1: `fn() owns names` hands the object over; a later use is E3040 (AN-71's message, with
## every line it replaces named, R18). The nearest allowed program is
## `accept_a_thread_shared_object_is_read_by_a_thread_and_its_maker.em`.

import std.thread

class Names:
    items: Array[String] = []

fn share_plain():
    names = Names(items=[String("ann")])
    h = thread.spawn(fn() owns names: names.items.len())
    println(names.items.len(), h.join())    #$ error[E3040]: `names` was handed to the thread on line 14 (`fn() owns names`); to keep using it here, create it shared and give the thread one more owner: write `names = thread_shared Names(items=[String("ann")])` instead of line 13 and `h = thread.spawn(fn() thread_shared names: names.items.len())` instead of line 14

fn main():
    share_plain()
