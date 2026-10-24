# A11: rebinding one name; Python prints [1].
fn main():
    a: Array[int] = [1]
    b = a
    a = [9]
    println(b)
