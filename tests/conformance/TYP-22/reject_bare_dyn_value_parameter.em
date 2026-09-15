#$ test: compile-fail
#$ rules: TYP-22

interface Drawable:
    fn draw(self):
        pass

fn take(x: dyn Drawable): #$ error[E2020]: unsized interface value cannot be used by value
    pass

