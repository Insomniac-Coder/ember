#$ test: compile-pass
#$ rules: TYP-22

interface Drawable:
    fn draw():
        pass

fn take(ref x: dyn Drawable):
    pass

fn main():
    pass
