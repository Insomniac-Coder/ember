from std.core import NonZero
#$ test: run-pass
#$ rules: TYP-13
#$ stdout: 8 8 8 8
#$ 16 16 16
#$ 1 4
#$ 1 2
#$ 4 4
#$ 8 16
## `[TYP-13]` - `Option[T]` has the size of `T` when `T` has a niche: a class object, an `owns T`, a `ref`, a
## `ref mut`, a `Span`, a `MutSpan`, a `StrView`, `bool`, `char`, a range type that leaves a value unused, a
## `NonZero`, a `CStrView`; and one more word when `T` has none (`int`, a range type that fills its integer).

class K:
    n: int

type Pct = u8 in 0 ..= 100
type Full = u8 in 0 ..= 255

fn main():
    println(size_of[Option[K]](), size_of[Option[owns int]](), size_of[Option[ref int]](), size_of[Option[ref mut int]]())
    println(size_of[Option[Span[int]]](), size_of[Option[MutSpan[int]]](), size_of[Option[StrView]]())
    println(size_of[Option[bool]](), size_of[Option[char]]())
    println(size_of[Option[Pct]](), size_of[Option[Full]]())
    println(size_of[Option[NonZero[u32]]](), size_of[NonZero[u32]]())
    println(size_of[Option[CStrView]](), size_of[Option[int]]())
