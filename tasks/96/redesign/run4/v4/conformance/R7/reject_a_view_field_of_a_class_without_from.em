#$ test: compile-fail
#$ rules: R7
#$ help: name: String
## R7: a class object is never a view; a view field of a class says what it points into (`from <let
## field>` or `from static`), and without it the field is refused (E2118). AN-26, point 6.

class Player:
    name: StrView    #$ error[E2118]: a `Player` keeps `name` after the text it came from may be gone; to keep its own copy write `name: String` and `this.name = String(name)`

    fn init(name: StrView):
        this.name = name

fn main():
    p = Player("ann")
    println(p.name)
