#$ test: run-pass
#$ rules: RC-3, RC-1, FN-1, DRP-2
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 1
#$ stdout: inside
#$ stdout: after callee drop
#$ stdout: drop 9
#$ stdout: caller

# Last use does not permit an early move into an owned parameter: the callee
# may release its owner before returning. The expression temporary survives.
import std.mem

class Token:
    id: i32

    fn drop(mut self):
        println("drop", self.id)

fn consume(owned value: Token):
    println("inside")
    mem.drop(value)
    println("after callee drop")

fn main():
    consume(Token(9))
    println("caller")
