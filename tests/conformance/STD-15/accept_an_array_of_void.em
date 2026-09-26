#$ test: run-pass
#$ rules: STD-15, TYP-1, TYP-36
#$ stdout: 5 [(), (), (), (), ()]
#$ stdout: 10 true false true
#$ stdout: Some(()) Some(()) 3
#$ stdout: 4 true
#$ stdout: [] None 0 true
#$ assert-c: !contains("sizeof(void)")
#$ assert-c: !contains("_Alignof(void)")
# `[TYP-1]` — `void` is zero-sized, so an `Array[void]` stores nothing: its elements have no
# storage and every one is at the buffer's base (ODR-068). Every `Array` method works on it,
# and it compares, sorts and prints (`[TYP-36]`: `void` is `Eq` and `Ord`, and its `Debug` is
# `()`). C has no `sizeof(void)` (GCC and Clang say 1, MSVC 0), so the backend never writes it
# (D-355).

fn unit():
    pass

fn main():
    units: Array[void] = [(), (), ()]
    units.push(unit())
    units.insert(0, ())
    println(len(units), units)
    n = 0
    for u in units:
        if u == ():
            n += 1
    for u in units.as_span().iter():
        if u == ():
            n += 1
    x = units[1]
    other = [(), (), (), (), ()]
    println(n, units == other, units != other, x == ())
    o = Some(())
    units.remove(0)
    println(o, units.pop(), len(units))
    units.swap_remove(0)
    copy = units.clone()
    units.extend(copy)
    units.sort()
    units.reverse()
    println(len(units), () in units)
    units.truncate(1)
    units.clear()
    last = units.pop()
    println(units, last, mem.size_of[void](), Box(()) == Box(()))
