#$ test: run-pass
#$ rules: LEX-15, LEX-14
#$ stdout: 55
#$ stdout: 155
#$ stdout: 3
#$ stdout: 6
## `[LEX-15]` — contextual keywords are keywords only in the stated position and identifiers everywhere
## else: `open` is a method of `File` (`[STD-22]`), `from` a static function, and each of the words
## `abstract`, `extend`, `gen`, `once`, `safe`, `some`, `c`, `cpp`, `language`, `threads`, `module`,
## `package`, `read`, `overlay`, `rename`, `hide`, `union`, `needs`, `copy`, `of`, `exists`, `runtime`,
## `guarded`, `by` names a variable; `cast(p)` stays a call.

struct File:
    n: int = 1

    fn open() -> int:
        return this.n

    static fn from() -> int:
        return 2

fn cast(p: int) -> int:
    return p + 1

fn main():
    abstract = 1
    extend = 2
    gen = 3
    once = 4
    safe = 5
    some = 6
    c = 7
    cpp = 8
    language = 9
    threads = 10
    println(abstract + extend + gen + once + safe + some + c + cpp + language + threads)
    module = 11
    package = 12
    read = 13
    overlay = 14
    rename = 15
    hide = 16
    union = 17
    needs = 18
    copy = 19
    of = 20
    println(module + package + read + overlay + rename + hide + union + needs + copy + of)
    exists = 1
    runtime = 1
    guarded = 1
    by = 1
    println(exists + runtime + guarded)
    f = File()
    println(f.open() + File.from() + cast(1) + by)
