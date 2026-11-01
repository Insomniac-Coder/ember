#$ test: run-pass
#$ rules: CLO-3, CLO-7, CLO-10
#$ profiles: debug, development, release
#$ stdout: pong 3
#$ lambda
#$ owned pong
#$ stored
#$ 11
## `[CLO-3]` (ODR-189) — a consumed callable parameter, `owned f: fn(A) -> R`, is an owned callable
## value in its function: a registry may store it in a collection or return it, whatever the caller
## gave — a function, a lambda that captures nothing, an `owned fn` with its captured state, or an
## owned callable value it already held. It was `E2020`, "found `Callable0`" (D-817).

enum Event:
    Ping
    Pong(int)

class Bus:
    handlers: Array[fn(Event)] = []

    fn subscribe(owns handler: fn(Event)):
        this.handlers.push(owns handler)

    fn publish(e: Event):
        for h in this.handlers.iter_mut():
            h(e)

fn report(e: Event):
    match e:
        Ping:
            println("ping")
        Pong(n):
            println("pong", n)

fn keep(owns f: fn(int) -> int) -> fn(int) -> int:
    return f

fn main():
    bus = Bus()
    bus.subscribe(owns report)
    bus.subscribe(owns fn(e) => println("lambda"))
    word = "owned".to_string()
    bus.subscribe(owns owned fn(e) => println(word, "pong"))
    stored: fn(Event) = fn(e) => println("stored")
    bus.subscribe(owns stored)
    bus.publish(Event.Pong(3))
    k = 10
    add = keep(owns owned fn(x) => x + k)
    println(add(1))
