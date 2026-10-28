#$ test: compile-fail
#$ rules: BRW-18, CTL-2
#$ help: write `a = s.area()` above line 24 and `println(line, a)` in place of line 25
## design rules: R5, R8, R14
## R5, R8 (c), verdict 86: what a call through an interface method changes is worked out from every
## implementation the program can put behind it (it replaces E2113, which compared an implementation
## with the interface method's first line): `Square.area` pushes to `LOG`, so `s.area()` inside a loop
## over `LOG` is refused (E3020), naming the implementation. The nearest program that is allowed is
## `accept_an_interface_method_whose_implementation_pushes_to_log_called_before_the_loop`.

static mut LOG: Array[String] = []

interface Shape:
    fn area() -> float

class Square:
    side: float = 2.0

extend Square implements Shape:
    fn area() -> float:
        LOG.push(String("area"))
        return this.side * this.side

fn report(s: dyn Shape):
    for line in LOG:
        println(line, s.area())    #$ error[E3020]: `s.area()` (line 25) may run `Square.area`, which pushes to `LOG` (line 20), while this loop walks `LOG` (line 24)

fn main():
    LOG.push(String("start"))
    sq = Square()
    report(sq)
