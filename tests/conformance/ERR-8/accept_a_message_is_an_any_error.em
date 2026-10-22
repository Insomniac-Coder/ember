#$ test: run-pass
#$ rules: ERR-8, ERR-9
#$ profiles: debug, development, release
#$ stdout:
#$ 3
#$ -1 is negative
#$ too big
# `[ERR-8]` (ODR-180) — `AnyError.msg(text)` makes a failure of a message, what Python's
# `raise ValueError(text)` means; its text is the message.

fn check(x: int) -> Result[int]:
    if x < 0:
        return Err(AnyError.msg(f"{x} is negative"))
    if x > 9:
        return Err(AnyError.msg("too big"))
    return Ok(x)

fn main():
    for x in [3, -1, 12]:
        match check(x):
            Ok(v):
                println(v)
            Err(e):
                println(e)
