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
    fn write_bytes(mut self, bytes: Span[u8])
    fn write_u8(mut self, x: u8)
    fn write_u16(mut self, x: u16)
    fn write_u32(mut self, x: u32)
    fn write_u64(mut self, x: u64)
    fn write_i8(mut self, x: i8)
    fn write_i16(mut self, x: i16)
    fn write_i32(mut self, x: i32)
    fn write_i64(mut self, x: i64)
    fn write_usize(mut self, x: usize)
    fn write_isize(mut self, x: isize)
    fn finish(owned self) -> u64

pub interface Hash:
    fn hash[H: Hasher](self, mut h: H)

## Canonical scalar implementations feed the matching-width operation into
## the supplied protocol. They intentionally do not depend on the concrete
## `DefaultHasher` mixer.
extend u8 implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_u8(self)

extend u16 implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_u16(self)

extend u32 implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_u32(self)

extend u64 implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_u64(self)

extend usize implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_usize(self)

extend i8 implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_i8(self)

extend i16 implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_i16(self)

extend i32 implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_i32(self)

extend i64 implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_i64(self)

extend isize implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_isize(self)

extend bool implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        if self:
            h.write_u8(1)
        else:
            h.write_u8(0)

extend char implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_u32(self as u32)

extend i128 implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_u64(self as u64)
        h.write_u64((self >> 64) as u64)

extend u128 implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_u64(self as u64)
        h.write_u64((self >> 64) as u64)

## G8-4 — its low 128 bits, then whether it is past a `u128` (a `u256` has no shifts to reach its
## high bits with).
extend u256 implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        low = self as u128
        h.write_u64(low as u64)
        h.write_u64((low >> 64) as u64)
        if self > (u128.MAX as u256):
            h.write_u8(2)
        else:
            h.write_u8(0)

## G8-4 — its low 128 bits, then where it lies against them: below zero, within a `u128`, or
## above one (an `i256` has no shifts to reach its high bits with).
extend i256 implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        low = self as u128
        h.write_u64(low as u64)
        h.write_u64((low >> 64) as u64)
        if self < 0:
            h.write_u8(1)
        elif self > (u128.MAX as i256):
            h.write_u8(2)
        else:
            h.write_u8(0)

## `[TYP-36]`, `[STD-12]` — text hashes its bytes, then a byte no UTF-8 text
## contains, so `("ab", "c")` and `("a", "bc")` feed different streams. A
## `String` hashes exactly as the `str` it holds.
extend str implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_bytes(self.as_bytes())
        h.write_u8(255)

extend String implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_bytes(self.as_bytes())
        h.write_u8(255)

## A sequence hashes its length, then its elements: `[[1], [2, 3]]` and
## `[[1, 2], [3]]` differ. A fixed array hashes as its `Span`.
extend[T: Hash] Span[T] implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_usize(self.len() as usize)
        for x in self:
            x.hash(h)

extend[T: Hash] Array[T] implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_usize(self.len() as usize)
        for x in self:
            x.hash(h)

extend[T: Hash] Option[T] implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        match self:
            Some(x):
                h.write_u8(1)
                x.hash(h)
            None:
                h.write_u8(0)

extend[T: Hash, E: Hash] Result[T, E] implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        match self:
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
pub interface AsKey[K]: Hash:
    fn is_key(self, key: K) -> bool

## `[STD-12]` (ODR-067) — a lookup key that can also make the key it matches,
## for `m[q] = v` when `q` is new. Every `K: Eq + Hash + Clone` is `ToKey[K]`
## (the compiler provides it: `to_key` clones).
pub interface ToKey[K]: AsKey[K]:
    fn to_key(self) -> K

