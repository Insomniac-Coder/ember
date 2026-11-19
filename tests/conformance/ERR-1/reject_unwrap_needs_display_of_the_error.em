#$ test: compile-fail
#$ rules: ERR-1, ERR-4
#$ help: implement `Display` for `Silent`
# `[ERR-1]` — printing one needs `Display`: `unwrap` on a `Result[T, E]` panics with the error's
# `Display` text, so an `E` with only `Debug` is `E2040`.

@derive(Debug)
struct Silent:
    code: int

fn open() -> Result[int, Silent]:
    return Err(Silent(2))

fn main():
    println(open().unwrap())    #$ error[E2040]: `Silent` does not implement `Display`, which `unwrap` on a `Result` needs of its error
