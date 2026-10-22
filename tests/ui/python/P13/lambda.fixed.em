fn main():
    xs = [1, 2, 3]
    doubled = xs.iter().map(fn(x) => x * 2).collect[Array[int]]()
    println(doubled)
