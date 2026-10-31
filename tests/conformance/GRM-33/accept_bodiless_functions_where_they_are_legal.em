#$ test: run-pass
#$ rules: GRM-33, FFI-1
#$ stdout: 6.0
#$ stdout: 3
#$ stdout: 4
## `[GRM-33]` — the three places a function may have no body: an interface,
## an `extern` block and a `virtual` method of an `abstract class`.

interface Measured:
    fn size() -> int

abstract class Solid:
    fn init():
        pass

    virtual fn volume() -> f64

class Cube(Solid):
    side: f64

    fn init(side: f64):
        this.side = side
        super.init()

    override fn volume() -> f64:
        return this.side * this.side * this.side

struct Row implements Measured:
    n: int

    fn size() -> int:
        return this.n

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @safety("callers pass what the C function `abs` requires")
    unsafe fn abs(x: i32) -> i32

fn main():
    solid: Solid = Cube(1.0)
    println(solid.volume() * 6.0)
    println(Row(3).size())
    unsafe:  # SAFETY(ffi): each foreign call matches its C declaration
        println(abs(-4))
