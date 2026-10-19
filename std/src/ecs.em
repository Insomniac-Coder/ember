## `std.ecs` — the entity-component system (Part XII.3, `[ECS-1]`..`[ECS-7]`).
##
## An ordinary library: a `World` holds one sparse-set storage per component type, keyed by the
## type's identity (`type_hash`). `@derive(Component)` writes the type's identity and layout as an
## `extend … implements Component` block, as every derive writes ordinary Ember (`[DRV-1]`), says
## how a query reads the type (`extend … implements QueryParam`), and adds the type to the
## component registry (`components()`). A system is a function taking a `Query` (`[ECS-3]`);
## `World.run` borrows the storages its query names, runs it and applies its `Commands`
## (`[ECS-4]`, `[ECS-5]`). See tasks/90/M-notes.md for the design.

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
    ## `[ECS-3]` — how a query reads the type (`Read[T]`, or `ReadSoa[T]` for an `SoA`) and how
    ## `Mut[T]` writes it (`Write[T]`, `WriteSoa[T]`).
    type Read: ParamOps
    type Write: ParamOps
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

## The dense slot of `e` in a sparse set, if it has one.
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

    ## `[ECS-4]` — runs `system` over this world's `Query[Q]` with `args`: the storages `Q` names
    ## are borrowed for the run, those it writes (`Mut[T]`) mutably and the others shared, and no
    ## other; the `Commands` it queued are applied once it returns (`[ECS-5]`).
    pub fn run[Q: QueryParam, A](self, system: fn(mut Query[Q], A), owned args: A):
        fetched = Q.Ops.fetch(self)
        q = Query[Q](view=Q.Ops.view(ref fetched), cmds=Commands())
        system(q, args)
        self.apply(q.cmds)

    ## `[ECS-4]` — runs `systems`, each over its own query, once no two of them conflict: two
    ## systems conflict when one writes a component the other reads or writes. Their sets are
    ## checked against each other before any of them starts, and a conflict panics, naming both
    ## systems and the component. (A conflict between systems written in the call's own array
    ## literal is `E7020` at compile time.)
    pub fn run_parallel(self, owned systems: Array[System]):
        sets: Array[Array[Access]] = []
        for system in systems.iter_mut():
            sets.push(system.accesses())
        for i in 0..systems.len():
            for j in i + 1..systems.len():
                for a in sets[i]:
                    for b in sets[j]:
                        if a.hash == b.hash and (a.write or b.write):
                            writer = systems[i].name if a.write else systems[j].name
                            panic(f"systems `{systems[i].name}` and `{systems[j].name}` cannot run at once: `{writer}` writes `{a.name}`, which the other uses")
        for system in systems.iter_mut():
            system.run_on(self)

    ## Runs `system` over this world's `Query[Q]`: `run`'s, for a system taking only its query.
    fn run_query[Q: QueryParam](self, system: fn(mut Query[Q])):
        fetched = Q.Ops.fetch(self)
        q = Query[Q](view=Q.Ops.view(ref fetched), cmds=Commands())
        system(q)
        self.apply(q.cmds)

    ## `[ECS-5]` — the changes `commands` holds, in the order they were asked for.
    pub fn apply(self, commands: Commands):
        for command in commands.queued:
            command.apply(self)
        commands.queued.clear()

## `[ECS-3]` — `Mut[T]` in a query: `iter_mut` yields `ref mut T` for it (an `SoAMut[T]` for a
## component stored as an `SoA`); `iter` yields what `T` does. A marker: `T` is phantom
## (`[TYP-35]`).
@derive(Copy)
pub struct Mut[T]:
    pass

## `[ECS-3]` — `Not[T]` in a query: only entities without a `T`. It yields nothing (`()`).
@derive(Copy)
pub struct Not[T]:
    pass

## `[ECS-3]` — a query's parameter, as written: a component `T` (read), `Mut[T]`, `Option[P]`,
## `Not[T]`, `Entity`, or a tuple of them. Each names the library type that reads it (`Ops`):
## `@derive(Component)` writes a component's, and the library the others'.
pub interface QueryParam:
    type Ops: ParamOps

## `[ECS-3]`, `[ECS-4]` — what a query does with one parameter (or a tuple of them).
pub interface ParamOps:
    ## What `iter_mut` yields for it.
    type Item
    ## What `iter` yields for it.
    type ReadItem
    ## The storages it holds for a run.
    type Fetch
    ## Their contents, borrowed for the run.
    type View
    ## `[ECS-4]` — the components it reads and those it writes, as a component or a tuple of them
    ## (`()` for none): compile-time constants of its type.
    type Reads
    type Writes
    ## Its storages in `world` (each made, empty, if no entity has had one yet).
    fn fetch(world: World) -> Fetch
    ## Borrows them for a run.
    fn view(fetch: ref Fetch) -> View
    ## The entities it requires one of its components of, in its storage's order: the query
    ## iterates the shortest such list and probes the rest (`None` requires nothing).
    fn driver(view: View) -> Option[Span[Entity]]
    ## Its item for `e`, read-only, or `None` where `e` does not match it.
    fn get(view: ref View, e: Entity) -> Option[ReadItem]
    ## Its item for `e`, or `None` where `e` does not match it.
    fn get_mut(mut view: View, e: Entity) -> Option[Item]
    ## `[ECS-4]` — its accesses, at run time: each component it reads or writes.
    fn accesses(mut out: Array[Access])

