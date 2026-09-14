#$ test: compile-pass
#$ rules: CLS-1, CLS-4, OBJ-1, OBJ-2
#$ assert-c: contains(const ember_type_info em_ti_Entity)
#$ assert-c: contains(const ember_type_info em_ti_Player)
#$ assert-c: contains("Entity")
#$ assert-c: contains("Player")
#$ assert-c: contains(&em_ti_Entity)
#$ assert-c: contains(ember_retain((ember_obj_header*)
#$ assert-c: contains(ember_release((ember_obj_header*)

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

fn read_inherited_method(player: Player) -> i32:
    return player.get_id()

fn retain_copy(entity: Entity) -> Entity:
    return entity

fn consume(owned entity: Entity):
    pass

fn main():
    println(1)
