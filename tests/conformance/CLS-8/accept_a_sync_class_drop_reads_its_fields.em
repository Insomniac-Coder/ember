#$ test: run-pass
#$ rules: CLS-8
#$ stdout: made 7
#$ dropped 7
# A `@sync` class may not declare a `mut self` method, so its `drop` takes `self`: the runtime
# calls it with the handle, and the body reads the object's fields (D-681: it was given the
# handle's address, and read past it).

@sync
class Tag:
    id: int = 7

    fn drop(self):
        println(f"dropped {self.id}")

fn main():
    t = Tag()
    println(f"made {t.id}")
