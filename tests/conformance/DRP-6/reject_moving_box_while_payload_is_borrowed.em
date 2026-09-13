#$ test: compile-fail
#$ rules: DRP-6, BRW-1, OWN-3
# `get` is a real borrow of the Box owner, not a detached raw pointer.

fn consume(owned value: Box[i32]):
    println(value.get())

fn main():
    boxed: Box[i32] = Box(9)
    payload = boxed.get()
    consume(boxed) #$ error[E3021]: `boxed` cannot be moved while it is borrowed
    println(payload)
