# Spec IX says Box has `b.get()`; does the checker know it?
fn main():
    b = Box(3)
    println(b.get())
