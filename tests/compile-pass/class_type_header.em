#$ test: compile-pass
#$ rules: CLS-1, CLS-4

# Class declarations are now collected into the nominal type table.  The
# runtime and source-level construction/lowering path is still a later slice.
open class Entity:
    id: i32
    fn get_id(self) -> i32:
        return self.id

class Player(Entity):
    score: i32

fn read_id(entity: Entity) -> i32:
    return entity.id

fn read_via_method(entity: Entity) -> i32:
    return entity.get_id()

fn read_inherited_id(player: Player) -> i32:
    return player.id

fn main():
    println(1)
