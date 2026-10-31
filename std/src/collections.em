## `std.collections` — fixed-capacity Arena-backed collections introduced by
## `[ARN-5]`–`[ARN-5g]`.
##
## These declarations own the public names and their concrete representation.
## Operations are compiler-known while the generic standard-library substrate
## is still being completed, exactly as `Array`, `Cell`, and `Arena` are. The
## first field is a real borrow/provenance anchor; the containers do not own
## the Arena or their backing bytes.

import std.mem
from std.core import DoubleEndedIterator, ExactSizeIterator, FromIterator

## `[HASH-1]` — the protocol is static at the `Hash.hash` boundary. A concrete
## hasher supplies `H`; no bare-interface conversion or mandatory dynamic
## dispatch is involved.
pub interface Hasher:
    fn write_bytes(bytes: Span[u8])
    fn write_u8(x: u8)
    fn write_u16(x: u16)
    fn write_u32(x: u32)
    fn write_u64(x: u64)
    fn write_i8(x: i8)
    fn write_i16(x: i16)
    fn write_i32(x: i32)
    fn write_i64(x: i64)
    fn write_usize(x: usize)
    fn write_isize(x: isize)
    fn finish() owns this -> u64

pub interface Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H)

## Canonical scalar implementations feed the matching-width operation into
## the supplied protocol. They intentionally do not depend on the concrete
## `DefaultHasher` mixer.
extend u8 implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_u8(this)

extend u16 implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_u16(this)

extend u32 implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_u32(this)

extend u64 implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_u64(this)

extend usize implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_usize(this)

extend i8 implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_i8(this)

extend i16 implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_i16(this)

extend i32 implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_i32(this)

extend i64 implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_i64(this)

extend isize implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_isize(this)

extend bool implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        if this:
            h.write_u8(1)
        else:
            h.write_u8(0)

extend char implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_u32(this as u32)

extend i128 implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_u64(this as u64)
        h.write_u64((this >> 64) as u64)

extend u128 implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_u64(this as u64)
        h.write_u64((this >> 64) as u64)

## G8-4 — its low 128 bits, then whether it is past a `u128` (a `u256` has no shifts to reach its
## high bits with).
extend u256 implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        low = this as u128
        h.write_u64(low as u64)
        h.write_u64((low >> 64) as u64)
        if this > (u128.MAX as u256):
            h.write_u8(2)
        else:
            h.write_u8(0)

## G8-4 — its low 128 bits, then where it lies against them: below zero, within a `u128`, or
## above one (an `i256` has no shifts to reach its high bits with).
extend i256 implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        low = this as u128
        h.write_u64(low as u64)
        h.write_u64((low >> 64) as u64)
        if this < 0:
            h.write_u8(1)
        elif this > (u128.MAX as i256):
            h.write_u8(2)
        else:
            h.write_u8(0)

## `[TYP-36]`, `[STD-12]` — text hashes its bytes, then a byte no UTF-8 text
## contains, so `("ab", "c")` and `("a", "bc")` feed different streams. A
## `String` hashes exactly as the `str` it holds.
extend StrView implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_bytes(this.as_bytes())
        h.write_u8(255)

extend String implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_bytes(this.as_bytes())
        h.write_u8(255)

## A sequence hashes its length, then its elements: `[[1], [2, 3]]` and
## `[[1, 2], [3]]` differ. A fixed array hashes as its `Span`.
extend[T: needs Hash] Span[T] implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_usize(this.len() as usize)
        for x in this:
            x.hash(h)

extend[T: needs Hash] Array[T] implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_usize(this.len() as usize)
        for x in this:
            x.hash(h)

extend[T: needs Hash] Option[T] implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        match this:
            Some(x):
                h.write_u8(1)
                x.hash(h)
            None:
                h.write_u8(0)

extend[T: needs Hash, E: needs Hash] Result[T, E] implements Hash:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        match this:
            Ok(x):
                h.write_u8(0)
                x.hash(h)
            Err(e):
                h.write_u8(1)
                e.hash(h)

## `[STD-12]` (ODR-032, ODR-067) — what a lookup may take for a key `K`:
## something that hashes as `K` would and compares with one without
## converting. Every `K: Eq + Hash` is `AsKey[K]` (the compiler provides it:
## `is_key` is `==`), so looking a key up never needs to copy one.
pub interface AsKey[K]: needs Hash:
    fn is_key(key: K) -> bool

## `[STD-12]` (ODR-067) — a lookup key that can also make the key it matches,
## for `m[q] = v` when `q` is new. Every `K: Eq + Hash + Clone` is `ToKey[K]`
## (the compiler provides it: `to_key` clones).
pub interface ToKey[K]: needs AsKey[K]:
    fn to_key() -> K

extend StrView implements AsKey[String], ToKey[String]:
    fn is_key(key: String) -> bool:
        return this == key

    ## It allocates a new `String`.
    fn to_key() -> String:
        return String.from(this)

## `[HASH-2]` — fixed-seed and FxHash's class: one multiply, one rotate and
## one xor per word. The state is spread by the multiply before each word is
## xored in, so a key of one word (any integer) hashes to itself, as C++'s
## `std::hash` does, and a `Map` keeps keys that are close together in
## neighbouring places (ADR-100); a key of several words still mixes them all.
## The exact mixer is replaceable and therefore is not part of Ember's source
## compatibility contract. Bytes go in eight to a word. No `Copy` derive is
## present: this state is moved.
pub struct DefaultHasher implements Hasher:
    state: u64

    pub static fn new() -> DefaultHasher:
        return DefaultHasher(0)

    @overflow(wrap)
    fn mix(word: u64):
        spread = this.state * 0x517cc1b727220a95
        this.state = ((spread << 5) | (spread >> 59)) ^ word

    fn write_bytes(bytes: Span[u8]):
        index = 0
        while index + 8 <= bytes.len():
            this.mix(full_word(bytes, index))
            index = index + 8
        if index < bytes.len():
            this.mix(tail_word(bytes, index))

    fn write_u8(x: u8):
        this.mix(x as u64)

    fn write_u16(x: u16):
        this.mix(x as u64)

    fn write_u32(x: u32):
        this.mix(x as u64)

    fn write_u64(x: u64):
        this.mix(x)

    fn write_i8(x: i8):
        this.mix(x as u64)

    fn write_i16(x: i16):
        this.mix(x as u64)

    fn write_i32(x: i32):
        this.mix(x as u64)

    fn write_i64(x: i64):
        this.mix(x as u64)

    fn write_usize(x: usize):
        this.mix(x as u64)

    fn write_isize(x: isize):
        this.mix(x as u64)

    fn finish() owns this -> u64:
        return this.state

extend DefaultHasher implements Default:
    static fn default() -> DefaultHasher:
        return DefaultHasher.new()

## The last one to seven bytes of `bytes`, from `index`, as one word, with
## their count in its top byte (D-405): without it `"a"` and `"a\0"` gave one
## word, so they hashed alike under every key.
fn tail_word(bytes: Span[u8], index: int) -> u64:
    word: u64 = 0
    k = 0
    while index + k < bytes.len():
        word = word | ((bytes[index + k] as u64) << ((8 * k) as u64))
        k = k + 1
    return word | ((k as u64) << 56)

