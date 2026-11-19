#$ test: run-pass
#$ rules: LEX-6
#$ stdout: 6 2
## `[LEX-6]` — inside `(`, `[` and `{` a newline does not end the line and
## indentation means nothing.

fn add(a: int,
          b: int,
  c: int) -> int:
    return a + b + c

fn main():
    values = [1,
  2,
            3]
    pairs = {"a": 1,
        "b": 2}
    println(add(values[0], values[1],
        values[2]), pairs.len())
