#$ test: compile-fail
#$ rules: CLS-7a, OBJ-5
#$ profiles: debug, development, release

# D-562: a store through a class handle held in a field of `self` lands in
# that other object, which outlives the drop, even wrapped in a value.
class Keeper:
    saved: Option[Item] = None #$ warning[L3001]: potential reference cycle

class Item:
    keeper: Keeper

    fn drop(mut self):
        self.keeper.saved = Some(self) #$ error[E3016]: `self` escapes its own drop

fn main():
    _item = Item(Keeper())
