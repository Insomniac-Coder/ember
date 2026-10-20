#$ test: run-pass
#$ rules: ERR-8, ERR-3, STD-22
#$ stdout: failed: true
# `[ERR-8]` — `?` converts any `E: Error` into `AnyError`; `std.io.Error` is one (D-749: it
# implemented only `Display`, and Part XIII's `fs.read_to_string(path)?` into `Result[void]` was
# `E2040`).

import std.fs
import std.io

fn load(path: str) -> Result[String]:
    text = fs.read_to_string(path)?
    return Ok(text)

fn main():
    match load("no-such-file-for-this-test.cfg"):
        Ok(_):
            println("read it")
        Err(e):
            println("failed:", e.is[io.Error]())
