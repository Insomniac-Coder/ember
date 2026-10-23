fn bump(mut x: int, mut y: int):
    x += 1
    y += 1

fn pick(i: int, j: int):
    a = [1, 2, 3]
    bump(a[i], a[j])
    println(a[0])

fn main():
    pick(0, 2)