## Every word of `bytes`, eight bytes to a word, then the tail's.
fn full_word(bytes: Span[u8], index: int) -> u64:
    word: u64 = 0
    for k in range(8):
        word = word | ((bytes[index + k] as u64) << ((8 * k) as u64))
    return word

## `[HASH-2]` — a per-process randomly seeded hasher, for maps keyed by
## untrusted input: SipHash-1-3, keyed by 128 bits the process draws once from
## the operating system, so which keys collide cannot be known outside it.
## Its values differ from run to run and are never seen: a
## `Map[K, V, RandomState]` iterates in insertion order (`[STD-11]`). Each
## write is one 64-bit word of the message, bytes eight to a word as
## `DefaultHasher` takes them; the result is SipHash-1-3 of the words'
## little-endian bytes. No `Copy` derive is present: this state is moved.
pub struct RandomState implements Hasher:
    v0: u64
    v1: u64
    v2: u64
    v3: u64
    words: u64

    pub static fn new() -> RandomState:
        k0 = process_key(0)
        k1 = process_key(1)
        return RandomState(k0 ^ 0x736f6d6570736575, k1 ^ 0x646f72616e646f6d, k0 ^ 0x6c7967656e657261, k1 ^ 0x7465646279746573, 0)

    @overflow(wrap)
    fn round():
        this.v0 = this.v0 + this.v1
        this.v1 = (this.v1 << 13) | (this.v1 >> 51)
        this.v1 = this.v1 ^ this.v0
        this.v0 = (this.v0 << 32) | (this.v0 >> 32)
        this.v2 = this.v2 + this.v3
        this.v3 = (this.v3 << 16) | (this.v3 >> 48)
        this.v3 = this.v3 ^ this.v2
        this.v0 = this.v0 + this.v3
        this.v3 = (this.v3 << 21) | (this.v3 >> 43)
        this.v3 = this.v3 ^ this.v0
        this.v2 = this.v2 + this.v1
        this.v1 = (this.v1 << 17) | (this.v1 >> 47)
        this.v1 = this.v1 ^ this.v2
        this.v2 = (this.v2 << 32) | (this.v2 >> 32)

    @overflow(wrap)
    fn mix(word: u64):
        this.v3 = this.v3 ^ word
        this.round()
        this.v0 = this.v0 ^ word
        this.words = this.words + 1

    fn write_bytes(bytes: Span[u8]):
        index = 0
        while index + 8 <= bytes.len():
            this.mix(full_word(bytes, index))
            index = index + 8
        if index < bytes.len():
            this.mix(tail_word(bytes, index))

    fn write_u8(x: u8):
        this.mix(x as u64)

    fn write_u16(x: u16):
        this.mix(x as u64)

    fn write_u32(x: u32):
        this.mix(x as u64)

    fn write_u64(x: u64):
        this.mix(x)

    fn write_i8(x: i8):
        this.mix(x as u64)

    fn write_i16(x: i16):
        this.mix(x as u64)

    fn write_i32(x: i32):
        this.mix(x as u64)

    fn write_i64(x: i64):
        this.mix(x as u64)

    fn write_usize(x: usize):
        this.mix(x as u64)

    fn write_isize(x: isize):
        this.mix(x as u64)

    @overflow(wrap)
    fn finish() owns this -> u64:
        s = this
        last: u64 = ((s.words * 8) & 0xff) << 56
        s.v3 = s.v3 ^ last
        s.round()
        s.v0 = s.v0 ^ last
        s.v2 = s.v2 ^ 0xff
        s.round()
        s.round()
        s.round()
        return s.v0 ^ s.v1 ^ s.v2 ^ s.v3

extend RandomState implements Default:
    static fn default() -> RandomState:
        return RandomState.new()

## `[HASH-2]` — word `which` (0 or 1) of the process's hash key, which the
## runtime draws from the operating system at the first call. The compiler
## makes a call the runtime's `process_key`; this staging body never runs.
fn process_key(which: int) -> u64:
    return 0

## The largest prime at most `size`, the length of a `Map`'s table before it
## switches (ADR-100): keys whose gap is a power of two do not share places.
fn prime_at_most(size: int) -> int:
    n = size
    while n > 3:
        prime = true
        d = 2
        while d * d <= n:
            if n % d == 0:
                prime = false
                break
            d += 1
        if prime:
            return n
        n -= 1
    return n

## One entry: its hash, kept so growing never hashes again, its key and its
## value.
[K, V]
struct MapSlot:
    hash: u64
    key: K
    value: V