extend str implements AsKey[String], ToKey[String]:
    fn is_key(self, key: String) -> bool:
        return self == key

    fn to_key(self) -> String:
        return String.from(self)

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

    pub fn new() -> DefaultHasher:
        return DefaultHasher(0)

    @overflow(wrap)
    fn mix(mut self, word: u64):
        spread = self.state * 0x517cc1b727220a95
        self.state = ((spread << 5) | (spread >> 59)) ^ word

    fn write_bytes(mut self, bytes: Span[u8]):
        index = 0
        while index + 8 <= bytes.len():
            self.mix(full_word(bytes, index))
            index = index + 8
        if index < bytes.len():
            self.mix(tail_word(bytes, index))

    fn write_u8(mut self, x: u8):
        self.mix(x as u64)

    fn write_u16(mut self, x: u16):
        self.mix(x as u64)

    fn write_u32(mut self, x: u32):
        self.mix(x as u64)

    fn write_u64(mut self, x: u64):
        self.mix(x)

    fn write_i8(mut self, x: i8):
        self.mix(x as u64)

    fn write_i16(mut self, x: i16):
        self.mix(x as u64)

    fn write_i32(mut self, x: i32):
        self.mix(x as u64)

    fn write_i64(mut self, x: i64):
        self.mix(x as u64)

    fn write_usize(mut self, x: usize):
        self.mix(x as u64)

    fn write_isize(mut self, x: isize):
        self.mix(x as u64)

    fn finish(owned self) -> u64:
        return self.state

