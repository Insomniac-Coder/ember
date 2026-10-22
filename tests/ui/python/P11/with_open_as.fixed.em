import std.fs

fn first(path: str) -> Result[String]:
    with f = fs.File.open(path)?:
        return Ok("opened")
    return Ok("")

fn main():
    println(first("notes.txt"))