## `[STD-11]`, `[STD-16]` (ODR-032) — a hash map that iterates in insertion
## order. `entries` holds the entries in that order; a removed one is `None`
## until the map closes the gaps up. `slots` is an open-addressed table of
## positions into `entries`: -1 is empty and -2 a removed entry's old place.
## It is at most seven eighths used, so a probe always meets an empty place
## and ends (`[HASH-3]`), and it is probed linearly from a key's home (ADR-100):
## * the hash's remainder by the table's length, a prime: keys close together
##   get neighbouring places, so a run of them is read in order, and keys a
##   power of two apart do not share one;
## * once storing an entry has walked past more than 100 used places (keys a
##   multiple of the length apart all share one home), the map switches for
##   good to the hash times FxHash's constant, its high bits, in a table a
##   power of two long (`use_multiply`), which spreads any such run.
## A place holds a 4-byte entry number while the table is at most 2^31 − 1
## long, an 8-byte one in a longer table (`wide`).
[K: needs (Eq, Hash), V, H: needs (Hasher, Default) = DefaultHasher]
pub struct Map:
    entries: Array[Option[MapSlot[K, V]]] = []
    slots: Array[i32] = []
    wide_slots: Array[int] = []
    wide: bool = false
    live: int = 0
    used: int = 0
    shift: u64 = 64
    use_multiply: bool = false

    ## `Map[K, V]()` is an empty map (`[STR-6]`: every field has a default).
    pub fn init():
        pass

    pub static fn with_capacity(n: int) -> Self:
        m = Self()
        m.reserve(n)
        return m

    pub fn len() -> int:
        return this.live

    pub fn is_empty() -> bool:
        return this.live == 0

    ## How many entries fit before the table grows.
    pub fn capacity() -> int:
        return this.slot_count() * 7 // 8

    pub fn clear():
        this.entries.clear()
        this.slots.clear()
        this.wide_slots.clear()
        this.wide = false
        this.live = 0
        this.used = 0
        this.shift = 64
        this.use_multiply = false

    ## Room for `n` more entries. A rebuild leaves at least half the usable
    ## room free, so at a steady load near the limit removed places are
    ## dropped only after as many operations again: insertion stays expected
    ## constant time, amortised (`[STD-11]`, ODR-033).
    pub fn reserve(n: int):
        if (this.used + n) * 8 <= this.slot_count() * 7:
            return
        size = 8
        while size * 7 < (this.live + n) * 16:
            size = size * 2
        this.rebuild(size)

    [Q: needs (AsKey[K], Hash)]
    fn hash_of(q: Q) -> u64:
        h = H.default()
        q.hash(h)
        return h.finish()

    ## The entry number at position `at`, from whichever list the table is in.
    fn slot_at(at: int) -> int:
        if this.wide:
            return this.wide_slots[at]
        return this.slots[at] as int

    fn set_slot(at: int, value: int):
        if this.wide:
            this.wide_slots[at] = value
        else:
            this.slots[at] = value as i32

    ## The table's length.
    fn slot_count() -> int:
        if this.wide:
            return this.wide_slots.len()
        return this.slots.len()

    ## Which position in `slots` a key starts at.
    @overflow(wrap)
    fn home(hash: u64, places: int) -> int:
        if this.use_multiply:
            return ((hash * 0x517cc1b727220a95) >> this.shift) as int
        return (hash % (places as u64)) as int

    ## The table position of `q`, or -1 when absent. Positions are nonnegative,
    ## so this private result fits one integer (ADR-144). Public lookups still
    ## return Option values or checked references as their contracts require.
    [Q: needs (AsKey[K], Hash)]
    fn find(q: Q, hash: u64) -> int:
        places = this.slot_count()
        if places == 0:
            return -1
        at = this.home(hash, places)
        if this.wide:
            while true:
                slot = this.wide_slots[at]
                if slot == -1:
                    return -1
                if slot >= 0:
                    match this.entries[slot]:
                        Some(e):
                            if e.hash == hash and q.is_key(e.key):
                                return at
                        None:
                            pass
                at += 1
                if at == places:
                    at = 0
        else:
            while true:
                slot = (this.slots[at] as int)
                if slot == -1:
                    return -1
                if slot >= 0:
                    match this.entries[slot]:
                        Some(e):
                            if e.hash == hash and q.is_key(e.key):
                                return at
                        None:
                            pass
                at += 1
                if at == places:
                    at = 0
        return -1

    ## Lists entry `i` in `slots`, in the first free place from its home, and
    ## says how many used places it walked past.
    fn place(hash: u64, i: int) -> int:
        places = this.slot_count()
        at = this.home(hash, places)
        passed = 0
        if this.wide:
            while this.wide_slots[at] >= 0:
                at += 1
                if at == places:
                    at = 0
                passed += 1
        else:
            while (this.slots[at] as int) >= 0:
                at += 1
                if at == places:
                    at = 0
                passed += 1
        if this.slot_at(at) == -1:
            this.used += 1
        this.set_slot(at, i)
        return passed

    ## Closes up removed entries, then lists every entry in a table of `size`.
    fn rebuild(size: int):
        if this.entries.len() != this.live:
            this.entries.retain(fn(e) => e.is_some())
        this.slots.clear()
        bits = 0
        while (1 << bits) < size:
            bits += 1
        length = prime_at_most(size)
        if this.use_multiply:
            length = 1 << bits
        this.wide_slots.clear()
        this.wide = length > 2147483647
        if this.wide:
            for _ in range(length):
                this.wide_slots.push(-1)
        else:
            for _ in range(length):
                this.slots.push(-1)
        this.shift = (64 - bits) as u64
        this.used = 0
        for i in range(this.entries.len()):
            hash: u64 = 0
            match this.entries[i]:
                Some(e):
                    hash = e.hash
                None:
                    pass
            if this.place(hash, i) > 100 and not this.use_multiply:
                this.use_multiply = true
                this.rebuild(size)
                return

    fn push_new(hash: u64, owns k: K, owns v: V):
        this.reserve(1)
        at = this.entries.len()
        this.entries.push(Some(MapSlot(hash, k, v)))
        passed = this.place(hash, at)
        this.live += 1
        if passed > 100 and not this.use_multiply:
            this.use_multiply = true
            this.rebuild(this.slot_count())

    pub fn insert(owns k: K, owns v: V) -> Option[V]:
        hash = this.hash_of(k)
        at = this.find(k, hash)
        if at >= 0:
            match this.entries[this.slot_at(at)]:
                Some(ref mut e):
                    return Some(mem.replace(e.value, v))
                None:
                    unreachable()  # `find` gives a listed entry's slot only
        this.push_new(hash, k, v)
        return None

    [Q: needs (AsKey[K], Hash)]
    pub fn get(q: Q) -> Option[ref V]:
        at = this.find(q, this.hash_of(q))
        if at >= 0:
            match this.entries[this.slot_at(at)]:
                Some(e):
                    return Some(ref e.value)
                None:
                    return None
        else:
            return None

    [Q: needs (AsKey[K], Hash)]
    pub fn get_mut(q: Q) -> Option[ref mut V]:
        at = this.find(q, this.hash_of(q))
        if at >= 0:
            match this.entries[this.slot_at(at)]:
                Some(ref mut e):
                    return Some(ref mut e.value)
                None:
                    return None
        else:
            return None

    [Q: needs (AsKey[K], Hash)]
    pub fn contains_key(q: Q) -> bool:
        return this.find(q, this.hash_of(q)) >= 0

    ## `k in m` (`[STD-8]`): by key.
    [Q: needs (AsKey[K], Hash)]
    pub fn contains(q: Q) -> bool:
        return this.find(q, this.hash_of(q)) >= 0

    [Q: needs (AsKey[K], Hash)]
    pub fn remove(q: Q) -> Option[V]:
        at = this.find(q, this.hash_of(q))
        if at >= 0:
            i = this.slot_at(at)
            this.set_slot(at, -2)
            old = mem.replace(this.entries[i], None)
            this.live -= 1
            if this.entries.len() > 16 and this.live * 2 < this.entries.len():
                this.rebuild(this.slot_count())
            match owns old:
                Some(e):
                    return Some(e.value)
                None:
                    return None
        else:
            return None

    ## Keeps the entries `keep` accepts, in order.
    [F: needs fn(ref K, ref V) -> bool]
    pub fn retain(owns keep: F):
        for i in range(this.entries.len()):
            gone = false
            match this.entries[i]:
                Some(e):
                    gone = not keep(ref e.key, ref e.value)
                None:
                    pass
            if gone:
                this.entries[i] = None
                this.live -= 1
        this.rebuild(this.slot_count())

    ## Removes and returns the newest entry, as Python's `dict.popitem` does.
    pub fn pop_item() -> Option[(K, V)]:
        while this.entries.len() > 0:
            last = this.entries.len() - 1
            hash: u64 = 0
            match this.entries[last]:
                Some(e):
                    hash = e.hash
                None:
                    pass
            slot = this.entries.pop()
            match owns slot:
                Some(Some(e)):
                    this.forget(hash, last)
                    this.live -= 1
                    return Some((e.key, e.value))
                _:
                    pass
        return None

    ## Marks the place listing entry `i` as removed.
    fn forget(hash: u64, i: int):
        places = this.slot_count()
        at = this.home(hash, places)
        while this.slot_at(at) != i:
            at += 1
            if at == places:
                at = 0
        this.set_slot(at, -2)

    ## Inserts `other`'s entries in its order.
    pub fn update(owns other: Map[K, V, H]):
        for i in range(other.entries.len()):
            slot = mem.replace(other.entries[i], None)
            match owns slot:
                Some(e):
                    this.insert(e.key, e.value)
                None:
                    pass

    ## The entries, in order, consuming the map.
    pub fn into_items() owns this -> Array[(K, V)]:
        out = []
        for i in range(this.entries.len()):
            slot = mem.replace(this.entries[i], None)
            match owns slot:
                Some(e):
                    out.push((e.key, e.value))
                None:
                    pass
        return out

