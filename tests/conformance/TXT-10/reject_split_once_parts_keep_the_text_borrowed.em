#$ test: compile-fail
#$ rules: TXT-10, BRW-1
#$ profiles: debug, development, release

fn main():
    text = String.from("left:right")
    match text.split_once(":"):
        Some((before, after)):
            text.clear()  #$ error[E3021]: `text` is borrowed here and mutably borrowed elsewhere
            println(before, after)
        None:
            println("none")
