#$ test: compile-pass
#$ rules: TYP-22

interface Drawable:
    fn draw():
        pass

fn take(ref x: dyn Drawable, owns boxed: owns dyn Drawable):
    pass

fn main():
    pass
