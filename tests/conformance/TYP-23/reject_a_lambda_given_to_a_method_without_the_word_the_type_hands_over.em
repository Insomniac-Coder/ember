#$ test: compile-fail
#$ rules: TYP-23, DIA-2
## `[TYP-23]` — the message names the callee and the callable type where the call has them, and gives
## the line to write; a lambda given to a typed local has no callee to name.

struct Names:
    list: Array[String]

    fn each_name(f: fn(owns String)):
        pass

fn main():
    names = Names([])
    names.each_name(fn(s) => println(s.len()))    #$ error[E2020]: `each_name` hands each value over (`fn(owns String)`), so this function receives it as its own: write `fn(owns s) => println(s.len())`
    handler: fn(owns String) = fn(s) => println(s.len())    #$ error[E2020]: the callable type `fn(owns String)` hands each value over, so this function receives it as its own: write `fn(owns s) => println(s.len())`
