#$ test: run-pass
#$ rules: CLO-5, RC-1
#$ stdout: 5
## `[CLO-5]` — a closure capturing a class handle holds a strong reference:
## the object outlives the function that made it.

class Counter:
    value: int

    fn init(mut self, v: int):
        self.value = v

fn make() -> fn() -> int:
    c = Counter(5)
    return fn() -> int => c.value

fn main():
    f = make()
    println(f())