## `[ECS-4]` — one component a system reads or writes.
@derive(Copy, Debug)
pub struct Access:
    pub hash: u64
    pub name: str
    pub write: bool

## A storage's sparse set borrowed for a run: its entities in dense order and the sparse index.
@derive(Copy)
pub struct SetView:
    entities: Span[Entity]
    sparse: Span[u32]

    ## The dense slot of `e`, if it has one.
    fn slot(self, e: Entity) -> Option[int]:
        i = e.index() as int
        if i >= self.sparse.len():
            return None
        at = self.sparse[i]
        if at == 0u32 or self.entities[(at - 1u32) as int] != e:
            return None
        return Some((at - 1u32) as int)

## `T`'s `aos` storage in `world`, made if no entity has had a `T` yet.
fn aos_storage[T: Component](world: World) -> AosStorage[T]:
    key = T.component_hash()
    if not world.storages.contains_key(key):
        _none = world.storages.insert(key, AosStorage[T]())
    return world.storages[key] as! AosStorage[T]

## `[ECS-3]` — a component `T` (`aos`) read by a query: `ref T`.
pub struct Read[T]:
    set: SetView
    values: Span[T]

extend[T: Component] Read[T] implements ParamOps:
    type Item = ref T
    type ReadItem = ref T
    type Fetch = AosStorage[T]
    type View = Read[T]
    type Reads = T
    type Writes = ()

    fn fetch(world: World) -> AosStorage[T]:
        return aos_storage[T](world)

    fn view(fetch: ref AosStorage[T]) -> Read[T]:
        return Read[T](set=SetView(entities=fetch.entities.as_span(), sparse=fetch.sparse.as_span()), values=fetch.dense.as_span())

    fn driver(view: Read[T]) -> Option[Span[Entity]]:
        return Some(view.set.entities)

    fn get(view: ref Read[T], e: Entity) -> Option[ref T]:
        match view.set.slot(e):
            Some(at):
                return Some(ref view.values[at])
            None:
                return None

    fn get_mut(mut view: Read[T], e: Entity) -> Option[ref T]:
        match view.set.slot(e):
            Some(at):
                return Some(ref view.values[at])
            None:
                return None

    fn accesses(mut out: Array[Access]):
        out.push(Access(hash=T.component_hash(), name=T.component_name(), write=false))

## `[ECS-3]` — `Mut[T]` (`aos`) in a query: `ref mut T` from `iter_mut`, `ref T` from `iter`.
pub struct Write[T]:
    set: SetView
    values: MutSpan[T]

extend[T: Component] Write[T] implements ParamOps:
    type Item = ref mut T
    type ReadItem = ref T
    type Fetch = AosStorage[T]
    type View = Write[T]
    type Reads = ()
    type Writes = T

    fn fetch(world: World) -> AosStorage[T]:
        return aos_storage[T](world)

    fn view(fetch: ref AosStorage[T]) -> Write[T]:
        return Write[T](set=SetView(entities=fetch.entities.as_span(), sparse=fetch.sparse.as_span()), values=fetch.dense.as_mut_span())

    fn driver(view: Write[T]) -> Option[Span[Entity]]:
        return Some(view.set.entities)

    fn get(view: ref Write[T], e: Entity) -> Option[ref T]:
        match view.set.slot(e):
            Some(at):
                return Some(ref view.values[at])
            None:
                return None

    fn get_mut(mut view: Write[T], e: Entity) -> Option[ref mut T]:
        match view.set.slot(e):
            Some(at):
                return Some(ref mut view.values[at])
            None:
                return None

    fn accesses(mut out: Array[Access]):
        out.push(Access(hash=T.component_hash(), name=T.component_name(), write=true))

extend[T: Component] Mut[T] implements QueryParam:
    type Ops = T.Write

## `T`'s `soa` storage in `world`, made if no entity has had a `T` yet.
fn soa_storage[T: Component](world: World) -> SoaStorage[T]:
    key = T.component_hash()
    if not world.storages.contains_key(key):
        _none = world.storages.insert(key, SoaStorage[T]())
    return world.storages[key] as! SoaStorage[T]

## `[ECS-3]`, `[SOA-6]` — a component `T` stored as an `SoA` (`@component(layout=soa)`) read by a
## query: its element proxy `SoARef[T]`, whose fields are places in the columns.
pub struct ReadSoa[T]:
    set: SetView
    values: ref SoA[T]

extend[T: Component] ReadSoa[T] implements ParamOps:
    type Item = SoARef[T]
    type ReadItem = SoARef[T]
    type Fetch = SoaStorage[T]
    type View = ReadSoa[T]
    type Reads = T
    type Writes = ()

    fn fetch(world: World) -> SoaStorage[T]:
        return soa_storage[T](world)

    fn view(fetch: ref SoaStorage[T]) -> ReadSoa[T]:
        return ReadSoa[T](set=SetView(entities=fetch.entities.as_span(), sparse=fetch.sparse.as_span()), values=ref fetch.dense)

    fn driver(view: ReadSoa[T]) -> Option[Span[Entity]]:
        return Some(view.set.entities)

    fn get(view: ref ReadSoa[T], e: Entity) -> Option[SoARef[T]]:
        match view.set.slot(e):
            Some(at):
                return Some(view.values[at])
            None:
                return None

    fn get_mut(mut view: ReadSoa[T], e: Entity) -> Option[SoARef[T]]:
        match view.set.slot(e):
            Some(at):
                return Some(view.values[at])
            None:
                return None

    fn accesses(mut out: Array[Access]):
        out.push(Access(hash=T.component_hash(), name=T.component_name(), write=false))

