## `std.random` — random numbers (`[STD-25]`).

import std.mem

## `[STD-25]` — a pseudo-random generator, xoshiro256** (Blackman and Vigna) with its state seeded
## by SplitMix64. Its arithmetic is integer only, so `Rng.seeded(seed)` gives the same sequence on
## every machine; `Rng.from_entropy()` seeds from the system's generator and is `Nondet`
## (`[DET-2]`).
pub struct Rng:
    s0: u64
    s1: u64
    s2: u64
    s3: u64

    ## The generator whose sequence `seed` fixes.
    @overflow(wrap)
    pub fn seeded(seed: u64) -> Rng:
        x = seed + 0x9E3779B97F4A7C15
        a = split_mix(x)
        x = x + 0x9E3779B97F4A7C15
        b = split_mix(x)
        x = x + 0x9E3779B97F4A7C15
        c = split_mix(x)
        x = x + 0x9E3779B97F4A7C15
        d = split_mix(x)
        return Rng(s0 = a, s1 = b, s2 = c, s3 = d)

    ## A generator seeded from the operating system's: a different sequence on each run.
    pub fn from_entropy() -> Rng:
        return Rng.seeded(entropy())

    ## The next 64 bits of the sequence.
    @overflow(wrap)
    pub fn next_u64(mut self) -> u64:
        result = rotate_left(self.s1 * 5, 7) * 9
        t = self.s1 << 17
        self.s2 = self.s2 ^ self.s0
        self.s3 = self.s3 ^ self.s1
        self.s1 = self.s1 ^ self.s2
        self.s0 = self.s0 ^ self.s3
        self.s2 = self.s2 ^ t
        self.s3 = rotate_left(self.s3, 45)
        return result

    ## An integer from `range.start` up to, not including, `range.end`, each equally likely. An
    ## empty range is the caller's bug.
    @overflow(wrap)
    pub fn int_in(mut self, range: Range[int]) -> int:
        assert(range.start < range.end, "`int_in` takes a range that is not empty")
        size = (range.end as u64) - (range.start as u64)
        # The draws below `floor` would make the low values likelier; they are drawn again.
        floor = (0 - size) % size
        r = self.next_u64()
        while r < floor:
            r = self.next_u64()
        return range.start + (r % size) as int

    ## A float in `[0, 1)`: 53 random bits, scaled exactly.
    pub fn float(mut self) -> f64:
        return (self.next_u64() >> 11) as f64 * 0.00000000000000011102230246251565

    ## One of `items`, each equally likely, or `None` when there are none.
    @borrows(items)
    pub fn choice[T](mut self, items: Span[T]) -> Option[ref T]:
        if items.len() == 0:
            return None
        return Some(ref items[self.int_in(0..items.len())])

    ## `items` put in an order chosen uniformly at random (Fisher and Yates).
    pub fn shuffle[T](mut self, mut items: MutSpan[T]):
        i = items.len() - 1
        while i > 0:
            j = self.int_in(0..i + 1)
            if j < i:
                parts = items.split_at(i)
                head = parts.0
                tail = parts.1
                mem.swap(head[j], tail[0])
            i = i - 1

@overflow(wrap)
fn split_mix(state: u64) -> u64:
    z = state
    z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9
    z = (z ^ (z >> 27)) * 0x94D049BB133111EB
    return z ^ (z >> 31)

fn rotate_left(x: u64, k: u64) -> u64:
    return (x << k) | (x >> (64 - k))

## The runtime's own (`ember_hir::RuntimeFn`): 64 bits from the operating system's generator. This
## staging body never runs.
fn entropy() -> u64:
    return 0
