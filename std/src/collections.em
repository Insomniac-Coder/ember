## `std.collections` — fixed-capacity Arena-backed collections introduced by
## `[ARN-5]`–`[ARN-5g]`.
##
## These declarations own the public names and their concrete representation.
## Operations are compiler-known while the generic standard-library substrate
## is still being completed, exactly as `Array`, `Cell`, and `Arena` are. The
## first field is a real borrow/provenance anchor; the containers do not own
## the Arena or their backing bytes.

import std.mem

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

## `[STD-12]` (ODR-032) — what a lookup may take for a key `K`: something that
## hashes as `K` would and compares with one without converting. Every
## `K: Eq + Hash` is `AsKey[K]` (the compiler provides it: `is_key` is `==`,
## `to_key` clones).
pub interface AsKey[K]: Hash:
    fn is_key(self, key: K) -> bool
    fn to_key(self) -> K

extend str implements AsKey[String]:
    fn is_key(self, key: String) -> bool:
        return self == key

    fn to_key(self) -> String:
        return String.from(self)

## `[HASH-2]` — fixed-seed and FxHash's class: one rotate, one xor and one
## multiply per word. The exact mixer is replaceable and therefore is not part
## of Ember's source compatibility contract. Bytes go in eight to a word. No
## `Copy` derive is present: this state is moved.
pub struct DefaultHasher implements Hasher:
    state: u64

    fn new() -> DefaultHasher:
        return DefaultHasher(0)

    @overflow(wrap)
    fn mix(mut self, word: u64):
        self.state = (((self.state << 5) | (self.state >> 59)) ^ word) * 0x517cc1b727220a95

    fn write_bytes(mut self, bytes: Span[u8]):
        index = 0
        while index + 8 <= bytes.len():
            word: u64 = 0
            for k in range(8):
                word = word | ((bytes[index + k] as u64) << ((8 * k) as u64))
            self.mix(word)
            index = index + 8
        if index < bytes.len():
            word: u64 = 0
            k = 0
            while index + k < bytes.len():
                word = word | ((bytes[index + k] as u64) << ((8 * k) as u64))
                k = k + 1
            self.mix(word)

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
## It is a power of two long, probed linearly from the hash's high bits, and
## at most seven eighths used, so a probe always meets an empty place and
## ends (`[HASH-3]`).
pub struct Map[K: Eq + Hash, V, H: Hasher + Default = DefaultHasher]:
    entries: Array[Option[MapSlot[K, V]]] = []
    slots: Array[int] = []
    live: int = 0
    used: int = 0
    shift: u64 = 64

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
        return self.slots.len() * 7 // 8

    pub fn clear(mut self):
        self.entries.clear()
        self.slots.clear()
        self.live = 0
        self.used = 0
        self.shift = 64

    ## Room for `n` more entries. A rebuild leaves at least half the usable
    ## room free, so at a steady load near the limit removed places are
    ## dropped only after as many operations again: insertion stays expected
    ## constant time, amortised (`[STD-11]`, ODR-033).
    pub fn reserve(mut self, n: int):
        if (self.used + n) * 8 <= self.slots.len() * 7:
            return
        size = 8
        while size * 7 < (self.live + n) * 16:
            size = size * 2
        self.rebuild(size)

    fn hash_of[Q: AsKey[K] + Hash](self, q: Q) -> u64:
        h = H.default()
        q.hash(h)
        return h.finish()

    ## Where in `slots` the entry keyed `q` is listed.
    fn find[Q: AsKey[K] + Hash](self, q: Q, hash: u64) -> Option[int]:
        if self.slots.len() == 0:
            return None
        mask = self.slots.len() - 1
        at = (hash >> self.shift) as int
        while true:
            slot = self.slots[at]
            if slot == -1:
                return None
            if slot >= 0:
                match self.entries[slot]:
                    Some(e):
                        if e.hash == hash and q.is_key(e.key):
                            return Some(at)
                    None:
                        pass
            at = (at + 1) & mask
        return None

    ## Lists entry `i` in `slots`, in the first free place from its home.
    fn place(mut self, hash: u64, i: int):
        mask = self.slots.len() - 1
        at = (hash >> self.shift) as int
        while self.slots[at] >= 0:
            at = (at + 1) & mask
        if self.slots[at] == -1:
            self.used += 1
        self.slots[at] = i

    ## Closes up removed entries, then lists every entry in a table of `size`.
    fn rebuild(mut self, size: int):
        if self.entries.len() != self.live:
            self.entries.retain(fn(e) => e.is_some())
        self.slots.clear()
        for _ in range(size):
            self.slots.push(-1)
        bits = 0
        while (1 << bits) < size:
            bits += 1
        self.shift = (64 - bits) as u64
        self.used = 0
        for i in range(self.entries.len()):
            hash: u64 = 0
            match self.entries[i]:
                Some(e):
                    hash = e.hash
                None:
                    pass
            self.place(hash, i)

    fn push_new(mut self, hash: u64, owned k: K, owned v: V):
        self.reserve(1)
        at = self.entries.len()
        self.entries.push(Some(MapSlot(hash, k, v)))
        self.place(hash, at)
        self.live += 1

    pub fn insert(mut self, owned k: K, owned v: V) -> Option[V]:
        hash = self.hash_of(k)
        match self.find(k, hash):
            Some(at):
                match self.entries[self.slots[at]]:
                    Some(ref mut e):
                        return Some(mem.replace(e.value, v))
                    None:
                        panic("a listed entry was removed")
            None:
                pass
        self.push_new(hash, k, v)
        return None

    pub fn get[Q: AsKey[K] + Hash](self, q: Q) -> Option[ref V]:
        match self.find(q, self.hash_of(q)):
            Some(at):
                match self.entries[self.slots[at]]:
                    Some(e):
                        return Some(ref e.value)
                    None:
                        return None
            None:
                return None

    pub fn get_mut[Q: AsKey[K] + Hash](mut self, q: Q) -> Option[ref mut V]:
        match self.find(q, self.hash_of(q)):
            Some(at):
                match self.entries[self.slots[at]]:
                    Some(ref mut e):
                        return Some(ref mut e.value)
                    None:
                        return None
            None:
                return None

    pub fn contains_key[Q: AsKey[K] + Hash](self, q: Q) -> bool:
        return self.find(q, self.hash_of(q)).is_some()

    ## `k in m` (`[STD-8]`): by key.
    pub fn contains[Q: AsKey[K] + Hash](self, q: Q) -> bool:
        return self.find(q, self.hash_of(q)).is_some()

    ## `m[q]` (`[TYP-20]`, `[STD-16]`): the value, or a panic naming the key.
    pub fn index[Q: AsKey[K] + Hash + Debug](self, q: Q) -> ref V:
        match self.find(q, self.hash_of(q)):
            Some(at):
                match self.entries[self.slots[at]]:
                    Some(e):
                        return ref e.value
                    None:
                        panic("a listed entry was removed")
            None:
                panic(f"key not found: {q!r}; use .get(k) for an Option")

    ## `m[q] op= v` (`[STD-17]`): the key must be there.
    pub fn index_mut[Q: AsKey[K] + Hash + Debug](mut self, q: Q) -> ref mut V:
        match self.find(q, self.hash_of(q)):
            Some(at):
                match self.entries[self.slots[at]]:
                    Some(ref mut e):
                        return ref mut e.value
                    None:
                        panic("a listed entry was removed")
            None:
                panic(f"key not found: {q!r}; use .get(k) for an Option")

    ## `m[q] = v` (`[STD-17]`): replaces, or inserts `q.to_key()` when the
    ## key is new (`[STD-12]`).
    pub fn index_set[Q: AsKey[K] + Hash](mut self, q: Q, owned v: V):
        hash = self.hash_of(q)
        match self.find(q, hash):
            Some(at):
                match self.entries[self.slots[at]]:
                    Some(ref mut e):
                        e.value = v
                        return
                    None:
                        panic("a listed entry was removed")
            None:
                pass
        self.push_new(hash, q.to_key(), v)

    pub fn remove[Q: AsKey[K] + Hash](mut self, q: Q) -> Option[V]:
        match self.find(q, self.hash_of(q)):
            Some(at):
                i = self.slots[at]
                self.slots[at] = -2
                old = mem.replace(self.entries[i], None)
                self.live -= 1
                if self.entries.len() > 16 and self.live * 2 < self.entries.len():
                    self.rebuild(self.slots.len())
                match owned old:
                    Some(e):
                        return Some(e.value)
                    None:
                        return None
            None:
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
        self.rebuild(self.slots.len())

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
        mask = self.slots.len() - 1
        at = (hash >> self.shift) as int
        while self.slots[at] != i:
            at = (at + 1) & mask
        self.slots[at] = -2

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
        match self.map.find(self.key, self.hash):
            Some(at):
                i = self.map.slots[at]
            None:
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
        match self.map.find(self.key, self.hash):
            Some(at):
                i = self.map.slots[at]
                match self.map.entries[i]:
                    Some(ref mut e):
                        return ref mut e.value
                    None:
                        panic("a listed entry was removed")
            None:
                return self.or_insert(make())

