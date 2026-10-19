#$ test: run-pass
#$ rules: LAY-2
#$ profiles: debug, development, release
#$ assert-c: contains("#pragma pack(push, 1)")
#$ stdout: 7 1 0 1 5 | 12 4 4 8
#$ 70006 300 1 25
#$ 13 5 7
#$ 30 2 32
#$ hi 3 ok 2
#$ 11 1 25
# `[LAY-2]` — `@packed`: no padding, so the fields sit back to back and the type is aligned to one
# byte; `size_of`, `align_of` and `offset_of` report it. Unaligned fields are read and written by
# value: plain and compound assignment, through a nested struct, in a list's elements, a packed
# enum's payload, and fields that own memory, which move out and drop like any other. A field
# aligned to one byte is never unaligned, so a reference to it is allowed.

@packed
@derive(Copy)
struct Header:
    tag: u8
    length: u32
    flags: u16

struct Plain:
    tag: u8
    length: u32
    flags: u16

@derive(Copy)
struct Inner:
    a: u32
    b: u64

@packed
@derive(Copy)
struct Outer:
    t: u8
    inner: Inner
    pair: (u16, u32)

@packed
enum Message:
    Ping(u8)
    Data(u64, u16)

@packed
struct Record:
    tag: u8
    name: String
    items: Array[int]

fn first(t: ref u8) -> u8:
    return t

fn main():
    println(size_of[Header](), align_of[Header](), offset_of[Header](tag), offset_of[Header](length), offset_of[Header](flags), "|", size_of[Plain](), align_of[Plain](), offset_of[Plain](length), offset_of[Plain](flags))
    h = Header(tag = 1, length = 70000, flags = 300)
    h.length += 6
    h.flags = h.flags * 1
    copy = h
    println(copy.length, copy.flags, first(ref h.tag), size_of[Outer]())
    o = Outer(t = 1, inner = Inner(a = 2, b = 3), pair = (4, 5))
    o.inner.a += 11
    o.pair.1 += 2
    println(o.inner.a, o.inner.b + 2, o.pair.1)
    headers: Array[Header] = [h, h]
    headers[1].length = 30
    headers[1].flags -= 298
    m = Message.Data(9, 2)
    size = match m:
        Message.Ping(x) => x as u64
        Message.Data(a, b) => a + b as u64 + 21
    println(headers[1].length, headers[1].flags, size)
    r = Record(tag = 2, name = "hi", items = [1, 2, 3])
    name = r.name
    items = r.items
    r.name = "ok"
    latest = r.name
    println(name, len(items), latest, r.tag)
    println(size_of[Message](), offset_of[Record](name), offset_of[Record](items))
