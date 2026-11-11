import lib.b

pub fn down(n: int) -> int:
    if n <= 1:
        return n
    return b.down2(n - 1) + 1
