## `std.ecs` — the entity-component system (Part XII.3, `[ECS-1]`..`[ECS-7]`).
##
## An ordinary library: a `World` holds one sparse-set storage per component type, keyed by the
## type's identity (`type_hash`). `@derive(Component)` writes the type's identity and layout as an
## `extend … implements Component` block, as every derive writes ordinary Ember (`[DRV-1]`), and
## adds the type to the component registry (`components()`). Queries, systems (`World.run`) and
## the `Commands` buffer (`[ECS-3]`..`[ECS-5]`) are not built yet: see tasks/85/D-notes.md.

from std.collections import Map

## `[ECS-1]` — a 32-bit generational handle: a 20-bit index and a 12-bit generation, the compact
## split engines use for entity ids. `Entity.NULL` is all ones and names no entity.
@derive(Copy, Eq, Hash, Debug)
pub struct Entity:
    bits: u32

    pub const NULL: Entity = Entity(4294967295u32)
    ## How many indices there are: the last, all ones, is `NULL`'s.
    pub const INDICES: u32 = 1048576u32

    ## The entity of index `index` (below 2^20 - 1) and generation `generation` (below 2^12).
    pub fn new(index: u32, generation: u32) -> Entity:
        if index >= Entity.INDICES - 1u32 or generation >= 4096u32:
            panic("an entity's index is below 2^20 - 1 and its generation below 2^12")
        return Entity((generation << 20u32) | index)

    pub fn index(self) -> u32:
        return self.bits & (Entity.INDICES - 1u32)

    pub fn generation(self) -> u32:
        return self.bits >> 20u32

    pub fn to_bits(self) -> u32:
        return self.bits

    pub fn is_null(self) -> bool:
        return self.bits == Entity.NULL.bits

## `[ECS-2]`, `[ECS-7]` — what `@derive(Component)` implements: the type's fully qualified name,
## its identity (the 64-bit FNV-1a hash of that name) and whether `@component(layout=soa)` stores
## it as an `SoA`.
pub interface Component:
    fn component_name() -> str
    fn component_hash() -> u64
    fn component_soa() -> bool

## `[ECS-2]` — a component type's identity: the 64-bit FNV-1a hash of its fully qualified name, the
## same in every module and every build.
pub fn type_hash[T: Component]() -> u64:
    return T.component_hash()

## `[ECS-2]` — the 64-bit FNV-1a hash of `text`'s bytes (`type_hash`'s function).
pub fn fnv1a(text: str) -> u64:
    hash = 14695981039346656037u64
    for byte in text.bytes():
        hash = (hash ^ (byte as u64)).wrapping_mul(1099511628211u64)
    return hash

## `[ECS-7]` — one entry of the component registry.
@derive(Copy, Debug)
pub struct ComponentInfo:
    pub name: str
    pub hash: u64
    pub soa: bool

## `[ECS-7]` — the compile-time component registry: every type of the program that derives
## `Component`, in declaration order, module by module. The compiler writes its entries; tools for
## serialisation and reflection read them. It allocates the array it gives back.
pub fn components() -> Array[ComponentInfo]:
    return []

## The dense slot of `e` in a sparse set, if it has one. (A `mut self` method reads it through this,
## not through a shared method on `self`: such a call panics as an access conflict, a defect noted
## in tasks/85/D-notes.md.)
fn slot_in(entities: Span[Entity], sparse: Span[u32], e: Entity) -> Option[int]:
    i = e.index() as int
    if i >= sparse.len():
        return None
    at = sparse[i]
    if at == 0u32 or entities[(at - 1u32) as int] != e:
        return None
    return Some((at - 1u32) as int)

## Whether `e` is the live generation of its index.
fn alive_in(generations: Span[u32], e: Entity) -> bool:
    i = e.index() as int
    return not e.is_null() and i < generations.len() and generations[i] == e.generation()

## `[ECS-2]` — one component type's sparse set, whatever its type: the entities in dense order
## and the sparse index by entity (an entity's dense slot plus one, `0` for none). The dense
## values are its subclasses'.
pub open class AnyStorage:
    pub entities: Array[Entity]
    sparse: Array[u32]

    fn init(mut self):
        self.entities = []
        self.sparse = []

    pub fn len(self) -> int:
        return self.entities.len()

    ## The dense slot of `e`'s value, if it has one.
    pub fn slot(self, e: Entity) -> Option[int]:
        return slot_in(self.entities, self.sparse, e)

    pub fn contains(self, e: Entity) -> bool:
        return self.slot(e).is_some()

    ## Records `e` at the end of the dense order.
    fn push_entity(mut self, e: Entity):
        i = e.index() as int
        while self.sparse.len() <= i:
            self.sparse.push(0u32)
        self.entities.push(e)
        self.sparse[i] = self.entities.len() as u32

    ## Removes the entity at `slot` by moving the last one into it (`[ECS-6]`'s swap-remove hole).
    fn swap_remove_entity(mut self, slot: int):
        gone = self.entities.swap_remove(slot)
        self.sparse[gone.index() as int] = 0u32
        if slot < self.entities.len():
            moved = self.entities[slot]
            self.sparse[moved.index() as int] = (slot + 1) as u32

    ## Removes `e`'s value, if it has one.
    pub virtual fn remove_entity(mut self, e: Entity) -> bool:
        return false