## `for k in owned m:` (`[CTL-1]`): the keys in insertion order, owned. Each
## value is dropped as its key is taken, and the entries left when a loop ends
## early are dropped with the iterator.
[K, V]
pub struct MapIntoKeys:
    entries: Array[Option[MapSlot[K, V]]]
    at: int

    pub fn next() -> Option[K]:
        while this.at < this.entries.len():
            i = this.at
            this.at += 1
            slot = mem.replace(this.entries[i], None)
            match owns slot:
                Some(e):
                    return Some(e.key)
                None:
                    pass
        return None

extend[K, V] MapIntoKeys[K, V] implements Iterator:
    type Item = K

## The key-value pairs (ODR-094); a later pair's value replaces an earlier
## one's for the same key, as `insert` does.
extend[K: needs (Eq, Hash), V, H: needs (Hasher, Default)] Map[K, V, H] implements FromIterator[(K, V)]:
    [I: needs Iterator[Item = (K, V)]]
    static fn from_iter(owns it: I) -> Map[K, V, H]:
        out = Map[K, V, H]()
        for (k, v) in it:
            out.insert(k, v)
        return out

extend[K: needs (Eq, Hash), V, H: needs (Hasher, Default)] Map[K, V, H] implements IntoIterator:
    type Item = K
    type Iter = MapIntoKeys[K, V]

    fn into_iter() owns this -> MapIntoKeys[K, V]:
        return MapIntoKeys(this.entries, 0)

## `m[q]`, `m[q] op= v` and `m[q] = v` (`[STD-12]`, `[STD-17]`): a map is
## indexed by anything that is its key, `Q: AsKey[K]`, each such `Q` giving it
## an implementation (a blanket one, `[TYP-19]`).
extend[K: needs (Eq, Hash), V, H: needs (Hasher, Default), Q: needs (AsKey[K], Hash, Debug)] Map[K, V, H] implements Index[Q], IndexMut[Q]:
    type Output = V

    ## The value, or a panic naming the key.
    @inline
    fn index(q: Q) -> ref V:
        at = this.find(q, this.hash_of(q))
        if at >= 0:
            match this.entries[this.slot_at(at)]:
                Some(e):
                    return ref e.value
                None:
                    unreachable()  # `find` gives a listed entry's slot only
        else:
            panic(f"key not found: {q!r}; use .get(k) for an Option")

    ## `m[q] op= v`: the key must be there.
    fn index_mut(q: Q) -> ref mut V:
        at = this.find(q, this.hash_of(q))
        if at >= 0:
            match this.entries[this.slot_at(at)]:
                Some(ref mut e):
                    return ref mut e.value
                None:
                    unreachable()  # `find` gives a listed entry's slot only
        else:
            panic(f"key not found: {q!r}; use .get(k) for an Option")

extend[K: needs (Eq, Hash), V, H: needs (Hasher, Default), Q: needs (ToKey[K], Hash)] Map[K, V, H] implements IndexSet[Q, V]:
    ## Replaces, or inserts `q.to_key()` when the key is new (`[STD-12]`).
    fn index_set(q: Q, owns v: V):
        hash = this.hash_of(q)
        at = this.find(q, hash)
        if at >= 0:
            match this.entries[this.slot_at(at)]:
                Some(ref mut e):
                    e.value = v
                    return
                None:
                    unreachable()  # `find` gives a listed entry's slot only
        this.push_new(hash, q.to_key(), v)

extend[K: needs (Eq, Hash), V: needs Copy, H: needs (Hasher, Default)] Map[K, V, H]:
    [Q: needs (AsKey[K], Hash)]
    pub fn get_or(q: Q, default: V) -> V:
        match this.get(q):
            Some(v):
                return v
            None:
                return default

## `[STD-16]` — equal as sets of entries: order does not matter.
extend[K: needs (Eq, Hash), V: needs Eq, H: needs (Hasher, Default)] Map[K, V, H] implements Eq:
    fn eq(other: Map[K, V, H]) -> bool:
        if this.live != other.live:
            return false
        for i in range(this.entries.len()):
            match this.entries[i]:
                Some(e):
                    match other.get(e.key):
                        Some(v):
                            if v != e.value:
                                return false
                        None:
                            return false
                None:
                    pass
        return true

## `sorted(m)` (`[STD-26]`, ODR-034) sorts the keys, and `sorted(s)` the
## elements, into a new `Array` of clones.
extend[K: needs (Eq, Hash, Ord, Clone), V, H: needs (Hasher, Default)] Map[K, V, H]:
    pub fn sorted_keys() -> Array[K]:
        out: Array[K] = []
        for i in range(this.entries.len()):
            match this.entries[i]:
                Some(e):
                    out.push(e.key.clone())
                None:
                    pass
        out.sort()
        return out

extend[K: needs (Eq, Hash), V, H: needs (Hasher, Default)] Map[K, V, H] implements Default:
    static fn default() -> Map[K, V, H]:
        return Map[K, V, H]()

## `m.entry(k)`: the map, borrowed until the entry is used, and the key.
@view
[K: needs (Eq, Hash), V, H: needs (Hasher, Default)]
pub struct MapEntry:
    map: ref mut Map[K, V, H]
    key: K
    hash: u64

    ## The value, inserting `v` first when the key is new.
    pub fn or_insert(owns v: V) owns this -> ref mut V:
        i = 0
        at = this.map.find(this.key, this.hash)
        if at >= 0:
            i = this.map.slot_at(at)
        else:
            # `push_new` may close up removed entries first, so the new
            # entry's place is read after it.
            this.map.push_new(this.hash, this.key, v)
            i = this.map.entries.len() - 1
        match this.map.entries[i]:
            Some(ref mut e):
                return ref mut e.value
            None:
                unreachable()  # `find` gives a listed entry's slot only

    [F: needs fn() -> V]
    pub fn or_insert_with(owns make: F) owns this -> ref mut V:
        at = this.map.find(this.key, this.hash)
        if at >= 0:
            i = this.map.slot_at(at)
            match this.map.entries[i]:
                Some(ref mut e):
                    return ref mut e.value
                None:
                    unreachable()  # `find` gives a listed entry's slot only
        else:
            return this.or_insert(make())