## `[ECS-3]`, `[SOA-6]` — `Mut[T]` for a component stored as an `SoA`: `SoAMut[T]` from `iter_mut`,
## `SoARef[T]` from `iter`.
pub struct WriteSoa[T]:
    set: SetView
    values: ref mut SoA[T]

extend[T: Component] WriteSoa[T] implements ParamOps:
    type Item = SoAMut[T]
    type ReadItem = SoARef[T]
    type Fetch = SoaStorage[T]
    type View = WriteSoa[T]
    type Reads = ()
    type Writes = T

    fn fetch(world: World) -> SoaStorage[T]:
        return soa_storage[T](world)

    fn view(fetch: ref SoaStorage[T]) -> WriteSoa[T]:
        return WriteSoa[T](set=SetView(entities=fetch.entities.as_span(), sparse=fetch.sparse.as_span()), values=ref mut fetch.dense)

    fn driver(view: WriteSoa[T]) -> Option[Span[Entity]]:
        return Some(view.set.entities)

    fn get(view: ref WriteSoa[T], e: Entity) -> Option[SoARef[T]]:
        match view.set.slot(e):
            Some(at):
                return Some(view.values[at])
            None:
                return None

    fn get_mut(mut view: WriteSoa[T], e: Entity) -> Option[SoAMut[T]]:
        match view.set.slot(e):
            Some(at):
                return Some(view.values[at])
            None:
                return None

    fn accesses(mut out: Array[Access]):
        out.push(Access(hash=T.component_hash(), name=T.component_name(), write=true))

## `[ECS-3]` — `Option[P]` in a query: every entity matches it, with `Some` of `P`'s item where
## the entity has one.
pub struct Maybe[O: ParamOps]:
    inner: O.View

extend[O: ParamOps] Maybe[O] implements ParamOps:
    type Item = Option[O.Item]
    type ReadItem = Option[O.ReadItem]
    type Fetch = O.Fetch
    type View = Maybe[O]
    type Reads = O.Reads
    type Writes = O.Writes

    fn fetch(world: World) -> O.Fetch:
        return O.fetch(world)

    fn view(fetch: ref O.Fetch) -> Maybe[O]:
        return Maybe[O](inner=O.view(fetch))

    fn driver(view: Maybe[O]) -> Option[Span[Entity]]:
        return None

    fn get(view: ref Maybe[O], e: Entity) -> Option[Option[O.ReadItem]]:
        return Some(O.get(ref view.inner, e))

    fn get_mut(mut view: Maybe[O], e: Entity) -> Option[Option[O.Item]]:
        return Some(O.get_mut(view.inner, e))

    fn accesses(mut out: Array[Access]):
        O.accesses(out)

extend[P: QueryParam] Option[P] implements QueryParam:
    type Ops = Maybe[P.Ops]

## `[ECS-3]` — `Not[T]` in a query: entities with a `T` do not match it.
pub struct Without[T]:
    set: SetView

extend[T: Component] Without[T] implements ParamOps:
    type Item = ()
    type ReadItem = ()
    type Fetch = AnyStorage
    type View = Without[T]
    type Reads = ()
    type Writes = ()

    fn fetch(world: World) -> AnyStorage:
        # `T`'s storage, made in its own layout if no entity has had a `T` yet.
        _made = T.Read.fetch(world)
        return world.storages[T.component_hash()]

    fn view(fetch: ref AnyStorage) -> Without[T]:
        return Without[T](set=SetView(entities=fetch.entities.as_span(), sparse=fetch.sparse.as_span()))

    fn driver(view: Without[T]) -> Option[Span[Entity]]:
        return None

    fn get(view: ref Without[T], e: Entity) -> Option[()]:
        if view.set.slot(e).is_some():
            return None
        return Some(())

    fn get_mut(mut view: Without[T], e: Entity) -> Option[()]:
        if view.set.slot(e).is_some():
            return None
        return Some(())

    fn accesses(mut out: Array[Access]):
        pass

extend[T: Component] Not[T] implements QueryParam:
    type Ops = Without[T]

## `[ECS-3]` — `Entity` in a query: the entity itself, matching every entity.
@derive(Copy)
pub struct Entities:
    pass

extend Entities implements ParamOps:
    type Item = Entity
    type ReadItem = Entity
    type Fetch = ()
    type View = Entities
    type Reads = ()
    type Writes = ()

    fn fetch(world: World) -> ():
        return ()

    fn view(fetch: ref ()) -> Entities:
        return Entities()

    fn driver(view: Entities) -> Option[Span[Entity]]:
        return None

    fn get(view: ref Entities, e: Entity) -> Option[Entity]:
        return Some(e)

    fn get_mut(mut view: Entities, e: Entity) -> Option[Entity]:
        return Some(e)

    fn accesses(mut out: Array[Access]):
        pass

extend Entity implements QueryParam:
    type Ops = Entities

