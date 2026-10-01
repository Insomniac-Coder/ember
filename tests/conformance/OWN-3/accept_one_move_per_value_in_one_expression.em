#$ test: run-pass
#$ rules: OWN-3, EXP-6
#$ stdout: aa 3 b c 2 7 7 d e
# `[OWN-3]` (D-426) — one expression may move a value once, copy a `Copy`
# value any number of times, move different fields of one struct, and use a
# clone beside the original.

struct Pair:
    a: String
    b: String

fn both(owned x: String, owned y: String) -> int:
    return x.len() + y.len()

fn main():
    s = "a".to_string()
    t = (s.clone(), s)
    u = "bb".to_string()
    n = both(u.clone(), "c".to_string()) + 0 * u.len()
    p = Pair("b".to_string(), "c".to_string())
    halves = (p.a, p.b)
    k = 7
    sevens = [k, k]
    q = Pair("d".to_string(), "e".to_string())
    parts = Pair(q.a, q.b)
    println(f"{t.0}{t.1}", n, halves.0, halves.1, sevens.len(), sevens[0], sevens[1], parts.a, parts.b)
