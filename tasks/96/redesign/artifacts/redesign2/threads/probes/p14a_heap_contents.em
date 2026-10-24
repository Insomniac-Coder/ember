# A14: a generator walking the characters of its own local String (heap contents) across yield.
gen fn letters() -> Generator[char]:
    s = "abc".to_string()
    for c in s.chars():
        yield c

fn main():
    for c in letters():
        println(c)