## The shorter of two driving lists (`None` requires nothing).
fn shorter(a: Option[Span[Entity]], b: Option[Span[Entity]]) -> Option[Span[Entity]]:
    match a:
        Some(x):
            match b:
                Some(y):
                    if y.len() < x.len():
                        return b
                    return a
                None:
                    return a
        None:
            return b

## `[ECS-3]` — a tuple of parameters: an entity matches it where it matches each; it is driven by
## the shortest list any of them requires. Tuples of two to eight parameters.
extend[A: ParamOps, B: ParamOps] (A, B) implements ParamOps:
    type Item = (A.Item, B.Item)
    type ReadItem = (A.ReadItem, B.ReadItem)
    type Fetch = (A.Fetch, B.Fetch)
    type View = (A.View, B.View)
    type Reads = (A.Reads, B.Reads)
    type Writes = (A.Writes, B.Writes)

    fn fetch(world: World) -> (A.Fetch, B.Fetch):
        return (A.fetch(world), B.fetch(world))

    fn view(fetch: ref (A.Fetch, B.Fetch)) -> (A.View, B.View):
        return (A.view(ref fetch.0), B.view(ref fetch.1))

    fn driver(view: (A.View, B.View)) -> Option[Span[Entity]]:
        return shorter(A.driver(view.0), B.driver(view.1))

    fn get(view: ref (A.View, B.View), e: Entity) -> Option[(A.ReadItem, B.ReadItem)]:
        got0 = A.get(ref view.0, e)?
        got1 = B.get(ref view.1, e)?
        return Some((got0, got1))

    fn get_mut(mut view: (A.View, B.View), e: Entity) -> Option[(A.Item, B.Item)]:
        got0 = A.get_mut(view.0, e)?
        got1 = B.get_mut(view.1, e)?
        return Some((got0, got1))

    fn accesses(mut out: Array[Access]):
        A.accesses(out)
        B.accesses(out)

extend[A: QueryParam, B: QueryParam] (A, B) implements QueryParam:
    type Ops = (A.Ops, B.Ops)

extend[A: ParamOps, B: ParamOps, C: ParamOps] (A, B, C) implements ParamOps:
    type Item = (A.Item, B.Item, C.Item)
    type ReadItem = (A.ReadItem, B.ReadItem, C.ReadItem)
    type Fetch = (A.Fetch, B.Fetch, C.Fetch)
    type View = (A.View, B.View, C.View)
    type Reads = (A.Reads, B.Reads, C.Reads)
    type Writes = (A.Writes, B.Writes, C.Writes)

    fn fetch(world: World) -> (A.Fetch, B.Fetch, C.Fetch):
        return (A.fetch(world), B.fetch(world), C.fetch(world))

    fn view(fetch: ref (A.Fetch, B.Fetch, C.Fetch)) -> (A.View, B.View, C.View):
        return (A.view(ref fetch.0), B.view(ref fetch.1), C.view(ref fetch.2))

    fn driver(view: (A.View, B.View, C.View)) -> Option[Span[Entity]]:
        return shorter(shorter(A.driver(view.0), B.driver(view.1)), C.driver(view.2))

    fn get(view: ref (A.View, B.View, C.View), e: Entity) -> Option[(A.ReadItem, B.ReadItem, C.ReadItem)]:
        got0 = A.get(ref view.0, e)?
        got1 = B.get(ref view.1, e)?
        got2 = C.get(ref view.2, e)?
        return Some((got0, got1, got2))

    fn get_mut(mut view: (A.View, B.View, C.View), e: Entity) -> Option[(A.Item, B.Item, C.Item)]:
        got0 = A.get_mut(view.0, e)?
        got1 = B.get_mut(view.1, e)?
        got2 = C.get_mut(view.2, e)?
        return Some((got0, got1, got2))

    fn accesses(mut out: Array[Access]):
        A.accesses(out)
        B.accesses(out)
        C.accesses(out)

extend[A: QueryParam, B: QueryParam, C: QueryParam] (A, B, C) implements QueryParam:
    type Ops = (A.Ops, B.Ops, C.Ops)

extend[A: ParamOps, B: ParamOps, C: ParamOps, D: ParamOps] (A, B, C, D) implements ParamOps:
    type Item = (A.Item, B.Item, C.Item, D.Item)
    type ReadItem = (A.ReadItem, B.ReadItem, C.ReadItem, D.ReadItem)
    type Fetch = (A.Fetch, B.Fetch, C.Fetch, D.Fetch)
    type View = (A.View, B.View, C.View, D.View)
    type Reads = (A.Reads, B.Reads, C.Reads, D.Reads)
    type Writes = (A.Writes, B.Writes, C.Writes, D.Writes)

    fn fetch(world: World) -> (A.Fetch, B.Fetch, C.Fetch, D.Fetch):
        return (A.fetch(world), B.fetch(world), C.fetch(world), D.fetch(world))

    fn view(fetch: ref (A.Fetch, B.Fetch, C.Fetch, D.Fetch)) -> (A.View, B.View, C.View, D.View):
        return (A.view(ref fetch.0), B.view(ref fetch.1), C.view(ref fetch.2), D.view(ref fetch.3))

    fn driver(view: (A.View, B.View, C.View, D.View)) -> Option[Span[Entity]]:
        return shorter(shorter(shorter(A.driver(view.0), B.driver(view.1)), C.driver(view.2)), D.driver(view.3))

    fn get(view: ref (A.View, B.View, C.View, D.View), e: Entity) -> Option[(A.ReadItem, B.ReadItem, C.ReadItem, D.ReadItem)]:
        got0 = A.get(ref view.0, e)?
        got1 = B.get(ref view.1, e)?
        got2 = C.get(ref view.2, e)?
        got3 = D.get(ref view.3, e)?
        return Some((got0, got1, got2, got3))

    fn get_mut(mut view: (A.View, B.View, C.View, D.View), e: Entity) -> Option[(A.Item, B.Item, C.Item, D.Item)]:
        got0 = A.get_mut(view.0, e)?
        got1 = B.get_mut(view.1, e)?
        got2 = C.get_mut(view.2, e)?
        got3 = D.get_mut(view.3, e)?
        return Some((got0, got1, got2, got3))

    fn accesses(mut out: Array[Access]):
        A.accesses(out)
        B.accesses(out)
        C.accesses(out)
        D.accesses(out)

