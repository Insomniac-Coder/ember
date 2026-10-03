#$ test: compile-fail
#$ rules: TXT-11, BRW-3, BCK-5

fn erase(mut text: String) -> int:
    text.clear()
    return 0

fn main():
    text: String = "abc"
    text.insert(erase(text), 'x')    #$ error[E3022]: `text` is already mutably borrowed
