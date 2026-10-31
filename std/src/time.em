## `std.time` — the clock and spans of time (`[STD-23]`).

## `[STD-23]` — a span of time: a whole count of nanoseconds. Its arithmetic is checked, so a sum
## or product too large for it panics (`[TYP-8]`).
@derive(Copy)
pub struct Duration:
    pub nanos: int

    pub static fn from_nanos(n: int) -> Duration:
        return Duration(nanos = n)

    pub static fn from_micros(n: int) -> Duration:
        return Duration(nanos = n * 1000)

    pub static fn from_millis(n: int) -> Duration:
        return Duration(nanos = n * 1000000)

    pub static fn from_secs(n: int) -> Duration:
        return Duration(nanos = n * 1000000000)

    pub fn as_nanos() -> int:
        return this.nanos

    pub fn as_millis() -> int:
        return this.nanos // 1000000

    pub fn as_secs_f64() -> f64:
        return this.nanos as f64 / 1000000000.0

extend Duration implements Add:
    type Output = Duration

    fn add(rhs: Duration) -> Duration:
        return Duration(nanos = this.nanos + rhs.nanos)

extend Duration implements Sub:
    type Output = Duration

    fn sub(rhs: Duration) -> Duration:
        return Duration(nanos = this.nanos - rhs.nanos)

## `[STD-23]` — a point on the monotonic clock, which never goes back. Reading the clock is
## `Nondet` (`[DET-2]`).
@derive(Copy)
pub struct Instant:
    nanos: int

    ## Now.
    pub static fn now() -> Instant:
        return Instant(nanos = monotonic_now())

    ## The time from `earlier` to this instant.
    pub fn duration_since(earlier: Instant) -> Duration:
        return Duration(nanos = this.nanos - earlier.nanos)

    ## The time since this instant.
    pub fn elapsed() -> Duration:
        return Instant.now().duration_since(this)

## `[STD-23]` — waits at least `d`; it carries `Block`.
pub fn sleep(d: Duration):
    sleep_nanos(d.nanos)

## The runtime's own (`ember_hir::RuntimeFn`): a call is the runtime's, and these staging bodies
## never run.
fn monotonic_now() -> int:
    return 0

fn sleep_nanos(nanos: int):
    pass