extend[A: QueryParam, B: QueryParam, C: QueryParam, D: QueryParam] (A, B, C, D) implements QueryParam:
    type Ops = (A.Ops, B.Ops, C.Ops, D.Ops)

extend[A: ParamOps, B: ParamOps, C: ParamOps, D: ParamOps, E: ParamOps] (A, B, C, D, E) implements ParamOps:
    type Item = (A.Item, B.Item, C.Item, D.Item, E.Item)
    type ReadItem = (A.ReadItem, B.ReadItem, C.ReadItem, D.ReadItem, E.ReadItem)
    type Fetch = (A.Fetch, B.Fetch, C.Fetch, D.Fetch, E.Fetch)
    type View = (A.View, B.View, C.View, D.View, E.View)
    type Reads = (A.Reads, B.Reads, C.Reads, D.Reads, E.Reads)
    type Writes = (A.Writes, B.Writes, C.Writes, D.Writes, E.Writes)

    fn fetch(world: World) -> (A.Fetch, B.Fetch, C.Fetch, D.Fetch, E.Fetch):
        return (A.fetch(world), B.fetch(world), C.fetch(world), D.fetch(world), E.fetch(world))

    fn view(fetch: ref (A.Fetch, B.Fetch, C.Fetch, D.Fetch, E.Fetch)) -> (A.View, B.View, C.View, D.View, E.View):
        return (A.view(ref fetch.0), B.view(ref fetch.1), C.view(ref fetch.2), D.view(ref fetch.3), E.view(ref fetch.4))

    fn driver(view: (A.View, B.View, C.View, D.View, E.View)) -> Option[Span[Entity]]:
        return shorter(shorter(shorter(shorter(A.driver(view.0), B.driver(view.1)), C.driver(view.2)), D.driver(view.3)), E.driver(view.4))

    fn get(view: ref (A.View, B.View, C.View, D.View, E.View), e: Entity) -> Option[(A.ReadItem, B.ReadItem, C.ReadItem, D.ReadItem, E.ReadItem)]:
        got0 = A.get(ref view.0, e)?
        got1 = B.get(ref view.1, e)?
        got2 = C.get(ref view.2, e)?
        got3 = D.get(ref view.3, e)?
        got4 = E.get(ref view.4, e)?
        return Some((got0, got1, got2, got3, got4))

    fn get_mut(mut view: (A.View, B.View, C.View, D.View, E.View), e: Entity) -> Option[(A.Item, B.Item, C.Item, D.Item, E.Item)]:
        got0 = A.get_mut(view.0, e)?
        got1 = B.get_mut(view.1, e)?
        got2 = C.get_mut(view.2, e)?
        got3 = D.get_mut(view.3, e)?
        got4 = E.get_mut(view.4, e)?
        return Some((got0, got1, got2, got3, got4))

    fn accesses(mut out: Array[Access]):
        A.accesses(out)
        B.accesses(out)
        C.accesses(out)
        D.accesses(out)
        E.accesses(out)

extend[A: QueryParam, B: QueryParam, C: QueryParam, D: QueryParam, E: QueryParam] (A, B, C, D, E) implements QueryParam:
    type Ops = (A.Ops, B.Ops, C.Ops, D.Ops, E.Ops)

