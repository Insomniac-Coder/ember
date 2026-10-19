#$ test: compile-fail
#$ rules: STD-22, EFF-5
#$ error[E4041]: `@noio` function `size` reaches I/O
# `[STD-22]` — every operation that touches the outside world carries `Io`.

import std.fs

@noio
fn size(path: str) -> int:
    match fs.metadata(path):
        Ok(meta):
            return meta.len
        Err(_):
            return 0

fn main():
    println(size("."))