extend[K: needs (Eq, Hash), V: needs Default, H: needs (Hasher, Default)] MapEntry[K, V, H]:
    pub fn or_default() owns this -> ref mut V:
        i = 0
        at = this.map.find(this.key, this.hash)
        if at >= 0:
            i = this.map.slot_at(at)
        else:
            this.map.push_new(this.hash, this.key, V.default())
            i = this.map.entries.len() - 1
        match this.map.entries[i]:
            Some(ref mut e):
                return ref mut e.value
            None:
                unreachable()  # `find` gives a listed entry's slot only

## `m.keys()`, and what `for k in m:` iterates: the keys in insertion order.
@view
[K: needs (Eq, Hash), V, H: needs (Hasher, Default)]
pub struct MapKeys:
    owner: ref Map[K, V, H]
    at: int

    pub fn next() -> Option[ref K] from owner:
        while this.at < this.owner.entries.len():
            i = this.at
            this.at += 1
            match this.owner.entries[i]:
                Some(e):
                    return Some(ref e.key)
                None:
                    pass
        return None

@view
[K: needs (Eq, Hash), V, H: needs (Hasher, Default)]
pub struct MapValues:
    owner: ref Map[K, V, H]
    at: int

    pub fn next() -> Option[ref V] from owner:
        while this.at < this.owner.entries.len():
            i = this.at
            this.at += 1
            match this.owner.entries[i]:
                Some(e):
                    return Some(ref e.value)
                None:
                    pass
        return None

@view
[K: needs (Eq, Hash), V, H: needs (Hasher, Default)]
pub struct MapValuesMut:
    owner: ref mut Map[K, V, H]
    at: int

    pub fn next() -> Option[ref mut V]:
        while this.at < this.owner.entries.len():
            i = this.at
            this.at += 1
            match this.owner.entries[i]:
                Some(ref mut e):
                    return Some(ref mut e.value)
                None:
                    pass
        return None

@view
[K: needs (Eq, Hash), V, H: needs (Hasher, Default)]
pub struct MapItems:
    owner: ref Map[K, V, H]
    at: int

    pub fn next() -> Option[(ref K, ref V)] from owner:
        while this.at < this.owner.entries.len():
            i = this.at
            this.at += 1
            match this.owner.entries[i]:
                Some(e):
                    return Some((ref e.key, ref e.value))
                None:
                    pass
        return None

## `[STD-19]` — each of them is an `Iterator`, with its adapters and
## consumers.
extend[K: needs (Eq, Hash), V, H: needs (Hasher, Default)] MapKeys[K, V, H] implements Iterator:
    type Item = ref K

extend[K: needs (Eq, Hash), V, H: needs (Hasher, Default)] MapValues[K, V, H] implements Iterator:
    type Item = ref V

extend[K: needs (Eq, Hash), V, H: needs (Hasher, Default)] MapValuesMut[K, V, H] implements Iterator:
    type Item = ref mut V

extend[K: needs (Eq, Hash), V, H: needs (Hasher, Default)] MapItems[K, V, H] implements Iterator:
    type Item = (ref K, ref V)

## The methods that name the view types declared above.
extend[K: needs (Eq, Hash), V, H: needs (Hasher, Default)] Map[K, V, H]:
    pub fn entry(owns k: K) -> MapEntry[K, V, H]:
        hash = this.hash_of(k)
        return MapEntry(ref mut this, k, hash)

    pub fn iter() -> MapKeys[K, V, H]:
        return MapKeys(ref this, 0)

    pub fn keys() -> MapKeys[K, V, H]:
        return MapKeys(ref this, 0)

    pub fn values() -> MapValues[K, V, H]:
        return MapValues(ref this, 0)

    pub fn values_mut() -> MapValuesMut[K, V, H]:
        return MapValuesMut(ref mut this, 0)

    pub fn items() -> MapItems[K, V, H]:
        return MapItems(ref this, 0)


## `[STD-11]`, `[STD-16]` (ODR-032) — a `Map` with no values: insertion
## order, expected constant-time membership.
[T: needs (Eq, Hash), H: needs (Hasher, Default) = DefaultHasher]
pub struct Set:
    map: Map[T, bool, H] = Map[T, bool, H]()

    ## `Set[T]()` is an empty set.
    pub fn init():
        pass

    pub static fn with_capacity(n: int) -> Self:
        s = Self()
        s.map.reserve(n)
        return s

    pub fn len() -> int:
        return this.map.live

    pub fn is_empty() -> bool:
        return this.map.live == 0

    pub fn capacity() -> int:
        return this.map.capacity()

    pub fn reserve(n: int):
        this.map.reserve(n)

    pub fn clear():
        this.map.clear()

    ## Whether `x` was new. An element already there keeps its position.
    pub fn add(owns x: T) -> bool:
        return this.map.insert(x, true).is_none()

    [Q: needs (AsKey[T], Hash)]
    pub fn remove(q: Q) -> bool:
        return this.map.remove(q).is_some()

    [Q: needs (AsKey[T], Hash)]
    pub fn contains(q: Q) -> bool:
        return this.map.contains_key(q)

    [F: needs fn(ref T) -> bool]
    pub fn retain(owns keep: F):
        for i in range(this.map.entries.len()):
            gone = false
            match this.map.entries[i]:
                Some(e):
                    gone = not keep(ref e.key)
                None:
                    pass
            if gone:
                this.map.entries[i] = None
                this.map.live -= 1
        this.map.rebuild(this.map.slot_count())

    ## Removes and returns the newest element.
    pub fn pop() -> Option[T]:
        match owns this.map.pop_item():
            Some(pair):
                return Some(pair.0)
            None:
                return None

## `s.iter()`, and what `for x in s:` iterates: the elements in order.
@view
[T: needs (Eq, Hash), H: needs (Hasher, Default)]
pub struct SetIter:
    owner: ref Map[T, bool, H]
    at: int

    pub fn next() -> Option[ref T] from owner:
        while this.at < this.owner.entries.len():
            i = this.at
            this.at += 1
            match this.owner.entries[i]:
                Some(e):
                    return Some(ref e.key)
                None:
                    pass
        return None

extend[T: needs (Eq, Hash), H: needs (Hasher, Default)] SetIter[T, H] implements Iterator:
    type Item = ref T

extend[T: needs (Eq, Hash), H: needs (Hasher, Default)] Set[T, H]:
    pub fn iter() -> SetIter[T, H]:
        return SetIter(ref this.map, 0)

    pub fn is_subset(other: Set[T, H]) -> bool:
        for x in this.iter():
            if not other.contains(x):
                return false
        return true

    pub fn is_superset(other: Set[T, H]) -> bool:
        return other.is_subset(this)

    pub fn is_disjoint(other: Set[T, H]) -> bool:
        for x in this.iter():
            if other.contains(x):
                return false
        return true

