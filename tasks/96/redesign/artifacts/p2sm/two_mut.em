fn bump(mut x: int, mut y: int):
    x += 1
    y += 1

fn main():
    a = [1, 2, 3]
    bump(a[0], a[2])
    println(a[0])
