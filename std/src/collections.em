#! language "0.9.6"
## `std.collections` — fixed-capacity Arena-backed collections introduced by
## `[ARN-5]`–`[ARN-5g]`.
##
## These declarations own the public names and their concrete representation.
## Operations are compiler-known while the generic standard-library substrate
## is still being completed, exactly as `Array`, `Cell`, and `Arena` are. The
## first field is a real borrow/provenance anchor; the containers do not own
## the Arena or their backing bytes.

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
