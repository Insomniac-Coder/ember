## `std.collections` — fixed-capacity Arena-backed collections introduced by
## `[ARN-5]`–`[ARN-5g]`.
##
## These declarations own the public names and their concrete representation.
## Operations are compiler-known while the generic standard-library substrate
## is still being completed, exactly as `Array`, `Cell`, and `Arena` are. The
## first field is a real borrow/provenance anchor; the containers do not own
## the Arena or their backing bytes.

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
