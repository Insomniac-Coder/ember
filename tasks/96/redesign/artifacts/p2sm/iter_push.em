fn main():
    a = [1, 2, 3]
    for x in a:
        if x == 1:
            a.push(4)
    println(a.len())