## The set operations make a new set: this one's elements first, in its
## order, then the other's.
extend[T: needs (Eq, Hash, Clone), H: needs (Hasher, Default)] Set[T, H]:
    pub fn union(other: Set[T, H]) -> Set[T, H]:
        out = Set[T, H]()
        for x in this.iter():
            out.add(x.clone())
        for x in other.iter():
            out.add(x.clone())
        return out

    pub fn intersection(other: Set[T, H]) -> Set[T, H]:
        out = Set[T, H]()
        for x in this.iter():
            if other.contains(x):
                out.add(x.clone())
        return out

    pub fn difference(other: Set[T, H]) -> Set[T, H]:
        out = Set[T, H]()
        for x in this.iter():
            if not other.contains(x):
                out.add(x.clone())
        return out

    pub fn symmetric_difference(other: Set[T, H]) -> Set[T, H]:
        out = this.difference(other)
        for x in other.iter():
            if not this.contains(x):
                out.add(x.clone())
        return out

## `s | t`, `s & t`, `s - t` and `s ^ t` are those operations (`[STD-16]`): a
## set implements the operators' interfaces (`[TYP-21]`).
extend[T: needs (Eq, Hash, Clone), H: needs (Hasher, Default)] Set[T, H] implements BitOr, BitAnd, Sub, BitXor:
    type Output = Set[T, H]

    fn bitor(other: Set[T, H]) -> Set[T, H]:
        return this.union(other)

    fn bitand(other: Set[T, H]) -> Set[T, H]:
        return this.intersection(other)

    fn sub(other: Set[T, H]) -> Set[T, H]:
        return this.difference(other)

    fn bitxor(other: Set[T, H]) -> Set[T, H]:
        return this.symmetric_difference(other)

extend[T: needs (Eq, Hash, Ord, Clone), H: needs (Hasher, Default)] Set[T, H]:
    pub fn sorted_elements() -> Array[T]:
        out: Array[T] = []
        for x in this.iter():
            out.push(x.clone())
        out.sort()
        return out

## `for x in owned s:` (`[CTL-1]`): the elements in insertion order, owned.
## The elements, each once, in the order they first came (ODR-094).
extend[T: needs (Eq, Hash), H: needs (Hasher, Default)] Set[T, H] implements FromIterator[T]:
    [I: needs Iterator[Item = T]]
    static fn from_iter(owns it: I) -> Set[T, H]:
        out = Set[T, H]()
        for x in it:
            out.add(x)
        return out

extend[T: needs (Eq, Hash), H: needs (Hasher, Default)] Set[T, H] implements IntoIterator:
    type Item = T
    type Iter = MapIntoKeys[T, bool]

    fn into_iter() owns this -> MapIntoKeys[T, bool]:
        return this.map.into_iter()

extend[T: needs (Eq, Hash), H: needs (Hasher, Default)] Set[T, H] implements Eq:
    fn eq(other: Set[T, H]) -> bool:
        return this.len() == other.len() and this.is_subset(other)

extend[T: needs (Eq, Hash), H: needs (Hasher, Default)] Set[T, H] implements Default:
    static fn default() -> Set[T, H]:
        return Set[T, H]()

pub enum CapacityError:
    Full

## `[SOA-4]` — the fixed-capacity, arena-backed `SoA[T]`: `ArenaSoA[T].with_capacity(arena, n)`
## takes one block for `n` elements from `arena`, which it borrows for its life, and the block never
## grows or moves (`[ARN-5]`). `push` past the capacity returns `Err(CapacityError.Full)` and
## changes nothing; `capacity()` tells it; the elements must not need drop (`E3090`). Its columns,
## elements, proxies and other operations are an `SoA[T]`'s (`[SOA-1]`..`[SOA-7]`). The compiler
## builds it for each `T`, as it builds `SoA[T]`; this declaration gives it its name here.
@view
[T]
pub struct ArenaSoA:
    arena: ref Arena
    first: *mut T

@view
[T]
pub struct ArenaArray:
    arena: ref Arena
    data: *mut T
    length: usize
    limit: usize

@view
[T]
pub struct ArenaArrayIter:
    owner: ref ArenaArray[T]
    index: usize

@view
[T]
pub struct ArenaArrayIterMut:
    owner: ref mut ArenaArray[T]
    index: usize

## `[SPN-4]`–`[SPN-6]` — the public identities and representations of the
## canonical Span iterators. Each stores one reborrowed source view plus an
## advancing cursor; chunk iterators additionally retain their non-zero
## width. The compiler currently lowers `next` and construction while the
## generic standard-library substrate is completed, as for the Arena-backed
## iterators above.
@view
[T]
pub struct SpanIter:
    source: Span[T]
    index: usize

@view
[T]
pub struct MutSpanIter:
    source: MutSpan[T]
    index: usize

@view
[T]
pub struct SpanChunks:
    source: Span[T]
    index: usize
    width: usize

@view
[T]
pub struct MutSpanChunks:
    source: MutSpan[T]
    index: usize
    width: usize

## `[STD-19]` — each is an `Iterator`, its `next` built in, with its adapters
## and consumers.
extend[T] SpanIter[T] implements Iterator:
    type Item = ref T

extend[T] MutSpanIter[T] implements Iterator:
    type Item = ref mut T

## ODR-091 — an element iterator runs backwards and knows its length, both
## built in: `next_back` takes the last element of the view it holds.
extend[T] SpanIter[T] implements DoubleEndedIterator, ExactSizeIterator:
    pass

extend[T] MutSpanIter[T] implements DoubleEndedIterator, ExactSizeIterator:
    pass

extend[T] SpanChunks[T] implements Iterator:
    type Item = Span[T]

extend[T] MutSpanChunks[T] implements Iterator:
    type Item = MutSpan[T]

## `[STD-15]` (ODR-031) — `windows(n)`: every run of `n` neighbours, one step
## apart. Shared views only: overlapping mutable ones would alias. The fields
## are `SpanChunks`'s, in its order, because construction is lowered the same.
@view
[T]
pub struct SpanWindows:
    source: Span[T]
    index: usize
    width: usize

extend[T] SpanWindows[T] implements Iterator:
    type Item = Span[T]

## Occupancy is separate from the two uninitialized carriers. This lets an
## empty fixed-capacity map reserve all backing bytes in one Arena allocation
## without manufacturing invalid `K` or `V` values.
[K, V]
struct ArenaMapSlot:
    occupied: bool
    key: MaybeUninit[K]
    value: MaybeUninit[V]

@view
[K, V]
pub struct ArenaMap:
    arena: ref Arena
    slots: *mut ArenaMapSlot[K, V]
    length: usize
    limit: usize

@view
[K, V]
pub struct ArenaMapIter:
    owner: ref ArenaMap[K, V]
    index: usize

## `[HND-2]` — how a `Pool`'s handle splits into an index and a generation, and the word that holds
## both. `Split32` (the default) is a `u64` of 32 and 32 bits; `Split20` a `u32` of a 20-bit index
## and a 12-bit generation, for pools that want small handles and few reuses of a slot.
pub interface HandleSplit:
    type Word: needs Copy
    static fn index_bits() -> int
    static fn generation_bits() -> int
    static fn pack(index: int, generation: int) -> Word
    static fn index_of(word: Word) -> int
    static fn generation_of(word: Word) -> int

pub struct Split32:
    pass

