#$ test: compile-fail
#$ rules: CLO-3, CLO-6
#$ error[E3030]: closure would move a captured value out
## `[CLO-3]` (ODR-189) — only a consumed callable parameter is an owned callable value in its
## function: one given to a borrowed parameter is lent, and may be called and lent on but not
## stored. A stored `fn(A) -> R` may be called many times, so an argument that moves a capture out
## (callable once, `[CLO-6]`) cannot become one (D-817).

class Bus:
    handlers: Array[fn(int)] = []

    fn subscribe(self, owned handler: fn(int)):
        self.handlers.push(handler)

    fn lend(self, handler: fn(int)):
        self.handlers.push(handler)    #$ error[E2020]: expected `fn(i64) -> void`

fn consume(owned s: String):
    println(s)

fn main():
    bus = Bus()
    s = "moved".to_string()
    bus.subscribe(owned fn(x) => consume(s))
