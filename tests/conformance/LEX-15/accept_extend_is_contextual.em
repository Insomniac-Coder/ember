#$ test: run-pass
#$ rules: LEX-15, GRM-34
#$ stdout:
#$ 3 [1, 2, 5, 6]
#$ 9 4
# ODR-030 — `extend` is a keyword only where an item begins with the type it
# extends, directly or after generic parameters. Anywhere else it is a name:
# a field, a method, a variable, even one indexed at the top of a script.

struct Tally:
    extend: i64

extend Tally:
    fn extend(mut self, by: i64):
        self.extend = self.extend + by

class Holder[T]:
    value: T

extend[T] Holder[T]:
    fn get(self) -> T:
        return self.value

t = Tally(1)
t.extend(2)
extend = [1, 2, 3, 4]
extend[2] = 5
extend[3] = 6
println(t.extend, extend)
h = Holder(9)
println(h.get(), extend.len())
