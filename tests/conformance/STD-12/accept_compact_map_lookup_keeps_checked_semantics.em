#$ test: run-pass
#$ rules: STD-11, STD-12, STD-16, STD-17
#$ profiles: debug, release, shipping
#$ stdout: 60 Some(20) None true
#$ stdout: 62 3
#$ stdout: None false
#$ stdout: None
#$ stdout: Some(7)
#$ stdout: 12 11 0 true
#$ stdout: Some(12)
#$ stdout: None false
#$ stdout: 0 3
#$ assert-c: contains("int64_t em_std_collections_Map_String_i64_std_collections_DefaultHasher_find")
# Lookup stays checked and uses the same probe path after removal and reinsertion.
# Both owning and borrowed string keys exercise the general Map implementation.

fn main():
    m: Map[String, int] = {"first": 10, "second": 20, "third": 30}
    owned_key = String.from("first")
    borrowed_key: str = "third"
    total = m[owned_key] + m["second"] + m[borrowed_key]
    removed = m.remove("second")
    missing = m.get("second")
    present = borrowed_key in m
    println(total, removed, missing, present)
    m["second"] = 22
    println(m[owned_key] + m["second"] + m[borrowed_key], len(m))

    # Hash zero starts at table position zero. It must never mean "absent".
    zero: Map[int, int] = {}
    println(zero.get(0), zero.contains_key(0))
    println(zero.remove(0))
    zero[0] = 7
    println(zero.insert(0, 9))
    zero[0] += 1
    match zero.get_mut(0):
        Some(v):
            v += 2
        None:
            pass
    zero.entry(0).or_insert(99)
    zero.entry(1).or_insert_with(fn() => 11)
    zero.entry(2).or_default()
    println(zero[0], zero[1], zero[2], zero.contains_key(0))
    println(zero.remove(0))
    println(zero.get(0), zero.contains_key(0))
    zero.entry(0).or_default()
    zero.entry(0).or_insert_with(fn() => 88)
    println(zero[0], len(zero))