extend Split32 implements HandleSplit:
    type Word = u64

    static fn index_bits() -> int:
        return 32

    static fn generation_bits() -> int:
        return 32

    static fn pack(index: int, generation: int) -> u64:
        return ((generation as u64) << 32) | (index as u64)

    static fn index_of(word: u64) -> int:
        return (word & 0xFFFFFFFF) as int

    static fn generation_of(word: u64) -> int:
        return (word >> 32) as int

pub struct Split20:
    pass

extend Split20 implements HandleSplit:
    type Word = u32

    static fn index_bits() -> int:
        return 20

    static fn generation_bits() -> int:
        return 12

    static fn pack(index: int, generation: int) -> u32:
        return ((generation as u32) << 20) | (index as u32)

    static fn index_of(word: u32) -> int:
        return (word & 0xFFFFF) as int

    static fn generation_of(word: u32) -> int:
        return (word >> 20) as int

## `[HND-1]` — a reference into a `Pool`: a plain `Copy` value holding the slot's index and the
## generation the slot had when the handle was given out (`[HND-2]`: in one `B.Word`).
@derive(Copy)
[T, B: needs HandleSplit = Split32]
pub struct Handle:
    word: B.Word

    ## The slot's index.
    pub fn index() -> int:
        return B.index_of(this.word)

    ## The slot's generation when this handle was given out.
    pub fn generation() -> int:
        return B.generation_of(this.word)

## `[HND-1]`, `[HND-3]` — values reached through handles that a removal makes stale. Every lookup
## compares the handle's generation with the slot's, in every profile: a stale handle gives `None`
## (`pool[h]` panics). A removal moves its slot to the next generation; a slot whose generation is
## exhausted is retired and never reused, so an old handle never becomes valid again. When every
## slot is retired or live, `insert` panics and `try_insert` returns `Err(CapacityError.Full)`.
[T, B: needs HandleSplit = Split32]
pub struct Pool:
    slots: Array[Option[T]] = []
    generations: Array[int] = []
    free: Array[int] = []
    live: int = 0

    ## `Pool[T]()` is an empty pool (`[STR-6]`: every field has a default).
    pub fn init():
        pass

    ## How many values the pool holds.
    pub fn len() -> int:
        return this.live

    ## Puts `value` in a free slot and gives its handle; panics when every slot is retired or live.
    pub fn insert(owns value: T) -> Handle[T, B]:
        match this.try_insert(value):
            Ok(handle):
                return handle
            Err(_):
                panic("`Pool.insert`: every slot is retired or live; use .try_insert(value) for a Result")

    ## Puts `value` in a free slot and gives its handle, or `Err(CapacityError.Full)` when every slot
    ## is retired or live.
    pub fn try_insert(owns value: T) -> Result[Handle[T, B], CapacityError]:
        index = 0
        if this.free.len() > 0:
            match this.free.pop():
                Some(at):
                    index = at
                None:
                    pass
            this.slots[index] = Some(value)
        elif this.slots.len() < (1 << B.index_bits()):
            index = this.slots.len()
            this.slots.push(Some(value))
            this.generations.push(0)
        else:
            return Err(CapacityError.Full)
        this.live += 1
        return Ok(Handle[T, B](word = B.pack(index, this.generations[index])))

    ## Whether `h` is live: its slot holds a value of `h`'s generation.
    pub fn contains(h: Handle[T, B]) -> bool:
        at = h.index()
        return at < this.slots.len() and this.generations[at] == h.generation() and this.slots[at].is_some()

    ## The value `h` refers to, or `None` when `h` is stale.
    pub fn get(h: Handle[T, B]) -> Option[ref T]:
        if not this.contains(h):
            return None
        match this.slots[h.index()]:
            Some(ref value):
                return Some(value)
            None:
                return None

    ## The value `h` refers to, mutably, or `None` when `h` is stale.
    pub fn get_mut(h: Handle[T, B]) -> Option[ref mut T]:
        if not this.contains(h):
            return None
        match this.slots[h.index()]:
            Some(ref mut value):
                return Some(value)
            None:
                return None

    ## The value `h` refers to, without comparing generations: `h` must be live.
    @safety("`h` is live: given by this pool and not removed since")
    pub unsafe fn get_unchecked(h: Handle[T, B]) -> ref T:
        match this.slots[h.index()]:
            Some(ref value):
                return value
            None:
                panic("`Pool.get_unchecked`: the slot is empty; use .get(h) for an Option")

    ## Takes the value `h` refers to out of the pool, or `None` when `h` is stale. Its slot moves to
    ## the next generation, or is retired when its generations are exhausted.
    pub fn remove(h: Handle[T, B]) -> Option[T]:
        if not this.contains(h):
            return None
        at = h.index()
        taken = this.slots[at].take()
        this.live -= 1
        next = this.generations[at] + 1
        this.generations[at] = next
        if next < (1 << B.generation_bits()):
            this.free.push(at)
        return taken

extend[T, B: needs HandleSplit] Pool[T, B] implements Index[Handle[T, B]]:
    type Output = T

    ## `pool[h]` — the value `h` refers to; a stale handle panics.
    fn index(h: Handle[T, B]) -> ref T:
        match this.get(h):
            Some(value):
                return value
            None:
                panic("stale handle; use .get(h) for an Option")

## `[STD-7]` — up to `N` values of `T` held inline, never on the heap. `push` past `N` returns
## `Err(CapacityError.Full)` and changes nothing; `push_or_drop` drops the value instead.
[T, const N: int]
pub struct FixedArray:
    items: [Option[T]; N]
    count: int

    pub static fn new() -> FixedArray[T, N]:
        return FixedArray[T, N]([None; N], 0)

    pub fn len() -> int:
        return this.count

    pub fn is_empty() -> bool:
        return this.count == 0

    pub fn is_full() -> bool:
        return this.count == N

    pub fn capacity() -> int:
        return N

    pub fn push(owns value: T) -> Result[void, CapacityError]:
        if this.count == N:
            return Err(CapacityError.Full)
        this.items[this.count] = Some(value)
        this.count += 1
        return Ok(())

    pub fn push_or_drop(owns value: T):
        _ = this.push(value)

    pub fn pop() -> Option[T]:
        if this.count == 0:
            return None
        this.count -= 1
        return this.items[this.count].take()

    pub fn get(i: int) -> Option[ref T]:
        if i < 0 or i >= this.count:
            return None
        match this.items[i]:
            Some(ref v):
                return Some(v)
            None:
                return None

    pub fn clear():
        while this.count > 0:
            this.count -= 1
            _ = this.items[this.count].take()

    pub fn iter() -> FixedArrayIter[T, N]:
        return FixedArrayIter(ref this, 0)

@view
[T, const N: int]
pub struct FixedArrayIter:
    owner: ref FixedArray[T, N]
    at: int

    pub fn next() -> Option[ref T] from owner:
        i = this.at
        this.at += 1
        return this.owner.get(i)

extend[T, const N: int] FixedArrayIter[T, N] implements Iterator:
    type Item = ref T

