#$ test: run-pass
#$ rules: TYP-17, IFC-3, MOD-3
## "Inside a generic body, only operations provided by the bounds are
## permitted." The bound, the interface it names, its supertrait and the
## implementation may each live in a different module: `[MOD-4]` allows import
## cycles inside a package, so no load order makes a per-module pass correct.

from std.core import Ordering, Ord, Eq

struct Version:
    major: i32

extend Version implements Eq:
    fn eq(other: Self) -> bool:
        return this.major == other.major

extend Version implements Ord:
    fn cmp(other: Self) -> Ordering:
        if this.major < other.major: return Ordering.Less
        if this.major > other.major: return Ordering.Greater
        return Ordering.Equal

[T: needs Ord]
fn newer(a: T, b: T) -> T:
    match a.cmp(b):
        Greater => return a
        _ => return b

fn main():
    println(newer(Version(3), Version(7)).major)
#$ stdout: 7
