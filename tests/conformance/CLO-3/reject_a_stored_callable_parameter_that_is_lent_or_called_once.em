#$ test: compile-fail
#$ rules: CLO-3, CLO-6
#$ error[E3013]: this function gives `s` away, so it can run only once, and `subscribe` may run it more than once
## `[CLO-3]` (ODR-189) — only a consumed callable parameter is an owned callable value in its
## function: one given to a borrowed parameter is lent, and may be called and lent on but not
## stored. A stored `fn(A) -> R` may be called many times, so an argument that moves a capture out
## (callable once, `[CLO-6]`) cannot become one (D-817).

class Bus:
    handlers: Array[fn(int)] = []

    fn subscribe(owns handler: fn(int)):
        this.handlers.push(owns handler)

    fn lend(handler: fn(int)):
        this.handlers.push(owns handler)    #$ error[E2020]: expected `fn(i64) -> void`

fn consume(owns s: String):
    println(s)

fn main():
    bus = Bus()
    s = "moved".to_string()
    bus.subscribe(owned fn(x) => consume(owns s))
