#$ test: run-pass
#$ rules: UNS-6
#$ stdout: 7
# `[UNS-6]` — `asm` is no keyword: inline assembly is refused (`E5090`) only where `asm` names
# nothing of the program's; a function the program calls `asm` is an ordinary function.

fn asm(x: int) -> int:
    return x + 4

fn main():
    println(asm(3))