## `[STD-7]` — text of up to `N` bytes of UTF-8 held inline. `push` and `push_str` past `N` bytes
## return `Err(CapacityError.Full)` and change nothing.
[const N: int]
pub struct FixedString:
    bytes: [u8; N]
    count: int

    pub static fn new() -> FixedString[N]:
        return FixedString[N]([0; N], 0)

    pub fn len() -> int:
        return this.count

    pub fn is_empty() -> bool:
        return this.count == 0

    pub fn capacity() -> int:
        return N

    pub fn push_str(s: StrView) -> Result[void, CapacityError]:
        b = s.as_bytes()
        if this.count + b.len() > N:
            return Err(CapacityError.Full)
        for x in b:
            this.bytes[this.count] = x
            this.count += 1
        return Ok(())

    pub fn push(c: char) -> Result[void, CapacityError]:
        return this.push_str(c.to_string())

    pub fn push_or_drop(s: StrView):
        _ = this.push_str(s)

    pub fn as_str() -> StrView:
        match this.bytes[0..this.count].to_str():
            Ok(text):
                return text
            Err(_):
                return ""

    pub fn clear():
        this.count = 0

## `[STD-7]` — a first-in, first-out queue of up to `N` values held inline. `push` adds at the
## back and past `N` returns `Err(CapacityError.Full)`; `push_or_drop` drops the new value then;
## `pop` takes from the front.
[T, const N: int]
pub struct RingBuffer:
    items: [Option[T]; N]
    head: int
    count: int

    pub static fn new() -> RingBuffer[T, N]:
        return RingBuffer[T, N]([None; N], 0, 0)

    pub fn len() -> int:
        return this.count

    pub fn is_empty() -> bool:
        return this.count == 0

    pub fn is_full() -> bool:
        return this.count == N

    pub fn capacity() -> int:
        return N

    pub fn push(owns value: T) -> Result[void, CapacityError]:
        if this.count == N:
            return Err(CapacityError.Full)
        this.items[(this.head + this.count) % N] = Some(value)
        this.count += 1
        return Ok(())

    pub fn push_or_drop(owns value: T):
        _ = this.push(value)

    pub fn pop() -> Option[T]:
        if this.count == 0:
            return None
        value = this.items[this.head].take()
        this.head = (this.head + 1) % N
        this.count -= 1
        return value

    ## The `i`th value from the front.
    pub fn get(i: int) -> Option[ref T]:
        if i < 0 or i >= this.count:
            return None
        match this.items[(this.head + i) % N]:
            Some(ref v):
                return Some(v)
            None:
                return None

    pub fn clear():
        while this.count > 0:
            _ = this.pop()

## `[STD-7]` — a double-ended queue on the heap: pushes and pops at either end in constant time,
## growing as it fills.
[T]
pub struct Deque:
    slots: Array[Option[T]]
    head: int
    count: int

    pub static fn new() -> Deque[T]:
        return Deque[T]([], 0, 0)

    pub fn len() -> int:
        return this.count

    pub fn is_empty() -> bool:
        return this.count == 0

    fn grow():
        if this.count < this.slots.len():
            return
        bigger: Array[Option[T]] = []
        size = 4 if this.slots.len() == 0 else this.slots.len() * 2
        for i in 0..size:
            if i < this.count:
                bigger.push(this.slots[(this.head + i) % this.slots.len()].take())
            else:
                bigger.push(None)
        this.slots = bigger
        this.head = 0

    pub fn push_back(owns value: T):
        this.grow()
        this.slots[(this.head + this.count) % this.slots.len()] = Some(value)
        this.count += 1

    pub fn push_front(owns value: T):
        this.grow()
        this.head = (this.head + this.slots.len() - 1) % this.slots.len()
        this.slots[this.head] = Some(value)
        this.count += 1

    pub fn pop_front() -> Option[T]:
        if this.count == 0:
            return None
        value = this.slots[this.head].take()
        this.head = (this.head + 1) % this.slots.len()
        this.count -= 1
        return value

    pub fn pop_back() -> Option[T]:
        if this.count == 0:
            return None
        this.count -= 1
        return this.slots[(this.head + this.count) % this.slots.len()].take()

    ## The `i`th value from the front.
    pub fn get(i: int) -> Option[ref T]:
        if i < 0 or i >= this.count:
            return None
        match this.slots[(this.head + i) % this.slots.len()]:
            Some(ref v):
                return Some(v)
            None:
                return None

    pub fn clear():
        while this.count > 0:
            _ = this.pop_back()

## `[STD-7]` — an array that holds up to `N` values inline and moves them to the heap when the
## `N + 1`th arrives.
[T, const N: int]
pub struct SmallArray:
    held: [Option[T]; N]
    count: int
    heap: Array[T]
    spilled: bool

    pub static fn new() -> SmallArray[T, N]:
        return SmallArray[T, N]([None; N], 0, [], false)

    pub fn len() -> int:
        return this.heap.len() if this.spilled else this.count

    pub fn is_empty() -> bool:
        return this.len() == 0

    ## Whether the values are still held inline.
    pub fn is_inline() -> bool:
        return not this.spilled

    pub fn push(owns value: T):
        if this.spilled:
            this.heap.push(value)
            return
        if this.count < N:
            this.held[this.count] = Some(value)
            this.count += 1
            return
        for i in 0..this.count:
            if this.held[i].take() exists as v:
                this.heap.push(v)
        this.count = 0
        this.spilled = true
        this.heap.push(value)

    pub fn pop() -> Option[T]:
        if this.spilled:
            return this.heap.pop()
        if this.count == 0:
            return None
        this.count -= 1
        return this.held[this.count].take()

    pub fn get(i: int) -> Option[ref T]:
        if this.spilled:
            if i < 0 or i >= this.heap.len():
                return None
            return Some(ref this.heap[i])
        if i < 0 or i >= this.count:
            return None
        match this.held[i]:
            Some(ref v):
                return Some(v)
            None:
                return None

## `[STD-7]` — a set of non-negative integers, one bit each, growing as larger ones arrive.
pub struct BitSet:
    words: Array[u64]

    ## It allocates a new `Array`.
    pub static fn new() -> BitSet:
        return BitSet([])

    ## Adds `i`; whether it was absent.
    ## It allocates as the set grows, and the message of the panic a negative `i` is.
    pub fn insert(i: int) -> bool:
        if i < 0:
            panic(f"a BitSet holds non-negative integers, not {i}")
        word = i // 64
        while this.words.len() <= word:
            this.words.push(0)
        bit = 1u64 << ((i % 64) as u64)
        absent = this.words[word] & bit == 0
        this.words[word] |= bit
        return absent

    ## Removes `i`; whether it was present.
    pub fn remove(i: int) -> bool:
        if not this.contains(i):
            return false
        this.words[i // 64] &= ~(1u64 << ((i % 64) as u64))
        return true

    pub fn contains(i: int) -> bool:
        if i < 0 or i // 64 >= this.words.len():
            return false
        return this.words[i // 64] & (1u64 << ((i % 64) as u64)) != 0

    ## How many integers it holds.
    pub fn len() -> int:
        n = 0
        for w in this.words:
            n += w.count_ones() as int
        return n

    pub fn is_empty() -> bool:
        return this.len() == 0

    pub fn clear():
        this.words.clear()
