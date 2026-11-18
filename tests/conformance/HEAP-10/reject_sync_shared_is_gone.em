#$ test: compile-fail
#$ rules: TYP-45, THR-1
# `[TYP-45]` — `SyncShared` is not a type of Ember: naming it is `E1010`, with what to write.

from std.sync import SyncShared    #$ error[E1010]: `SyncShared` is not a type of Ember

fn total(s: SyncShared[int]) -> int:    #$ error[E1010]: write `thread_shared int`
    return s

fn main():
    println(total(SyncShared(1)))    #$ error[E1010]: `SyncShared` is not Ember