extend DefaultHasher implements Default:
    fn default() -> DefaultHasher:
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

    pub fn new() -> RandomState:
        k0 = process_key(0)
        k1 = process_key(1)
        return RandomState(k0 ^ 0x736f6d6570736575, k1 ^ 0x646f72616e646f6d, k0 ^ 0x6c7967656e657261, k1 ^ 0x7465646279746573, 0)

    @overflow(wrap)
    fn round(mut self):
        self.v0 = self.v0 + self.v1
        self.v1 = (self.v1 << 13) | (self.v1 >> 51)
        self.v1 = self.v1 ^ self.v0
        self.v0 = (self.v0 << 32) | (self.v0 >> 32)
        self.v2 = self.v2 + self.v3
        self.v3 = (self.v3 << 16) | (self.v3 >> 48)
        self.v3 = self.v3 ^ self.v2
        self.v0 = self.v0 + self.v3
        self.v3 = (self.v3 << 21) | (self.v3 >> 43)
        self.v3 = self.v3 ^ self.v0
        self.v2 = self.v2 + self.v1
        self.v1 = (self.v1 << 17) | (self.v1 >> 47)
        self.v1 = self.v1 ^ self.v2
        self.v2 = (self.v2 << 32) | (self.v2 >> 32)

    @overflow(wrap)
    fn mix(mut self, word: u64):
        self.v3 = self.v3 ^ word
        self.round()
        self.v0 = self.v0 ^ word
        self.words = self.words + 1

    fn write_bytes(mut self, bytes: Span[u8]):
        index = 0
        while index + 8 <= bytes.len():
            self.mix(full_word(bytes, index))
            index = index + 8
        if index < bytes.len():
            self.mix(tail_word(bytes, index))

    fn write_u8(mut self, x: u8):
        self.mix(x as u64)

    fn write_u16(mut self, x: u16):
        self.mix(x as u64)

    fn write_u32(mut self, x: u32):
        self.mix(x as u64)

    fn write_u64(mut self, x: u64):
        self.mix(x)

    fn write_i8(mut self, x: i8):
        self.mix(x as u64)

    fn write_i16(mut self, x: i16):
        self.mix(x as u64)

    fn write_i32(mut self, x: i32):
        self.mix(x as u64)

    fn write_i64(mut self, x: i64):
        self.mix(x as u64)

    fn write_usize(mut self, x: usize):
        self.mix(x as u64)

    fn write_isize(mut self, x: isize):
        self.mix(x as u64)

    @overflow(wrap)
    fn finish(owned self) -> u64:
        s = self
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
    fn default() -> RandomState:
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
struct MapSlot[K, V]:
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
pub struct Map[K: Eq + Hash, V, H: Hasher + Default = DefaultHasher]:
    entries: Array[Option[MapSlot[K, V]]] = []
    slots: Array[i32] = []
    wide_slots: Array[int] = []
    wide: bool = false
    live: int = 0
    used: int = 0
    shift: u64 = 64
    use_multiply: bool = false

    ## `Map[K, V]()` is an empty map (`[STR-6]`: every field has a default).
    pub fn init(self):
        pass

    pub fn with_capacity(n: int) -> Self:
        m = Self()
        m.reserve(n)
        return m

    pub fn len(self) -> int:
        return self.live

    pub fn is_empty(self) -> bool:
        return self.live == 0

    ## How many entries fit before the table grows.
    pub fn capacity(self) -> int:
        return self.slot_count() * 7 // 8

    pub fn clear(mut self):
        self.entries.clear()
        self.slots.clear()
        self.wide_slots.clear()
        self.wide = false
        self.live = 0
        self.used = 0
        self.shift = 64
        self.use_multiply = false

    ## Room for `n` more entries. A rebuild leaves at least half the usable
    ## room free, so at a steady load near the limit removed places are
    ## dropped only after as many operations again: insertion stays expected
    ## constant time, amortised (`[STD-11]`, ODR-033).
    pub fn reserve(mut self, n: int):
        if (self.used + n) * 8 <= self.slot_count() * 7:
            return
        size = 8
        while size * 7 < (self.live + n) * 16:
            size = size * 2
        self.rebuild(size)

    fn hash_of[Q: AsKey[K] + Hash](self, q: Q) -> u64:
        h = H.default()
        q.hash(h)
        return h.finish()

    ## The entry number at position `at`, from whichever list the table is in.
    fn slot_at(self, at: int) -> int:
        if self.wide:
            return self.wide_slots[at]
        return self.slots[at] as int

    fn set_slot(mut self, at: int, value: int):
        if self.wide:
            self.wide_slots[at] = value
        else:
            self.slots[at] = value as i32

    ## The table's length.
    fn slot_count(self) -> int:
        if self.wide:
            return self.wide_slots.len()
        return self.slots.len()

    ## Which position in `slots` a key starts at.
    @overflow(wrap)
    fn home(self, hash: u64, places: int) -> int:
        if self.use_multiply:
            return ((hash * 0x517cc1b727220a95) >> self.shift) as int
        return (hash % (places as u64)) as int

    ## The table position of `q`, or -1 when absent. Positions are nonnegative,
    ## so this private result fits one integer (ADR-144). Public lookups still
    ## return Option values or checked references as their contracts require.
    fn find[Q: AsKey[K] + Hash](self, q: Q, hash: u64) -> int:
        places = self.slot_count()
        if places == 0:
            return -1
        at = self.home(hash, places)
        if self.wide:
            while true:
                slot = self.wide_slots[at]
                if slot == -1:
                    return -1
                if slot >= 0:
                    match self.entries[slot]:
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
                slot = (self.slots[at] as int)
                if slot == -1:
                    return -1
                if slot >= 0:
                    match self.entries[slot]:
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
    fn place(mut self, hash: u64, i: int) -> int:
        places = self.slot_count()
        at = self.home(hash, places)
        passed = 0
        if self.wide:
            while self.wide_slots[at] >= 0:
                at += 1
                if at == places:
                    at = 0
                passed += 1
        else:
            while (self.slots[at] as int) >= 0:
                at += 1
                if at == places:
                    at = 0
                passed += 1
        if self.slot_at(at) == -1:
            self.used += 1
        self.set_slot(at, i)
        return passed

    ## Closes up removed entries, then lists every entry in a table of `size`.
    fn rebuild(mut self, size: int):
        if self.entries.len() != self.live:
            self.entries.retain(fn(e) => e.is_some())
        self.slots.clear()
        bits = 0
        while (1 << bits) < size:
            bits += 1
        length = prime_at_most(size)
        if self.use_multiply:
            length = 1 << bits
        self.wide_slots.clear()
        self.wide = length > 2147483647
        if self.wide:
            for _ in range(length):
                self.wide_slots.push(-1)
        else:
            for _ in range(length):
                self.slots.push(-1)
        self.shift = (64 - bits) as u64
        self.used = 0
        for i in range(self.entries.len()):
            hash: u64 = 0
            match self.entries[i]:
                Some(e):
                    hash = e.hash
                None:
                    pass
            if self.place(hash, i) > 100 and not self.use_multiply:
                self.use_multiply = true
                self.rebuild(size)
                return

    fn push_new(mut self, hash: u64, owned k: K, owned v: V):
        self.reserve(1)
        at = self.entries.len()
        self.entries.push(Some(MapSlot(hash, k, v)))
        passed = self.place(hash, at)
        self.live += 1
        if passed > 100 and not self.use_multiply:
            self.use_multiply = true
            self.rebuild(self.slot_count())

    pub fn insert(mut self, owned k: K, owned v: V) -> Option[V]:
        hash = self.hash_of(k)
        at = self.find(k, hash)
        if at >= 0:
            match self.entries[self.slot_at(at)]:
                Some(ref mut e):
                    return Some(mem.replace(e.value, v))
                None:
                    panic("a listed entry was removed")
        self.push_new(hash, k, v)
        return None

    pub fn get[Q: AsKey[K] + Hash](self, q: Q) -> Option[ref V]:
        at = self.find(q, self.hash_of(q))
        if at >= 0:
            match self.entries[self.slot_at(at)]:
                Some(e):
                    return Some(ref e.value)
                None:
                    return None
        else:
            return None

    pub fn get_mut[Q: AsKey[K] + Hash](mut self, q: Q) -> Option[ref mut V]:
        at = self.find(q, self.hash_of(q))
        if at >= 0:
            match self.entries[self.slot_at(at)]:
                Some(ref mut e):
                    return Some(ref mut e.value)
                None:
                    return None
        else:
            return None

    pub fn contains_key[Q: AsKey[K] + Hash](self, q: Q) -> bool:
        return self.find(q, self.hash_of(q)) >= 0

    ## `k in m` (`[STD-8]`): by key.
    pub fn contains[Q: AsKey[K] + Hash](self, q: Q) -> bool:
        return self.find(q, self.hash_of(q)) >= 0

    pub fn remove[Q: AsKey[K] + Hash](mut self, q: Q) -> Option[V]:
        at = self.find(q, self.hash_of(q))
        if at >= 0:
            i = self.slot_at(at)
            self.set_slot(at, -2)
            old = mem.replace(self.entries[i], None)
            self.live -= 1
            if self.entries.len() > 16 and self.live * 2 < self.entries.len():
                self.rebuild(self.slot_count())
            match owned old:
                Some(e):
                    return Some(e.value)
                None:
                    return None
        else:
            return None

    ## Keeps the entries `keep` accepts, in order.
    pub fn retain(mut self, keep: fn(ref K, ref V) -> bool):
        for i in range(self.entries.len()):
            gone = false
            match self.entries[i]:
                Some(e):
                    gone = not keep(ref e.key, ref e.value)
                None:
                    pass
            if gone:
                self.entries[i] = None
                self.live -= 1
        self.rebuild(self.slot_count())

    ## Removes and returns the newest entry, as Python's `dict.popitem` does.
    pub fn pop_item(mut self) -> Option[(K, V)]:
        while self.entries.len() > 0:
            last = self.entries.len() - 1
            hash: u64 = 0
            match self.entries[last]:
                Some(e):
                    hash = e.hash
                None:
                    pass
            slot = self.entries.pop()
            match owned slot:
                Some(Some(e)):
                    self.forget(hash, last)
                    self.live -= 1
                    return Some((e.key, e.value))
                _:
                    pass
        return None

    ## Marks the place listing entry `i` as removed.
    fn forget(mut self, hash: u64, i: int):
        places = self.slot_count()
        at = self.home(hash, places)
        while self.slot_at(at) != i:
            at += 1
            if at == places:
                at = 0
        self.set_slot(at, -2)

    ## Inserts `other`'s entries in its order.
    pub fn update(mut self, owned other: Map[K, V, H]):
        for i in range(other.entries.len()):
            slot = mem.replace(other.entries[i], None)
            match owned slot:
                Some(e):
                    self.insert(e.key, e.value)
                None:
                    pass

    ## The entries, in order, consuming the map.
    pub fn into_items(owned self) -> Array[(K, V)]:
        out = []
        for i in range(self.entries.len()):
            slot = mem.replace(self.entries[i], None)
            match owned slot:
                Some(e):
                    out.push((e.key, e.value))
                None:
                    pass
        return out

