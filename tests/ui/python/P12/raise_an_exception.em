fn check(n: int) -> Result[int]:
    if n < 0:
        raise ValueError("negative")
    return Ok(n)

fn main():
    println(check(1))
