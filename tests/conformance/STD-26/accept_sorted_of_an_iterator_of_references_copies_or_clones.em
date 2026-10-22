#$ test: run-pass
#$ rules: STD-26, TYP-15, STD-12
#$ profiles: debug, development, release
#$ stdout: a 2
#$ b 1
#$ ['b', 'a'] [1, 2]
#$ [1, 2, 3] ['b', 'a']
## `[STD-26]` (ODR-207) — `sorted` of an iterator that lends references gives an `Array[T]` of
## clones (or copies, for a `Copy` `T`), as `sorted(m)` does: an `Array` holds no view it does not
## own (`[TYP-15]`). So `sorted(m.keys())` is an `Array[String]`, and its items index the map
## (it was `Array[ref String]`, refused with `E3063` inside std). With `key=` and `reverse=` too.

fn main():
    m: Map[String, int] = {}
    m["b".to_string()] = 1
    m["a".to_string()] = 2
    for k in sorted(m.keys()):
        println(k, m[k])
    names: Array[String] = sorted(m.keys(), reverse=true)
    println(names, sorted(m.values()))
    xs = [3, 1, 2]
    println(sorted(xs.iter()), sorted(m.keys(), key=fn(k: ref String) => m[k]))
