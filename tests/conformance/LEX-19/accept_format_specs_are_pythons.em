#$ test: run-pass
#$ rules: LEX-19
#$ profiles: debug, release, shipping
#$ stdout: [   1234567] [**1234567**] [1,234,567] [1_234_567] [0,001,234,567]
#$ [ff] [0XFF] [101] [0o10] [-42] [-0007]
#$ [3.14] [     3.142] [3.141590e+00] [314.16%] [1,234.5] [1e+02]
#$ [   Ann] [  Ann  ] [An] ['Ann'] ["it's"]
#$ n=1234567 name='Ann' x = 3.1 [  true] [ z ]
# The format spec is Python's mini-language, `{x=}` writes the source text
# first, and `{x!r}` the `Debug` text; every line here is what Python prints.

fn main():
    n = 1234567
    x = 3.14159
    name = "Ann"
    s: String = "it's"
    println(f"[{n:>10}] [{n:*^11}] [{n:,}] [{n:_}] [{n:012,}]")
    println(f"[{255:x}] [{255:#X}] [{5:b}] [{8:#o}] [{-42:+}] [{-7:05}]")
    println(f"[{x:.2f}] [{x:10.3f}] [{x:e}] [{x:.2%}] [{1234.5:,.1f}] [{100.0:.3}]")
    println(f"[{name:>6}] [{name:^7}] [{name:.2}] [{name!r}] [{s!r}]")
    println(f"{n=} {name=} {x = :.1f} [{true:>6}] [{'z':^3}]")
