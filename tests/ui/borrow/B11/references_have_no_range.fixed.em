from std.mem import assert_disjoint

fn main():
    a: Array[int] = [1]
    b: Array[int] = [2]
    println(assert_disjoint(a[..], b[..]).is_ok())