## `for k in owned m:` (`[CTL-1]`): the keys in insertion order, owned. Each
## value is dropped as its key is taken, and the entries left when a loop ends
## early are dropped with the iterator.
pub struct MapIntoKeys[K, V]:
    entries: Array[Option[MapSlot[K, V]]]
    at: int

    pub fn next(mut self) -> Option[K]:
        while self.at < self.entries.len():
            i = self.at
            self.at += 1
            slot = mem.replace(self.entries[i], None)
            match owned slot:
                Some(e):
                    return Some(e.key)
                None:
                    pass
        return None

extend[K, V] MapIntoKeys[K, V] implements Iterator:
    type Item = K

## The key-value pairs (ODR-094); a later pair's value replaces an earlier
## one's for the same key, as `insert` does.
extend[K: Eq + Hash, V, H: Hasher + Default] Map[K, V, H] implements FromIterator[(K, V)]:
    fn from_iter[I: Iterator[Item = (K, V)]](owned it: I) -> Map[K, V, H]:
        out = Map[K, V, H]()
        for (k, v) in it:
            out.insert(k, v)
        return out

extend[K: Eq + Hash, V, H: Hasher + Default] Map[K, V, H] implements IntoIterator:
    type Item = K
    type Iter = MapIntoKeys[K, V]

    fn into_iter(owned self) -> MapIntoKeys[K, V]:
        return MapIntoKeys(self.entries, 0)

