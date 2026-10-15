#$ test: run-pass
#$ rules: CT-2, CT-1, CT-4
#$ stdout: 2 first line true None
# `[CT-2]` — compile time is given two inputs: `comptime.read_file(path)`, relative to the package
# directory, and `comptime.env(name)`, an `Option[String]`. Both are recorded as build inputs: a
# value is computed again when the file or the variable changes (`[CT-4]`).

fn count_lines(text: String) -> int:
    n = 0
    for _ in text.lines():
        n += 1
    return n

const LINES: int = count_lines(comptime.read_file("support/notes.txt"))

fn first_line(text: String) -> String:
    for line in text.lines():
        return line.to_string()
    return String()

fn main():
    first = comptime(first_line(comptime.read_file("support/notes.txt")))
    has_path = comptime(comptime.env("PATH").is_some())
    unset = comptime(comptime.env("EMBER_SURELY_UNSET_VARIABLE_FOR_CT_2"))
    println(LINES, first, has_path, unset)
