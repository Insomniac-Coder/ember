#$ test: run-pass
#$ rules: CLS-2, CLS-7
#$ stdout: front 2
# D-563: `[CLS-2]` spells a class constructor `fn init(self, …)`; any class
# method writes its object's fields through `self` (`[CLS-7]`).

class Door:
    name: String
    opened: int = 0

    fn init(owns name: String):
        this.name = name
        this.opened += 1

    fn swing():
        this.opened += 1

fn main():
    door = Door("front")
    door.swing()
    println(door.name, door.opened)
