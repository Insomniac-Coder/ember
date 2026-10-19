#$ test: run-pass
#$ rules: PHIL-7, ERR-13
#$ stdout: not there
#$ stdout: on we go
# `[PHIL-7]` — a recoverable failure is a `Result`: a file that is not there is something the
# program handles and goes on from, never a panic. Panics are for the programmer's errors.

import std.fs

fn main():
    match fs.read_to_string("this file is not there.txt"):
        Ok(text):
            println(text)
        Err(_):
            println("not there")
    println("on we go")