extend[A: ParamOps, B: ParamOps, C: ParamOps, D: ParamOps, E: ParamOps, F: ParamOps] (A, B, C, D, E, F) implements ParamOps:
    type Item = (A.Item, B.Item, C.Item, D.Item, E.Item, F.Item)
    type ReadItem = (A.ReadItem, B.ReadItem, C.ReadItem, D.ReadItem, E.ReadItem, F.ReadItem)
    type Fetch = (A.Fetch, B.Fetch, C.Fetch, D.Fetch, E.Fetch, F.Fetch)
    type View = (A.View, B.View, C.View, D.View, E.View, F.View)
    type Reads = (A.Reads, B.Reads, C.Reads, D.Reads, E.Reads, F.Reads)
    type Writes = (A.Writes, B.Writes, C.Writes, D.Writes, E.Writes, F.Writes)

    fn fetch(world: World) -> (A.Fetch, B.Fetch, C.Fetch, D.Fetch, E.Fetch, F.Fetch):
        return (A.fetch(world), B.fetch(world), C.fetch(world), D.fetch(world), E.fetch(world), F.fetch(world))

    fn view(fetch: ref (A.Fetch, B.Fetch, C.Fetch, D.Fetch, E.Fetch, F.Fetch)) -> (A.View, B.View, C.View, D.View, E.View, F.View):
        return (A.view(ref fetch.0), B.view(ref fetch.1), C.view(ref fetch.2), D.view(ref fetch.3), E.view(ref fetch.4), F.view(ref fetch.5))

    fn driver(view: (A.View, B.View, C.View, D.View, E.View, F.View)) -> Option[Span[Entity]]:
        return shorter(shorter(shorter(shorter(shorter(A.driver(view.0), B.driver(view.1)), C.driver(view.2)), D.driver(view.3)), E.driver(view.4)), F.driver(view.5))

    fn get(view: ref (A.View, B.View, C.View, D.View, E.View, F.View), e: Entity) -> Option[(A.ReadItem, B.ReadItem, C.ReadItem, D.ReadItem, E.ReadItem, F.ReadItem)]:
        got0 = A.get(ref view.0, e)?
        got1 = B.get(ref view.1, e)?
        got2 = C.get(ref view.2, e)?
        got3 = D.get(ref view.3, e)?
        got4 = E.get(ref view.4, e)?
        got5 = F.get(ref view.5, e)?
        return Some((got0, got1, got2, got3, got4, got5))

    fn get_mut(mut view: (A.View, B.View, C.View, D.View, E.View, F.View), e: Entity) -> Option[(A.Item, B.Item, C.Item, D.Item, E.Item, F.Item)]:
        got0 = A.get_mut(view.0, e)?
        got1 = B.get_mut(view.1, e)?
        got2 = C.get_mut(view.2, e)?
        got3 = D.get_mut(view.3, e)?
        got4 = E.get_mut(view.4, e)?
        got5 = F.get_mut(view.5, e)?
        return Some((got0, got1, got2, got3, got4, got5))

    fn accesses(mut out: Array[Access]):
        A.accesses(out)
        B.accesses(out)
        C.accesses(out)
        D.accesses(out)
        E.accesses(out)
        F.accesses(out)

extend[A: QueryParam, B: QueryParam, C: QueryParam, D: QueryParam, E: QueryParam, F: QueryParam] (A, B, C, D, E, F) implements QueryParam:
    type Ops = (A.Ops, B.Ops, C.Ops, D.Ops, E.Ops, F.Ops)

extend[A: ParamOps, B: ParamOps, C: ParamOps, D: ParamOps, E: ParamOps, F: ParamOps, G: ParamOps] (A, B, C, D, E, F, G) implements ParamOps:
    type Item = (A.Item, B.Item, C.Item, D.Item, E.Item, F.Item, G.Item)
    type ReadItem = (A.ReadItem, B.ReadItem, C.ReadItem, D.ReadItem, E.ReadItem, F.ReadItem, G.ReadItem)
    type Fetch = (A.Fetch, B.Fetch, C.Fetch, D.Fetch, E.Fetch, F.Fetch, G.Fetch)
    type View = (A.View, B.View, C.View, D.View, E.View, F.View, G.View)
    type Reads = (A.Reads, B.Reads, C.Reads, D.Reads, E.Reads, F.Reads, G.Reads)
    type Writes = (A.Writes, B.Writes, C.Writes, D.Writes, E.Writes, F.Writes, G.Writes)

    fn fetch(world: World) -> (A.Fetch, B.Fetch, C.Fetch, D.Fetch, E.Fetch, F.Fetch, G.Fetch):
        return (A.fetch(world), B.fetch(world), C.fetch(world), D.fetch(world), E.fetch(world), F.fetch(world), G.fetch(world))

    fn view(fetch: ref (A.Fetch, B.Fetch, C.Fetch, D.Fetch, E.Fetch, F.Fetch, G.Fetch)) -> (A.View, B.View, C.View, D.View, E.View, F.View, G.View):
        return (A.view(ref fetch.0), B.view(ref fetch.1), C.view(ref fetch.2), D.view(ref fetch.3), E.view(ref fetch.4), F.view(ref fetch.5), G.view(ref fetch.6))

    fn driver(view: (A.View, B.View, C.View, D.View, E.View, F.View, G.View)) -> Option[Span[Entity]]:
        return shorter(shorter(shorter(shorter(shorter(shorter(A.driver(view.0), B.driver(view.1)), C.driver(view.2)), D.driver(view.3)), E.driver(view.4)), F.driver(view.5)), G.driver(view.6))

    fn get(view: ref (A.View, B.View, C.View, D.View, E.View, F.View, G.View), e: Entity) -> Option[(A.ReadItem, B.ReadItem, C.ReadItem, D.ReadItem, E.ReadItem, F.ReadItem, G.ReadItem)]:
        got0 = A.get(ref view.0, e)?
        got1 = B.get(ref view.1, e)?
        got2 = C.get(ref view.2, e)?
        got3 = D.get(ref view.3, e)?
        got4 = E.get(ref view.4, e)?
        got5 = F.get(ref view.5, e)?
        got6 = G.get(ref view.6, e)?
        return Some((got0, got1, got2, got3, got4, got5, got6))

    fn get_mut(mut view: (A.View, B.View, C.View, D.View, E.View, F.View, G.View), e: Entity) -> Option[(A.Item, B.Item, C.Item, D.Item, E.Item, F.Item, G.Item)]:
        got0 = A.get_mut(view.0, e)?
        got1 = B.get_mut(view.1, e)?
        got2 = C.get_mut(view.2, e)?
        got3 = D.get_mut(view.3, e)?
        got4 = E.get_mut(view.4, e)?
        got5 = F.get_mut(view.5, e)?
        got6 = G.get_mut(view.6, e)?
        return Some((got0, got1, got2, got3, got4, got5, got6))

    fn accesses(mut out: Array[Access]):
        A.accesses(out)
        B.accesses(out)
        C.accesses(out)
        D.accesses(out)
        E.accesses(out)
        F.accesses(out)
        G.accesses(out)