## `m[q]`, `m[q] op= v` and `m[q] = v` (`[STD-12]`, `[STD-17]`): a map is
## indexed by anything that is its key, `Q: AsKey[K]`, each such `Q` giving it
## an implementation (a blanket one, `[TYP-19]`).
extend[K: Eq + Hash, V, H: Hasher + Default, Q: AsKey[K] + Hash + Debug] Map[K, V, H] implements Index[Q], IndexMut[Q]:
    type Output = V

    ## The value, or a panic naming the key.
    @inline
    fn index(self, q: Q) -> ref V:
        at = self.find(q, self.hash_of(q))
        if at >= 0:
            match self.entries[self.slot_at(at)]:
                Some(e):
                    return ref e.value
                None:
                    panic("a listed entry was removed")
        else:
            panic(f"key not found: {q!r}; use .get(k) for an Option")

    ## `m[q] op= v`: the key must be there.
    fn index_mut(mut self, q: Q) -> ref mut V:
        at = self.find(q, self.hash_of(q))
        if at >= 0:
            match self.entries[self.slot_at(at)]:
                Some(ref mut e):
                    return ref mut e.value
                None:
                    panic("a listed entry was removed")
        else:
            panic(f"key not found: {q!r}; use .get(k) for an Option")

extend[K: Eq + Hash, V, H: Hasher + Default, Q: ToKey[K] + Hash] Map[K, V, H] implements IndexSet[Q, V]:
    ## Replaces, or inserts `q.to_key()` when the key is new (`[STD-12]`).
    fn index_set(mut self, q: Q, owned v: V):
        hash = self.hash_of(q)
        at = self.find(q, hash)
        if at >= 0:
            match self.entries[self.slot_at(at)]:
                Some(ref mut e):
                    e.value = v
                    return
                None:
                    panic("a listed entry was removed")
        self.push_new(hash, q.to_key(), v)

extend[K: Eq + Hash, V: Copy, H: Hasher + Default] Map[K, V, H]:
    pub fn get_or[Q: AsKey[K] + Hash](self, q: Q, default: V) -> V:
        match self.get(q):
            Some(v):
                return v
            None:
                return default

## `[STD-16]` — equal as sets of entries: order does not matter.
extend[K: Eq + Hash, V: Eq, H: Hasher + Default] Map[K, V, H] implements Eq:
    fn eq(self, other: Map[K, V, H]) -> bool:
        if self.live != other.live:
            return false
        for i in range(self.entries.len()):
            match self.entries[i]:
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
extend[K: Eq + Hash + Ord + Clone, V, H: Hasher + Default] Map[K, V, H]:
    pub fn sorted_keys(self) -> Array[K]:
        out: Array[K] = []
        for i in range(self.entries.len()):
            match self.entries[i]:
                Some(e):
                    out.push(e.key.clone())
                None:
                    pass
        out.sort()
        return out

extend[K: Eq + Hash, V, H: Hasher + Default] Map[K, V, H] implements Default:
    fn default() -> Map[K, V, H]:
        return Map[K, V, H]()

