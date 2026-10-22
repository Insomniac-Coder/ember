#$ test: run-pass
#$ rules: CTL-5
#$ stdout: 1 0
## `[CTL-5]` — the first arm that matches runs, and an arm the arms above
## already cover is `W2091`: here a string literal already named above (D-815).

fn code(word: str) -> int:
    match word:
        "yes" | "y":
            return 1
        "y":    #$ warning[W2091]: this arm can never match
            return 2
        _:
            return 0

fn main():
    println(code("y"), code("no"))
