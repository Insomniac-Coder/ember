#$ test: run-pass
#$ rules: TXT-11, TXT-4, TYP-31, HEAP-8
#$ profiles: debug, release, shipping
#$ stdout: o

fn main():
    n: u128 = 16
    s = String.with_capacity(n)
    extra: i128 = 8
    s.reserve(extra)
    index: i8 = 0
    s.insert(index, 'é')
    at: u128 = 0
    assert(s.remove(at) == 'é')
    s.push_str("ok")
    count: i128 = 1
    s.truncate(count)
    println(s)