## `m.entry(k)`: the map, borrowed until the entry is used, and the key.
@view
pub struct MapEntry[K: Eq + Hash, V, H: Hasher + Default]:
    map: ref mut Map[K, V, H]
    key: K
    hash: u64

    ## The value, inserting `v` first when the key is new.
    pub fn or_insert(owned self, owned v: V) -> ref mut V:
        i = 0
        at = self.map.find(self.key, self.hash)
        if at >= 0:
            i = self.map.slot_at(at)
        else:
            # `push_new` may close up removed entries first, so the new
            # entry's place is read after it.
            self.map.push_new(self.hash, self.key, v)
            i = self.map.entries.len() - 1
        match self.map.entries[i]:
            Some(ref mut e):
                return ref mut e.value
            None:
                panic("a listed entry was removed")

    pub fn or_insert_with(owned self, make: fn() -> V) -> ref mut V:
        at = self.map.find(self.key, self.hash)
        if at >= 0:
            i = self.map.slot_at(at)
            match self.map.entries[i]:
                Some(ref mut e):
                    return ref mut e.value
                None:
                    panic("a listed entry was removed")
        else:
            return self.or_insert(make())

extend[K: Eq + Hash, V: Default, H: Hasher + Default] MapEntry[K, V, H]:
    pub fn or_default(owned self) -> ref mut V:
        i = 0
        at = self.map.find(self.key, self.hash)
        if at >= 0:
            i = self.map.slot_at(at)
        else:
            self.map.push_new(self.hash, self.key, V.default())
            i = self.map.entries.len() - 1
        match self.map.entries[i]:
            Some(ref mut e):
                return ref mut e.value
            None:
                panic("a listed entry was removed")

## `m.keys()`, and what `for k in m:` iterates: the keys in insertion order.
@view
pub struct MapKeys[K: Eq + Hash, V, H: Hasher + Default]:
    owner: ref Map[K, V, H]
    at: int

    pub fn next(mut self) -> Option[ref K]:
        while self.at < self.owner.entries.len():
            i = self.at
            self.at += 1
            match self.owner.entries[i]:
                Some(e):
                    return Some(ref e.key)
                None:
                    pass
        return None

@view
pub struct MapValues[K: Eq + Hash, V, H: Hasher + Default]:
    owner: ref Map[K, V, H]
    at: int

    pub fn next(mut self) -> Option[ref V]:
        while self.at < self.owner.entries.len():
            i = self.at
            self.at += 1
            match self.owner.entries[i]:
                Some(e):
                    return Some(ref e.value)
                None:
                    pass
        return None

@view
pub struct MapValuesMut[K: Eq + Hash, V, H: Hasher + Default]:
    owner: ref mut Map[K, V, H]
    at: int

    pub fn next(mut self) -> Option[ref mut V]:
        while self.at < self.owner.entries.len():
            i = self.at
            self.at += 1
            match self.owner.entries[i]:
                Some(ref mut e):
                    return Some(ref mut e.value)
                None:
                    pass
        return None

@view
pub struct MapItems[K: Eq + Hash, V, H: Hasher + Default]:
    owner: ref Map[K, V, H]
    at: int

    pub fn next(mut self) -> Option[(ref K, ref V)]:
        while self.at < self.owner.entries.len():
            i = self.at
            self.at += 1
            match self.owner.entries[i]:
                Some(e):
                    return Some((ref e.key, ref e.value))
                None:
                    pass
        return None

## `[STD-19]` — each of them is an `Iterator`, with its adapters and
## consumers.
extend[K: Eq + Hash, V, H: Hasher + Default] MapKeys[K, V, H] implements Iterator:
    type Item = ref K

extend[K: Eq + Hash, V, H: Hasher + Default] MapValues[K, V, H] implements Iterator:
    type Item = ref V

extend[K: Eq + Hash, V, H: Hasher + Default] MapValuesMut[K, V, H] implements Iterator:
    type Item = ref mut V

extend[K: Eq + Hash, V, H: Hasher + Default] MapItems[K, V, H] implements Iterator:
    type Item = (ref K, ref V)

