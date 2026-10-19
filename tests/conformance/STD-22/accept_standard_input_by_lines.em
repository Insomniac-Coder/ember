#$ test: run-pass
#$ rules: STD-22
#$ stdin: first
#$ stdin: second
#$ stdin: third
#$ stdout: 1 first
#$ 2 second
#$ 3 third
#$ end
# `[STD-22]` — `io.stdin().read_line()` is one line without its ending, `None` at the end of
# input; `lines()` gives the lines that are left.

import std.io

fn main():
    match io.stdin().read_line():
        Ok(Some(line)):
            println(1, line)
        _:
            println("no line")
    n = 2
    for line in io.stdin().lines():
        match line:
            Ok(text):
                println(n, text)
            Err(error):
                println(error)
        n += 1
    match io.stdin().read_line():
        Ok(None):
            println("end")
        _:
            println("more?")
