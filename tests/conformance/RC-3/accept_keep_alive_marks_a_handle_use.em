#$ test: run-pass
#$ rules: RC-3, RC-2a
#$ profiles: debug, release, shipping
#$ stdout: live
#$ stdout: dropped
# `keep_alive` is a borrowed use of the owner at the named point.
import std.mem

class Token:
    fn drop(mut self):
        println("dropped")

fn main():
    h = Token()
    mem.keep_alive(h)
    println("live")