## The methods that name the view types declared above.
extend[K: Eq + Hash, V, H: Hasher + Default] Map[K, V, H]:
    pub fn entry(mut self, owned k: K) -> MapEntry[K, V, H]:
        hash = self.hash_of(k)
        return MapEntry(ref mut self, k, hash)

    pub fn iter(self) -> MapKeys[K, V, H]:
        return MapKeys(ref self, 0)

    pub fn keys(self) -> MapKeys[K, V, H]:
        return MapKeys(ref self, 0)

    pub fn values(self) -> MapValues[K, V, H]:
        return MapValues(ref self, 0)

    pub fn values_mut(mut self) -> MapValuesMut[K, V, H]:
        return MapValuesMut(ref mut self, 0)

    pub fn items(self) -> MapItems[K, V, H]:
        return MapItems(ref self, 0)


## `[STD-11]`, `[STD-16]` (ODR-032) — a `Map` with no values: insertion
## order, expected constant-time membership.
pub struct Set[T: Eq + Hash, H: Hasher + Default = DefaultHasher]:
    map: Map[T, bool, H] = Map[T, bool, H]()

    ## `Set[T]()` is an empty set.
    pub fn init(self):
        pass

    pub fn with_capacity(n: int) -> Self:
        s = Self()
        s.map.reserve(n)
        return s

    pub fn len(self) -> int:
        return self.map.live

    pub fn is_empty(self) -> bool:
        return self.map.live == 0

    pub fn capacity(self) -> int:
        return self.map.capacity()

    pub fn reserve(mut self, n: int):
        self.map.reserve(n)

    pub fn clear(mut self):
        self.map.clear()

    ## Whether `x` was new. An element already there keeps its position.
    pub fn add(mut self, owned x: T) -> bool:
        return self.map.insert(x, true).is_none()

    pub fn remove[Q: AsKey[T] + Hash](mut self, q: Q) -> bool:
        return self.map.remove(q).is_some()

    pub fn contains[Q: AsKey[T] + Hash](self, q: Q) -> bool:
        return self.map.contains_key(q)

    pub fn retain(mut self, keep: fn(ref T) -> bool):
        for i in range(self.map.entries.len()):
            gone = false
            match self.map.entries[i]:
                Some(e):
                    gone = not keep(ref e.key)
                None:
                    pass
            if gone:
                self.map.entries[i] = None
                self.map.live -= 1
        self.map.rebuild(self.map.slot_count())

    ## Removes and returns the newest element.
    pub fn pop(mut self) -> Option[T]:
        match owned self.map.pop_item():
            Some(pair):
                return Some(pair.0)
            None:
                return None

## `s.iter()`, and what `for x in s:` iterates: the elements in order.
@view
pub struct SetIter[T: Eq + Hash, H: Hasher + Default]:
    owner: ref Map[T, bool, H]
    at: int

    pub fn next(mut self) -> Option[ref T]:
        while self.at < self.owner.entries.len():
            i = self.at
            self.at += 1
            match self.owner.entries[i]:
                Some(e):
                    return Some(ref e.key)
                None:
                    pass
        return None

extend[T: Eq + Hash, H: Hasher + Default] SetIter[T, H] implements Iterator:
    type Item = ref T

extend[T: Eq + Hash, H: Hasher + Default] Set[T, H]:
    pub fn iter(self) -> SetIter[T, H]:
        return SetIter(ref self.map, 0)

    pub fn is_subset(self, other: Set[T, H]) -> bool:
        for x in self.iter():
            if not other.contains(x):
                return false
        return true

    pub fn is_superset(self, other: Set[T, H]) -> bool:
        return other.is_subset(self)

    pub fn is_disjoint(self, other: Set[T, H]) -> bool:
        for x in self.iter():
            if other.contains(x):
                return false
        return true

## The set operations make a new set: this one's elements first, in its
## order, then the other's.
extend[T: Eq + Hash + Clone, H: Hasher + Default] Set[T, H]:
    pub fn union(self, other: Set[T, H]) -> Set[T, H]:
        out = Set[T, H]()
        for x in self.iter():
            out.add(x.clone())
        for x in other.iter():
            out.add(x.clone())
        return out

    pub fn intersection(self, other: Set[T, H]) -> Set[T, H]:
        out = Set[T, H]()
        for x in self.iter():
            if other.contains(x):
                out.add(x.clone())
        return out

    pub fn difference(self, other: Set[T, H]) -> Set[T, H]:
        out = Set[T, H]()
        for x in self.iter():
            if not other.contains(x):
                out.add(x.clone())
        return out

    pub fn symmetric_difference(self, other: Set[T, H]) -> Set[T, H]:
        out = self.difference(other)
        for x in other.iter():
            if not self.contains(x):
                out.add(x.clone())
        return out

