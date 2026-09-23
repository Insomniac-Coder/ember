#$ test: run-fail
#$ rules: PAN-1
#$ profiles: debug, release, shipping
#$ panics: the fuel ran out at 3
# A panic prints its message and aborts; an f-string message is a `String`,
# borrowed as the `str` the message is.

fn main():
    left = 3
    panic(f"the fuel ran out at {left}")
