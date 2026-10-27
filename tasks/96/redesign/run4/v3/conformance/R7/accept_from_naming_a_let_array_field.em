#$ test: run-pass
#$ rules: R7
#$ stdout: 2
## R7: `from` may name a `let` field that is an `Array`, whose elements live in a block of their own,
## so moving the object never moves them (area B, ruling 2). The nearest program to
## `reject_from_naming_a_fixed_array_field.em` that is allowed.

class Packet:
    let buf: Array[u8]
    head: Span[u8] from buf

    fn init():
        this.buf = [1, 2, 3, 4]
        this.head = this.buf[..2]

fn main():
    p = Packet()
    println(p.head.len())
