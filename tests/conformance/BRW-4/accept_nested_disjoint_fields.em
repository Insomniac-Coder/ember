#$ test: run-pass
#$ rules: BRW-4
# "This holds through arbitrary nesting of field projections."

struct Inner:
    pub a: i32
    pub b: i32

struct Outer:
    pub inner: Inner
    pub tag: i32

fn main():
    o = Outer(Inner(1, 2), 3)
    x: ref mut i32 = ref mut o.inner.a
    y: ref mut i32 = ref mut o.inner.b
    x = 10
    y = 20
    println(o.inner.a + o.inner.b)
#$ stdout: 30
