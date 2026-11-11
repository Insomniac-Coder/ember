#$ test: compile-fail
#$ rules: ERR-4
# `[ERR-4]` — `unwrap` and `expect` on a `Result[T, E]` need `E: needs Display`: `E2040` otherwise,
# whose help is to implement `Display`.

struct Silent:
    code: int

fn open() -> Result[int, Silent]:
    return Err(Silent(code=2))

fn main():
    a = open().unwrap()    #$ error[E2040]: `Silent` does not implement `Display`
    b = open().expect("opening")    #$ error[E2040]: `Silent` does not implement `Display`
    println(a, b)
