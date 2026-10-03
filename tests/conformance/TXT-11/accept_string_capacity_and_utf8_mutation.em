#$ test: run-pass
#$ assert-c: contains("vec_reserve_more_inline(")
#$ rules: TXT-11, TXT-4, TYP-31, HEAP-8
#$ profiles: debug, release, shipping
#$ stdout: é a
#$ stdout: λ🌶
#$ stdout: true true
#$ assert-c: contains("ember_string_insert(")
#$ assert-c: contains("ember_string_remove(")
#$ assert-c: contains("ember_string_truncate(")

fn main():
    s = String.with_capacity(32)
    cap = s.capacity()
    assert(cap >= 32)
    assert(s.len() == 0)
    s.push_str("aé🌶z")
    s.insert(1, 'λ')
    s.insert(s.len(), '🦀')
    assert(s.capacity() == cap)
    println(s.remove(3), s.remove(0))
    assert(s.capacity() == cap)
    s.truncate(6)
    println(s)
    assert(s.capacity() == cap)
    s.truncate(900)
    assert(s.len() == 6)
    s.reserve(40)
    assert(s.capacity() >= s.len() + 40)
    grown = s.capacity()
    s.reserve(0)
    assert(s.capacity() == grown)
    s.clear()
    assert(s.len() == 0)
    assert(s.capacity() == grown)
    s.insert(0, '\0')
    assert(s.remove(0) == '\0')
    empty = String.with_capacity(0)
    empty.reserve(0)
    empty.truncate(10)
    empty.clear()
    assert(empty.capacity() == 0)
    println(s.is_empty(), empty.is_empty())
