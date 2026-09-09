#$ test: run-pass
#$ rules: BRW-5
# "`ref mut a[i]` and `ref mut a[j]` conflict **unless both indices are
# constants and different**."
#
# The exemption was unreachable: every index went into a runtime slot and came
# back as `Index(local)`, so `overlaps` — which is ready to tell two
# `ConstIndex` projections apart — never saw one. `v[0]` and `v[1]` were
# rejected as though the indices might be equal.
#
# For an `Array[T]` a second thing was in the way: the bounds check reads the
# length through the container's internal `Field(1)`, and that read was treated
# as overlapping the element borrow. Ember has no `v.len` *field* — only a
# `len()` method — so that pair is only ever the compiler's own header access.

fn main():
    v: Array[i32] = Array[i32]()
    v.push(1)
    v.push(2)
    a: ref mut i32 = ref mut v[0]
    b: ref mut i32 = ref mut v[1]
    a = 10
    b = 20
    println(v[0] + v[1])
#$ stdout: 30
