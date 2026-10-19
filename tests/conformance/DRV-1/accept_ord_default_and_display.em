#$ test: run-pass
#$ rules: DRV-1, STR-5
#$ profiles: debug, development
#$ stdout: true false true
#$ stdout: Less Greater Equal
#$ stdout: true true false
#$ stdout: [Version(major=1, minor=2, tag='a'), Version(major=1, minor=10, tag=''), Version(major=2, minor=0, tag='')]
#$ stdout: Version(major=0, minor=7, tag='') Mode.Slow 0 '' []
#$ stdout: Fast Shape.Circle(r=1.5) Version(major=3, minor=1, tag='rc')
# `[DRV-1]` — `Ord` compares the fields in declaration order, the next only when the one before
# is `Equal`, and an enum's variants by their order before their fields; `Default` is each
# field's declared default, else its type's, and an enum's `@default` variant; `Display` is a
# unit-only enum's variant name and the `Debug` text of anything else. `Eq` and `Debug` are
# implicit (`[STR-5]`) and may be written too.

@derive(Ord, Default, Display, Eq, Debug)
struct Version:
    major: int
    minor: int = 7
    tag: String

@derive(Ord, Default, Display)
enum Mode:
    Fast
    @default
    Slow

@derive(Ord, Display)
enum Shape:
    Circle(r: float)
    Square(side: float)

@derive(Default)
struct Settings:
    level: int
    name: String
    seen: Array[int]

fn main():
    a = Version(major=1, minor=2, tag="a")
    b = Version(major=1, minor=10, tag="")
    c = Version(major=2, minor=0, tag="")
    println(a < b, b < a, a <= a)
    println(a.cmp(b), c.cmp(b), b.cmp(b))
    println(Mode.Fast < Mode.Slow, Shape.Circle(1.0) < Shape.Square(0.5), Shape.Circle(2.0) < Shape.Circle(1.0))
    versions = [c, b, a]
    versions.sort()
    println(versions)
    d = Version.default()
    s = Settings.default()
    println(f"{d!r}", f"{Mode.default()!r}", s.level, f"{s.name!r}", s.seen)
    println(Mode.Fast, Shape.Circle(1.5), Version(major=3, minor=1, tag="rc"))
