import lib.a

pub fn down2(n: int) -> int:
    if n <= 1:
        return n
    return a.down(n - 1)
