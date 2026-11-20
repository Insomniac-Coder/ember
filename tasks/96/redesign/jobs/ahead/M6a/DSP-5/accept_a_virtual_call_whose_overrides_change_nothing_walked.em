# Clause DSP-5.2: "What a call through a `virtual` or interface method may change, read and end is
# worked out from every override and implementation the program has"
#$ test: run-pass
#$ rules: DSP-5, EFF-1, BRW-18
#$ stdout: start
#$ stdout: 2
# Every override changes only its own object, so the call may be made while `LOG` is walked.

static mut LOG: Array[String] = []

open class Shape:
    size: int = 0

    fn init():
        pass

    virtual fn grow():
        this.size += 1

class Square(Shape):
    fn init():
        super.init()

    override fn grow():
        this.size += 2

fn tick_all(ref mut s: Shape):
    for line in LOG:
        println(line)
        s.grow()

fn main():
    LOG.push(owns String("start"))
    sq: Shape = Square()
    tick_all(ref mut sq)
    println(sq.size)