## `[ECS-2]` — the default layout (`aos`): the values in an `Array[T]`, in the entities' order.
pub class AosStorage[T](AnyStorage):
    pub dense: Array[T]

    fn init(mut self):
        self.dense = []
        super.init()

    fn insert(mut self, e: Entity, owned value: T):
        match slot_in(self.entities, self.sparse, e):
            Some(at):
                self.dense[at] = value
            None:
                self.dense.push(value)
                self.push_entity(e)

    pub override fn remove_entity(mut self, e: Entity) -> bool:
        match slot_in(self.entities, self.sparse, e):
            Some(at):
                _gone = self.dense.swap_remove(at)
                self.swap_remove_entity(at)
                return true
            None:
                return false

## `[ECS-2]` — `@component(layout=soa)`: the values in an `SoA[T]`, one column per field.
pub class SoaStorage[T](AnyStorage):
    pub dense: SoA[T]

    fn init(mut self):
        self.dense = SoA[T].with_capacity(0)
        super.init()

    fn insert(mut self, e: Entity, owned value: T):
        match slot_in(self.entities, self.sparse, e):
            Some(at):
                self.dense.push(value)
                self.dense.swap(at, self.dense.len() - 1)
                _old = self.dense.pop()
            None:
                self.dense.push(value)
                self.push_entity(e)

    pub override fn remove_entity(mut self, e: Entity) -> bool:
        match slot_in(self.entities, self.sparse, e):
            Some(at):
                _gone = self.dense.swap_remove(at)
                self.swap_remove_entity(at)
                return true
            None:
                return false

## The world: entities (a generation per index, and the indices free to reuse) and one storage per
## component type, by its identity.
pub class World:
    generations: Array[u32]
    free: Array[u32]
    alive: int
    storages: Map[u64, AnyStorage]

    fn init(mut self):
        self.generations = []
        self.free = []
        self.alive = 0
        self.storages = Map[u64, AnyStorage]()

    ## A new entity: a freed index at its next generation, else a new index.
    pub fn create(mut self) -> Entity:
        self.alive += 1
        if self.free.len() > 0:
            index = self.free.pop().unwrap()
            return Entity.new(index, self.generations[index as int])
        index = self.generations.len() as u32
        self.generations.push(0u32)
        return Entity.new(index, 0u32)

    pub fn is_alive(self, e: Entity) -> bool:
        return alive_in(self.generations, e)

    ## Removes every component of `e` and frees its index for a later generation.
    pub fn destroy(mut self, e: Entity) -> bool:
        if not alive_in(self.generations, e):
            return false
        for storage in self.storages.values_mut():
            _had = storage.remove_entity(e)
        i = e.index() as int
        self.generations[i] = (self.generations[i] + 1u32) & 4095u32
        self.free.push(e.index())
        self.alive -= 1
        return true

    ## How many entities are alive.
    pub fn len(self) -> int:
        return self.alive

    ## Gives `e` the component `value` (replacing one of its type it had).
    pub fn add[T: Component](mut self, e: Entity, owned value: T):
        if not alive_in(self.generations, e):
            panic("a component is added to a live entity")
        key = T.component_hash()
        if T.component_soa():
            match self.storages.get(key):
                Some(found):
                    storage = found as! SoaStorage[T]
                    storage.insert(e, value)
                None:
                    made = SoaStorage[T]()
                    made.insert(e, value)
                    _none = self.storages.insert(key, made)
        else:
            match self.storages.get(key):
                Some(found):
                    storage = found as! AosStorage[T]
                    storage.insert(e, value)
                None:
                    made = AosStorage[T]()
                    made.insert(e, value)
                    _none = self.storages.insert(key, made)

    ## Removes `e`'s component of type `T`, if it has one.
    pub fn remove[T: Component](mut self, e: Entity) -> bool:
        match self.storages.get_mut(T.component_hash()):
            Some(found):
                return found.remove_entity(e)
            None:
                return false

    pub fn has[T: Component](self, e: Entity) -> bool:
        match self.storages.get(T.component_hash()):
            Some(found):
                return found.contains(e)
            None:
                return false

    ## The storage of `T`, if any entity has had one.
    pub fn storage[T: Component](self) -> Option[AnyStorage]:
        match self.storages.get(T.component_hash()):
            Some(found):
                return Some(found)
            None:
                return None

    ## `[ECS-6]` — the entities with a `T`, in its storage's order: insertion order, each removal
    ## filling its hole with the last entity.
    pub fn entities_with[T: Component](self) -> Array[Entity]:
        match self.storages.get(T.component_hash()):
            Some(found):
                return found.entities.clone()
            None:
                return []