extend[A: QueryParam, B: QueryParam, C: QueryParam, D: QueryParam, E: QueryParam, F: QueryParam, G: QueryParam] (A, B, C, D, E, F, G) implements QueryParam:
    type Ops = (A.Ops, B.Ops, C.Ops, D.Ops, E.Ops, F.Ops, G.Ops)

extend[A: ParamOps, B: ParamOps, C: ParamOps, D: ParamOps, E: ParamOps, F: ParamOps, G: ParamOps, H: ParamOps] (A, B, C, D, E, F, G, H) implements ParamOps:
    type Item = (A.Item, B.Item, C.Item, D.Item, E.Item, F.Item, G.Item, H.Item)
    type ReadItem = (A.ReadItem, B.ReadItem, C.ReadItem, D.ReadItem, E.ReadItem, F.ReadItem, G.ReadItem, H.ReadItem)
    type Fetch = (A.Fetch, B.Fetch, C.Fetch, D.Fetch, E.Fetch, F.Fetch, G.Fetch, H.Fetch)
    type View = (A.View, B.View, C.View, D.View, E.View, F.View, G.View, H.View)
    type Reads = (A.Reads, B.Reads, C.Reads, D.Reads, E.Reads, F.Reads, G.Reads, H.Reads)
    type Writes = (A.Writes, B.Writes, C.Writes, D.Writes, E.Writes, F.Writes, G.Writes, H.Writes)

    fn fetch(world: World) -> (A.Fetch, B.Fetch, C.Fetch, D.Fetch, E.Fetch, F.Fetch, G.Fetch, H.Fetch):
        return (A.fetch(world), B.fetch(world), C.fetch(world), D.fetch(world), E.fetch(world), F.fetch(world), G.fetch(world), H.fetch(world))

    fn view(fetch: ref (A.Fetch, B.Fetch, C.Fetch, D.Fetch, E.Fetch, F.Fetch, G.Fetch, H.Fetch)) -> (A.View, B.View, C.View, D.View, E.View, F.View, G.View, H.View):
        return (A.view(ref fetch.0), B.view(ref fetch.1), C.view(ref fetch.2), D.view(ref fetch.3), E.view(ref fetch.4), F.view(ref fetch.5), G.view(ref fetch.6), H.view(ref fetch.7))

    fn driver(view: (A.View, B.View, C.View, D.View, E.View, F.View, G.View, H.View)) -> Option[Span[Entity]]:
        return shorter(shorter(shorter(shorter(shorter(shorter(shorter(A.driver(view.0), B.driver(view.1)), C.driver(view.2)), D.driver(view.3)), E.driver(view.4)), F.driver(view.5)), G.driver(view.6)), H.driver(view.7))

    fn get(view: ref (A.View, B.View, C.View, D.View, E.View, F.View, G.View, H.View), e: Entity) -> Option[(A.ReadItem, B.ReadItem, C.ReadItem, D.ReadItem, E.ReadItem, F.ReadItem, G.ReadItem, H.ReadItem)]:
        got0 = A.get(ref view.0, e)?
        got1 = B.get(ref view.1, e)?
        got2 = C.get(ref view.2, e)?
        got3 = D.get(ref view.3, e)?
        got4 = E.get(ref view.4, e)?
        got5 = F.get(ref view.5, e)?
        got6 = G.get(ref view.6, e)?
        got7 = H.get(ref view.7, e)?
        return Some((got0, got1, got2, got3, got4, got5, got6, got7))

    fn get_mut(mut view: (A.View, B.View, C.View, D.View, E.View, F.View, G.View, H.View), e: Entity) -> Option[(A.Item, B.Item, C.Item, D.Item, E.Item, F.Item, G.Item, H.Item)]:
        got0 = A.get_mut(view.0, e)?
        got1 = B.get_mut(view.1, e)?
        got2 = C.get_mut(view.2, e)?
        got3 = D.get_mut(view.3, e)?
        got4 = E.get_mut(view.4, e)?
        got5 = F.get_mut(view.5, e)?
        got6 = G.get_mut(view.6, e)?
        got7 = H.get_mut(view.7, e)?
        return Some((got0, got1, got2, got3, got4, got5, got6, got7))

    fn accesses(mut out: Array[Access]):
        A.accesses(out)
        B.accesses(out)
        C.accesses(out)
        D.accesses(out)
        E.accesses(out)
        F.accesses(out)
        G.accesses(out)
        H.accesses(out)

