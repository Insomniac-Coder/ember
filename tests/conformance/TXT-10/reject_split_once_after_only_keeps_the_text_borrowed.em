#$ test: compile-fail
#$ rules: TXT-10, BRW-1, LT-20, LT-36
#$ profiles: debug, release, shipping
# Keep only the second field. The discarded first field cannot supply the
# source loan that this case requires, so a missing second-field contract is
# caught independently of the first-field contract.

fn main():
    text = String.from("left:right")
    match text.split_once(":"):
        Some((_, after)):
            text.clear()  #$ error[E3021]: `text` is borrowed here and mutably borrowed elsewhere
            println(after)
        None:
            println("none")
