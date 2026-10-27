#$ test: compile-fail
#$ rules: MOD-3
#$ help: write `fn reset():` in place of line 10 and `this.count = 0` in place of line 11
## design rules: R5
## R5, rules section 3's removed words: `mut self` is gone; a method names its object `this` and changes
## it with no word (E1010; completeness G8; verdict 86).

struct Counter:
    count: int = 0

    fn reset(mut self):    #$ error[E1010]: `mut self` is not Ember: inside a method its object is `this`, which the method changes with no word
        self.count = 0

fn main():
    c = Counter(count=3)
    c.reset()
    println(c.count)
