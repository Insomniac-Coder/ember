fn first(path: str) -> Result[String]:
    with open(path) as f:
        return Ok("opened")
    return Ok("")

fn main():
    println(first("notes.txt"))
