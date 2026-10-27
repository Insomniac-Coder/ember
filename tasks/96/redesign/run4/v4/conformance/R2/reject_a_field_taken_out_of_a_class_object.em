#$ test: compile-fail
#$ rules: R2
#$ help: take it and leave a value in its place: write `names.push(mem.take(ref mut q.name))`, with `import std.mem` at the top
## R2: a field of a class object is not handed out on its own, since other names may reach the object
## and would see the hole (E3013; verdict 78, today's `EXP-6/reject_moving_out_of_a_class_field.em`);
## `mem.take` takes it and leaves a value in its place.

class Player:
    name: String

fn main():
    q = Player(name=String("ann"))
    names: Array[String] = []
    names.push(q.name)         #$ error[E3013]: `q` is a `Player` object, which other names may reach, so `q.name` cannot be taken out and leave a hole
    println(names.len())
