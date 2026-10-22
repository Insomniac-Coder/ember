#$ test: run-pass
#$ rules: CLO-1, OWN-3, FN-1
#$ profiles: debug, development, release
#$ stdout: [1, 2] 2 bb
#$ stdout: 6
# D-876 — a lambda with no result type written whose body is a value read through a reference,
# which only a copy could give by value, gives the reference: `fn(w: ref String) => w` is a
# `fn(ref String) -> ref String`, as with `-> ref String` written, so `map` lends each word and
# nothing is moved out of a borrowed parameter. It was `E3013`. A `Copy` value is given by value
# (`fn(x: ref int) => x` gives the `int`).

fn main():
    words = ["a".to_string(), "bb".to_string()]
    kept = [w.len() for w in words.iter().map(fn(w: ref String) => w)]
    same = fn(w: ref String) => w
    longest = same(words[1])
    println(kept, longest.len(), longest)
    numbers = [1, 2, 3]
    println(numbers.iter().map(fn(x: ref int) => x).sum())
