#$ test: run-pass
#$ rules: TXT-11, RNG-8, TYP-31
#$ profiles: debug, release, shipping
#$ stdout: é ok

type Size = i128 in 0 ..= 64

fn reserve_from_ref(mut text: String, count: ref int):
    text.reserve(count)

fn main():
    capacity: Size = 16
    text = String.with_capacity(capacity)
    offset: Size = 0
    text.insert(offset, 'é')
    println(text.remove(offset), "ok")
    count = 8
    reserve_from_ref(text, ref count)
    text.push_str("abc")
    prefix: Size = 1
    text.truncate(prefix)
    assert(text.len() == 1)
