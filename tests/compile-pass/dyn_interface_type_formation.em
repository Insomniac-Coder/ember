#$ test: compile-pass
#$ rules: TYP-22

interface Drawable:
    fn draw(self):
        pass

fn take(x: ref dyn Drawable):
    pass

fn main():
    pass
