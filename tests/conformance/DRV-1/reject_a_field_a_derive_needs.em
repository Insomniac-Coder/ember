#$ test: compile-fail
#$ rules: DRV-1, ARN-11
# `[DRV-1]` — a derive whose requirement a field does not meet is `E2080`, naming the field: a
# class handle has no `Ord` and no `Default`; an enum's `Default` is the one unit
# variant marked `@default`. `[ARN-11]` — an enum is `Zeroable` when its variant whose discriminant
# is zero is, with every field of it.

class Sensor:
    id: int

@derive(Ord)
struct Reading:
    at: int
    source: Sensor #$ error[E2080]: field `source` has type `Sensor`, which is not Ord

@derive(Default)
struct Probe:
    sensor: Sensor #$ error[E2080]: field `sensor` has type `Sensor`, which is not Default
    count: int

@derive(Default) #$ error[E2080]: `Level` derives `Default` but marks no variant `@default`
enum Level:
    Low
    High

@derive(Zeroable)
enum Code: #$ error[E2080]: `Code` has no variant whose discriminant is zero, so zero bytes are no `Code`
    Ok = 1
    Failed = 2

@derive(Zeroable)
enum Slot:
    Empty
    Full(at: ref int)

@derive(Zeroable)
enum Bad:
    Held(at: ref int) #$ error[E2080]: payload `at` has type `ref i64`, which is not Zeroable
    Free

fn main():
    frame = Arena.with_capacity(64)
    slots = frame.alloc_zeroed[Slot](2)
    println(slots.len())
