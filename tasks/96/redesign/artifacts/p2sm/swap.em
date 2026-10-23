fn main():
    a = [1, 2, 3]
    i = 0
    j = 2
    a[i], a[j] = a[j], a[i]
    println(a[0])