extend[A: QueryParam, B: QueryParam, C: QueryParam, D: QueryParam, E: QueryParam, F: QueryParam, G: QueryParam, H: QueryParam] (A, B, C, D, E, F, G, H) implements QueryParam:
    type Ops = (A.Ops, B.Ops, C.Ops, D.Ops, E.Ops, F.Ops, G.Ops, H.Ops)

## `[ECS-3]` — a view over a world's storages for one system's run: `iter` yields each matching
## entity's items read-only, `iter_mut` yields `ref mut` for `Mut` parameters (and needs `mut q`).
## Entities come in the order of the shortest storage the query requires (`[ECS-6]`).
pub struct Query[Q: QueryParam]:
    view: Q.Ops.View
    cmds: Commands

    pub fn iter(self) -> QueryIter[Q]:
        driver = driving(Q.Ops.driver(self.view))
        return QueryIter[Q](view=ref self.view, driver=driver, at=0)

    pub fn iter_mut(mut self) -> QueryIterMut[Q]:
        driver = driving(Q.Ops.driver(self.view))
        return QueryIterMut[Q](view=ref mut self.view, driver=driver, at=0)

    ## `[ECS-5]` — the buffer for adding, removing and destroying while iterating: applied after
    ## the system returns.
    pub fn commands(self) -> Commands:
        return self.cmds

## A query's driving list: one of its parameters must require a component.
fn driving(list: Option[Span[Entity]]) -> Span[Entity]:
    match list:
        Some(found):
            return found
        None:
            panic("a query requires at least one component (a parameter that is not `Option`, `Not` or `Entity`)")

## `q.iter()`.
pub struct QueryIter[Q: QueryParam]:
    view: ref Q.Ops.View
    driver: Span[Entity]
    at: int

    pub fn next(mut self) -> Option[Q.Ops.ReadItem]:
        while self.at < self.driver.len():
            e = self.driver[self.at]
            self.at += 1
            match Q.Ops.get(self.view, e):
                Some(item):
                    return Some(item)
                None:
                    pass
        return None

extend[Q: QueryParam] QueryIter[Q] implements Iterator:
    type Item = Q.Ops.ReadItem

## `q.iter_mut()`.
pub struct QueryIterMut[Q: QueryParam]:
    view: ref mut Q.Ops.View
    driver: Span[Entity]
    at: int

    pub fn next(mut self) -> Option[Q.Ops.Item]:
        while self.at < self.driver.len():
            e = self.driver[self.at]
            self.at += 1
            match Q.Ops.get_mut(self.view, e):
                Some(item):
                    return Some(item)
                None:
                    pass
        return None

extend[Q: QueryParam] QueryIterMut[Q] implements Iterator:
    type Item = Q.Ops.Item

## `[ECS-5]` — one change a system asked for, made once it has returned.
pub open class Command:
    fn init(mut self):
        pass

    virtual fn apply(self, world: World):
        pass

class DestroyCommand(Command):
    e: Entity

    fn init(mut self, e: Entity):
        self.e = e
        super.init()

    override fn apply(self, world: World):
        _was = world.destroy(self.e)

class RemoveCommand[T: Component](Command):
    e: Entity

    fn init(mut self, e: Entity):
        self.e = e
        super.init()

    override fn apply(self, world: World):
        _had = world.remove[T](self.e)

class AddCommand[T: Component](Command):
    e: Entity
    value: Option[T]

    fn init(mut self, e: Entity, owned value: T):
        self.e = e
        self.value = Some(value)
        super.init()

    override fn apply(self, world: World):
        match self.value.take():
            Some(value):
                world.add(self.e, value)
            None:
                pass

## `[ECS-5]` — the changes a system asks for while it iterates, in order: applied to the world
## after the system returns (`World.run`), so no storage changes under a query.
pub class Commands:
    queued: Array[Command]

    fn init(mut self):
        self.queued = []

    ## Destroys `e` (and every component it has).
    pub fn destroy(self, e: Entity):
        self.queued.push(DestroyCommand(e))

    ## Gives `e` the component `value`.
    pub fn add[T: Component](self, e: Entity, owned value: T):
        self.queued.push(AddCommand[T](e, value))

    ## Removes `e`'s component of type `T`.
    pub fn remove[T: Component](self, e: Entity):
        self.queued.push(RemoveCommand[T](e))

    ## How many changes are waiting.
    pub fn len(self) -> int:
        return self.queued.len()

## `[ECS-4]` — a system as a value: its name, and its query's accesses and its run, each an owned
## callable value (`[CLO-3]`), so systems over different queries are one type. A function over a
## `Query` written in `run_parallel`'s array literal is made one by the compiler (as
## `system(f, "f")`), which checks the literal's systems against each other (`E7020`).
pub struct System:
    pub name: str
    accesses_of: fn(mut Array[Access])
    run_on: fn(World)

    ## `[ECS-4]` — the components it reads and writes.
    pub fn accesses(mut self) -> Array[Access]:
        out: Array[Access] = []
        self.accesses_of(out)
        return out

## `[ECS-4]` — the system `body`, named `name` (what a conflict reports).
pub fn system[Q: QueryParam](body: fn(mut Query[Q]), name: str) -> System:
    return System(name=name, accesses_of=owned fn(mut out: Array[Access]) => Q.Ops.accesses(out), run_on=owned fn(world: World) => world.run_query[Q](body))
