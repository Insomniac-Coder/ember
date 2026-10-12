#$ test: run-pass
#$ rules: GRM-33, FFI-1
#$ stdout: 6.0
#$ stdout: 3
#$ stdout: 4
## `[GRM-33]` — the three places a function may have no body: an interface,
## an `extern` block and a `virtual` method of an `abstract class`.

interface Measured:
    fn size(self) -> int

abstract class Solid:
    fn init(mut self):
        pass

    virtual fn volume(self) -> f64

class Cube(Solid):
    side: f64

    fn init(mut self, side: f64):
        self.side = side
        super.init()

    override fn volume(self) -> f64:
        return self.side * self.side * self.side

struct Row implements Measured:
    n: int

    fn size(self) -> int:
        return self.n

unsafe extern "C":
    fn abs(x: i32) -> i32

fn main():
    solid: Solid = Cube(1.0)
    println(solid.volume() * 6.0)
    println(Row(3).size())
    unsafe:
        println(abs(-4))
