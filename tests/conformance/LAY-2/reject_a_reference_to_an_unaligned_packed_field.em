#$ test: compile-fail
#$ rules: LAY-2
#$ profiles: debug
# `[LAY-2]` — a field of a `@packed` type may sit at any address, so taking a reference to one is
# `E2170`: written with `ref` or `ref mut`, or made by a parameter mode — a `mut` argument, a
# borrowed argument passed by address, a method's receiver, a loop over it. A field aligned to
# one byte (`tag`) is never unaligned, so its reference is allowed.

struct Big:
    words: [u64; 4]

    fn first(self) -> u64:
        return self.words[0]

@packed
struct Record:
    tag: u8
    length: u32
    big: Big
    items: Array[int]

fn bump(mut n: u32):
    n += 1

fn total(b: Big) -> u64:
    return b.words[1]

fn main():
    r = Record(tag = 1, length = 2, big = Big(words = [1, 2, 3, 4]), items = [5])
    t = ref r.tag
    println(t)
    l = ref r.length  #$ error[E2170]: reference to a field of a packed struct
    println(l)
    m = ref mut r.length  #$ error[E2170]: reference to a field of a packed struct
    println(m)
    bump(r.length)  #$ error[E2170]: reference to a field of a packed struct
    a = total(r.big)  #$ error[E2170]: reference to a field of a packed struct
    b = r.big.first()  #$ error[E2170]: reference to a field of a packed struct
    r.items.push(6)  #$ error[E2170]: reference to a field of a packed struct
    for x in r.items:  #$ error[E2170]: reference to a field of a packed struct
        println(x)
    println(a, b)