## `s | t`, `s & t`, `s - t` and `s ^ t` are those operations (`[STD-16]`): a
## set implements the operators' interfaces (`[TYP-21]`).
extend[T: Eq + Hash + Clone, H: Hasher + Default] Set[T, H] implements BitOr, BitAnd, Sub, BitXor:
    type Output = Set[T, H]

    fn bitor(self, other: Set[T, H]) -> Set[T, H]:
        return self.union(other)

    fn bitand(self, other: Set[T, H]) -> Set[T, H]:
        return self.intersection(other)

    fn sub(self, other: Set[T, H]) -> Set[T, H]:
        return self.difference(other)

    fn bitxor(self, other: Set[T, H]) -> Set[T, H]:
        return self.symmetric_difference(other)

extend[T: Eq + Hash + Ord + Clone, H: Hasher + Default] Set[T, H]:
    pub fn sorted_elements(self) -> Array[T]:
        out: Array[T] = []
        for x in self.iter():
            out.push(x.clone())
        out.sort()
        return out

## `for x in owned s:` (`[CTL-1]`): the elements in insertion order, owned.
## The elements, each once, in the order they first came (ODR-094).
extend[T: Eq + Hash, H: Hasher + Default] Set[T, H] implements FromIterator[T]:
    fn from_iter[I: Iterator[Item = T]](owned it: I) -> Set[T, H]:
        out = Set[T, H]()
        for x in it:
            out.add(x)
        return out

extend[T: Eq + Hash, H: Hasher + Default] Set[T, H] implements IntoIterator:
    type Item = T
    type Iter = MapIntoKeys[T, bool]

    fn into_iter(owned self) -> MapIntoKeys[T, bool]:
        return self.map.into_iter()

extend[T: Eq + Hash, H: Hasher + Default] Set[T, H] implements Eq:
    fn eq(self, other: Set[T, H]) -> bool:
        return self.len() == other.len() and self.is_subset(other)

extend[T: Eq + Hash, H: Hasher + Default] Set[T, H] implements Default:
    fn default() -> Set[T, H]:
        return Set[T, H]()

pub enum CapacityError:
    Full

@view
pub struct ArenaArray[T]:
    arena: ref Arena
    data: *mut T
    length: usize
    limit: usize

@view
pub struct ArenaArrayIter[T]:
    owner: ref ArenaArray[T]
    index: usize

@view
pub struct ArenaArrayIterMut[T]:
    owner: ref mut ArenaArray[T]
    index: usize

## `[SPN-4]`–`[SPN-6]` — the public identities and representations of the
## canonical Span iterators. Each stores one reborrowed source view plus an
## advancing cursor; chunk iterators additionally retain their non-zero
## width. The compiler currently lowers `next` and construction while the
## generic standard-library substrate is completed, as for the Arena-backed
## iterators above.
@view
pub struct SpanIter[T]:
    source: Span[T]
    index: usize

@view
pub struct MutSpanIter[T]:
    source: MutSpan[T]
    index: usize

@view
pub struct SpanChunks[T]:
    source: Span[T]
    index: usize
    width: usize

@view
pub struct MutSpanChunks[T]:
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
pub struct SpanWindows[T]:
    source: Span[T]
    index: usize
    width: usize

extend[T] SpanWindows[T] implements Iterator:
    type Item = Span[T]

## Occupancy is separate from the two uninitialized carriers. This lets an
## empty fixed-capacity map reserve all backing bytes in one Arena allocation
## without manufacturing invalid `K` or `V` values.
struct ArenaMapSlot[K, V]:
    occupied: bool
    key: MaybeUninit[K]
    value: MaybeUninit[V]

@view
pub struct ArenaMap[K, V]:
    arena: ref Arena
    slots: *mut ArenaMapSlot[K, V]
    length: usize
    limit: usize

@view
pub struct ArenaMapIter[K, V]:
    owner: ref ArenaMap[K, V]
    index: usize
