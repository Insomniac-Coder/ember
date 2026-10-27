#$ test: compile-fail
#$ rules: R7
#$ help: let buf: Array[u8]
## R7: `from` names a field whose contents live in their own heap block; a fixed array is stored inside
## the object and moves with it, so it is refused (E2118). The message of `V4/rules.md` section 3
## (`let buf: String`, `let` kept so that the line it gives compiles), in this program's names.

class Packet:
    let buf: [u8; 4]
    head: Span[u8] from buf    #$ error[E2118]: `buf` is stored inside the value, so moving the value moves it; keep the bytes in an `Array`, which has a block of its own: write the field as `let buf: Array[u8]`

    fn init():
        this.buf = [1, 2, 3, 4]
        this.head = this.buf[..2]

fn main():
    p = Packet()
    println(p.head.len())
