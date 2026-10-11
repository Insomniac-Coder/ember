from std.mem import assert_disjoint

fn main():
    a = 1
    b = 2
    r = assert_disjoint(ref a, ref b)
