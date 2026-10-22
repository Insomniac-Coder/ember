fn main():
    xs = [1, 2, 3]
    doubled = xs.iter().map(lambda x: x * 2).collect[Array[int]]()
    println(doubled)
