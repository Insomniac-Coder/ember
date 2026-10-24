# A14 heap-contents rule: a generator walking its own local Map across yield (Python: for k in d: yield k).
gen fn keys_of() -> Generator[int]:
    m: Map[int, int] = {}
    m[1] = 10
    m[2] = 20
    for k in m.keys():
        yield k

fn main():
    for k in keys_of():
        println(k)
