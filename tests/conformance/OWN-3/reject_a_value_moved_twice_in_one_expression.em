#$ test: compile-fail
#$ rules: OWN-3, EXP-6
# `[OWN-3]` (D-426) — the parts of one expression are evaluated left to
# right, so the second part that takes a value by move finds it gone. Each
# of these compiled and freed the string twice at run time.

struct Pair:
    a: String
    b: String

fn both(owned x: String, owned y: String) -> int:
    return x.len() + y.len()

fn main():
    s = "a".to_string()
    t = (s, s)              #$ error[E3040]: `s` is moved twice in one expression
    u = "b".to_string()
    n = both(u, u)          #$ error[E3040]: `u` is moved twice in one expression
    v = "c".to_string()
    p = Pair(v, v)          #$ error[E3040]: `v` is moved twice in one expression
    w = "d".to_string()
    xs = [w, w]             #$ error[E3040]: `w` is moved twice in one expression
    q = Pair("e".to_string(), "f".to_string())
    r = (q, q.a)            #$ error[E3040]: `q` is moved twice in one expression
    println(t.0, n, p.a, xs.len(), r.1)
