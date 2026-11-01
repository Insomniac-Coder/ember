fn double(ref mut n: i32):
    n = n + n

fn main():
    value: i32 = 1 + 1
    double(ref mut value)
