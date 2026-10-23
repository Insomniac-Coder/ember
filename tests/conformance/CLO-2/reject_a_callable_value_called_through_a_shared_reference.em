#$ test: compile-fail
#$ rules: CLO-2, BRW-1
#$ profiles: debug
# `[CLO-2]` — an owned callable value may change the state it captured, so it is called from a
# mutable place: one reached through a shared reference (a `for` loop's element of a collection of
# them) is `E3021`, whose help borrows it mutably, `.iter_mut()` for the loop (the first week's 04
# calls its handlers so). Not the help for rebinding a reference: nothing is assigned.

struct Bus:
    handlers: Array[fn(int)]

fn publish(bus: Bus, n: int):
    for handler in bus.handlers:
        handler(n)    #$ error[E3021]: cannot write through a shared reference
    #$ help: iterate a collection of them with `.iter_mut()`

fn main():
    bus = Bus([fn(n: int) => println(n)])
    publish(bus, 1)
