#$ test: run-pass
#$ rules: RC-3, RC-1, FN-1, DRP-2
#$ profiles: debug, development, release
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 0
#$ stdout: inside
#$ stdout: drop 9
#$ stdout: after callee drop
#$ stdout: caller

# `[OWN-9]` — a new object handed to an `owns` parameter has that one owner, so
# the callee ends it where it says.
import std.mem

class Token:
    id: i32

    fn drop():
        println("drop", this.id)

fn consume(owns value: Token):
    println("inside")
    mem.drop(value)
    println("after callee drop")

fn main():
    consume(owns Token(9))
    println("caller")
