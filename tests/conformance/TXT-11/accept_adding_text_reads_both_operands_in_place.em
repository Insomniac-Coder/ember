#$ test: run-pass
#$ rules: TXT-11, TXT-8
#$ stdout: ab cd abcd abef
#$ true 0
#$ Z x Z 1
#$ true true
#$ abcdcdcd
## `[TXT-11]` - `a + b` for `a: String, b: StrView` (or a `String`, read as a `StrView`) reads both in
## place and gives a new `String`, `a` then `b`, so `a` stays usable; `s += t` appends in place;
## `with_capacity`, `capacity`, `reserve`, `insert`, `remove`, `truncate` and `clear` as the rule says
## (`remove`, `truncate` and `clear` keep the capacity).

fn main():
    a = String("ab")
    b = String("cd")
    c = a + b
    d = a + "ef"
    println(a, b, c, d)
    s = String.with_capacity(10)
    println(s.capacity() >= 10, String.with_capacity(0).capacity())
    s += "xy"
    s.reserve(5)
    s.insert(1, 'Z')
    r = s.remove(0)
    s.truncate(1)
    println(s, r, s.as_str(), s.len())
    s.clear()
    println(s.is_empty(), s.capacity() >= 10)
    t = String("ab")
    t = t + b + b + b[0..2]
    println(t)
