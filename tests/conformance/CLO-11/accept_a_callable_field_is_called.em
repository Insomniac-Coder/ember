#$ test: run-pass
#$ rules: CLO-11, CLO-3
#$ stdout:
#$ 12 1000 -5 7
#$ -9 101 9
#$ 8 21 6
# `[CLO-11]` — `recv.f(args)` calls a field `f` of callable type when the
# receiver's type has no method `f`; a method of that name takes precedence,
# and `(recv.f)(args)` reaches the field. `[CLO-3]` — any expression of a
# callable type can be called: a returned function, an array element, a
# lambda written in place (D-222).

struct H:
    f: fn(int) -> int
    g: fn(int) -> int

    fn g(self, x: int) -> int:
        return 1000

fn double(x: int) -> int:
    return x * 2

fn triple(x: int) -> int:
    return x * 3

fn neg(x: int) -> int:
    return -x

fn pick(b: bool) -> fn(int) -> int:
    return double if b else triple

fn through(h: ref H, x: int) -> int:
    return h.f(x) + 1

class Button:
    on_click: fn(int) -> int

    fn init(mut self):
        self.on_click = neg

h = H(f=double, g=neg)
println(h.f(6), h.g(5), (h.g)(5), through(h, 3))
b = Button()
k = H(f=fn(x) => x + 100, g=neg)
println(b.on_click(9), k.f(1), (fn(x: int) => x - 1)(10))
fs = [double, triple]
println(pick(true)(4), pick(false)(7), fs[1](2))
