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

## `[HASH-2]` — one deliberately simple deterministic implementation. The
## exact mixer is replaceable and therefore is not part of Ember's source
## compatibility contract. No `Copy` derive is present: this state is moved.
pub struct DefaultHasher implements Hasher:
    state: u64

    fn new() -> DefaultHasher:
        return DefaultHasher(1)

    fn mix(mut self, word: u64):
        self.state = ((self.state << 7) | (self.state >> 57)) ^ word

    fn write_bytes(mut self, bytes: Span[u8]):
        index = 0
        while index < bytes.len():
            self.mix(bytes[index] as u64)
            index = index + 1

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
