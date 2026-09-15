#$ test: compile-pass
#$ rules: TYP-22

interface Drawable:
    fn draw(self):
        pass

fn take(x: ref dyn Drawable, owned boxed: Box[dyn Drawable]):
    pass

fn main():
    pass
