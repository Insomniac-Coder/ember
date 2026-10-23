fn main():
    a = [1, 2, 3]
    r = ref mut a[0]
    a.push(4)
    r += 1
