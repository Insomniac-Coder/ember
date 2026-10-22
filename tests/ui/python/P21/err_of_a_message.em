fn check(n: int) -> Result[int]:
    if n < 0:
        return Err("negative")
    return Ok(n)

fn main():
    println(check(1))