extend[K: Eq + Hash, V: Default, H: Hasher + Default] MapEntry[K, V, H]:
    pub fn or_default(owned self) -> ref mut V:
        i = 0
        match self.map.find(self.key, self.hash):
            Some(at):
                i = self.map.slots[at]
            None:
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
        self.map.rebuild(self.map.slots.len())

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

    pub fn bitor(self, other: Set[T, H]) -> Set[T, H]:
        return self.union(other)

    pub fn bitand(self, other: Set[T, H]) -> Set[T, H]:
        return self.intersection(other)

    pub fn sub(self, other: Set[T, H]) -> Set[T, H]:
        return self.difference(other)

    pub fn bitxor(self, other: Set[T, H]) -> Set[T, H]:
        return self.symmetric_difference(other)

extend[T: Eq + Hash + Ord + Clone, H: Hasher + Default] Set[T, H]:
    pub fn sorted_elements(self) -> Array[T]:
        out: Array[T] = []
        for x in self.iter():
            out.push(x.clone())
        out.sort()
        return out

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

## `[STD-15]` (ODR-031) — `windows(n)`: every run of `n` neighbours, one step
## apart. Shared views only: overlapping mutable ones would alias. The fields
## are `SpanChunks`'s, in its order, because construction is lowered the same.
@view
pub struct SpanWindows[T]:
    source: Span[T]
    index: usize
    width: usize

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
