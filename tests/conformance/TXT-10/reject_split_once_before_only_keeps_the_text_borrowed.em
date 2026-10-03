#$ test: compile-fail
#$ rules: TXT-10, BRW-1, LT-20, LT-36
#$ profiles: debug, release, shipping
# Keep only the first field. The discarded second field cannot supply the
# source loan that this case requires, so a missing first-field contract is
# caught independently of the second-field contract.

fn main():
    text = String.from("left:right")
    match text.split_once(":"):
        Some((before, _)):
            text.clear()  #$ error[E3021]: `text` is borrowed here and mutably borrowed elsewhere
            println(before)
        None:
            println("none")
