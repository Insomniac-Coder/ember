## `std.ser` — reflection and serialisation (Part XIV).
##
## `[RFL-1]` — `reflect[T]()` describes a type while compiling: its name, its kind, its size and
## alignment, its fields (each one's name, its type's name, its offset and its attributes), its
## variants and its attributes. It is not available at run time (`E6010`): call it in a constant's
## or static's initialiser, `comptime(…)` or a `comptime:` block. A `TypeDesc` owns arrays, so it is
## kept in a `static` (`static D: TypeDesc = reflect[Point]()`), not a `const`. A field names its
## type by name: a description of each field's type would not end for a recursive type (ODR-158).
## The compiler knows `reflect` by its name here and writes the description.

## What kind of type a `TypeDesc` describes.
@derive(Eq, Debug)
pub enum TypeKind:
    Struct
    Enum
    Class
    Integer
    Float
    Bool
    Char
    Text
    Tuple
    Array
    Other

## `[RFL-3]` — one argument of an attribute, as written: an attribute's arguments are numbers,
## text, `bool`s and `char`s (ODR-158).
@derive(Eq, Debug)
pub enum AttrValue:
    Int(i64)
    Float(f64)
    Str(String)
    Bool(bool)
    Char(char)

## An attribute written on a type, a field or a variant: its name and its arguments, each by the
## name of the field it sets (a positional argument by its position, `"0"`, `"1"`, …).
@derive(Debug)
pub struct AttributeDesc:
    pub name: String
    pub args: Array[(String, AttrValue)]

    ## The argument named `name`.
    pub fn arg(self, name: str) -> Option[ref AttrValue]:
        for a in self.args:
            if a.0 == name:
                return Some(ref a.1)
        return None

## A field: its name, its type's name, its offset in bytes in the value (in a class's object,
## counted from the end of the header the runtime owns, `[OBJ-1]`) and its attributes.
@derive(Debug)
pub struct FieldDesc:
    pub name: String
    pub type_name: String
    pub offset: int
    pub attributes: Array[AttributeDesc]

## A variant of an enum: its name, its discriminant, the fields of its payload (offsets in the whole
## value) and its attributes.
@derive(Debug)
pub struct VariantDesc:
    pub name: String
    pub discriminant: i64
    pub fields: Array[FieldDesc]
    pub attributes: Array[AttributeDesc]

## `[RFL-1]` — a type described while compiling.
@derive(Debug)
pub struct TypeDesc:
    pub name: String
    pub kind: TypeKind
    pub size: int
    pub align: int
    pub fields: Array[FieldDesc]
    pub variants: Array[VariantDesc]
    pub attributes: Array[AttributeDesc]

    ## The field named `name`.
    pub fn field(self, name: str) -> Option[ref FieldDesc]:
        for f in self.fields:
            if f.name == name:
                return Some(f)
        return None

    ## The variant named `name`.
    pub fn variant(self, name: str) -> Option[ref VariantDesc]:
        for v in self.variants:
            if v.name == name:
                return Some(v)
        return None

    ## The attribute named `name` written on the type.
    pub fn attribute(self, name: str) -> Option[ref AttributeDesc]:
        for a in self.attributes:
            if a.name == name:
                return Some(a)
        return None

## `[RFL-1]` — `T` described. The compiler writes the description where the call is; this staging
## body never runs.
pub fn reflect[T]() -> TypeDesc:
    return TypeDesc(String(), TypeKind.Other, 0, 0, [], [], [])

## `[RFL-2]` — a type written `@reflect`, whose description the program keeps for run time. The
## compiler implements it for each such type, and for each class derived from one (ODR-158).
pub interface Reflect:
    pass

## `[RFL-2]` — a description kept in the program: the same descriptors a `TypeDesc` holds.
pub type TypeInfo = TypeDesc

## `[RFL-2]` — `T`'s description, which `@reflect` keeps in the program: one constant table for
## each type, which the link removes where no code reaches it. The compiler writes the reference
## where the call is; this staging body never runs. On a class handle, `h.type_info()` gives the
## description of the object's own class (`[RT-3]`).
pub fn type_info[T: Reflect]() -> ref TypeInfo:
    return ref NO_TYPE

static NO_TYPE: TypeInfo = TypeDesc(String(), TypeKind.Other, 0, 0, [], [], [])
