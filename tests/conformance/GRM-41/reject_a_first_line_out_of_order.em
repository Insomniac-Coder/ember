#$ test: compile-fail
#$ rules: GRM-41
## `[GRM-41]` — a function's first line gives its parts in this order: the parameters, `owns this`, `->`
## and the result type, `from` and what the result points into. Another order is `E0118`, whose message
## gives it in order.

struct Counter:
    n: int = 0

    fn finish() -> u64 owns this:    #$ error[E0118]: a function's first line gives its parts in this order
                                     #$ help: `fn finish() owns this -> u64:`
        return 1

fn next_word(src: String) from src -> StrView:    #$ error[E0118]: a function's first line gives its parts in this order
    return src.as_str()

fn main():
    c = Counter()
    println(c.n)
