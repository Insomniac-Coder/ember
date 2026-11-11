## `std.ser` — reflection and serialisation (Part XIV).
##
## `[RFL-1]` — `reflect[T]()` describes a type while compiling: its name, its kind, its size and
## alignment, its fields (each one's name, its type's name, its offset and its attributes), its
## variants and its attributes. It is not available at run time (`E6010`): call it in a constant's
## or static's initialiser, `comptime(…)` or a `comptime:` block. A `TypeDesc` owns arrays, so it is
## kept in a `static` (`static D: TypeDesc = reflect[Point]()`), not a `const`. A field names its
## type by name: a description of each field's type would not end for a recursive type (ODR-158).
## The compiler knows `reflect` by its name here and writes the description.
##
## `[SER-1]` — `@derive(Serialize, Deserialize)` writes a struct's or an enum's `serialize` and
## `deserialize` over this module's `Writer` and `Reader`, monomorphised for each format;
## `std.ser.binary` and `std.ser.json` provide one each. The data is field-tagged and versioned: a
## struct is its version and its fields by name, an enum value its variant's name and its payload's
## fields, so a reader takes fields in any order and skips one it does not know (`[SER-2]`).

from std.fmt import Formatter, FmtError

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
    pub fn arg(name: StrView) -> Option[ref AttrValue]:
        for a in this.args:
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
    pub fn field(name: StrView) -> Option[ref FieldDesc]:
        for f in this.fields:
            if f.name == name:
                return Some(f)
        return None

    ## The variant named `name`.
    pub fn variant(name: StrView) -> Option[ref VariantDesc]:
        for v in this.variants:
            if v.name == name:
                return Some(v)
        return None

    ## The attribute named `name` written on the type.
    pub fn attribute(name: StrView) -> Option[ref AttributeDesc]:
        for a in this.attributes:
            if a.name == name:
                return Some(a)
        return None

## `[RFL-1]` — `T` described. The compiler writes the description where the call is; this staging
## body never runs.
[T]
pub fn reflect() -> TypeDesc:
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
[T: needs Reflect]
pub fn type_info() -> ref TypeInfo:
    return ref NO_TYPE

static NO_TYPE: TypeInfo = TypeDesc(String(), TypeKind.Other, 0, 0, [], [], [])

## `[SER-1]` — why a value could not be written or read.
@derive(Debug, Eq)
pub enum SerError:
    ## A field the type has, with no `@ser(default)` or `@ser(since=…)`, is not in the input (`[SER-2]`).
    MissingField(String)
    ## The input names a variant the enum does not have (`[SER-2]`).
    UnknownVariant(String)
    ## A range-typed field's value is outside its range (`[RNG-10a]`).
    OutOfRange(String)
    ## The input is not what the format writes.
    Malformed(String)

extend SerError implements Display:
    ## It allocates the message it formats.
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        match this:
            SerError.MissingField(name):
                return f.write_str(f"missing field `{name}`")
            SerError.UnknownVariant(name):
                return f.write_str(f"unknown variant `{name}`")
            SerError.OutOfRange(name):
                return f.write_str(f"field `{name}` is out of its range")
            SerError.Malformed(what):
                return f.write_str(f"malformed input: {what}")

extend SerError implements Error:
    pass

## `[SER-1]` — what a format writes a value into: a struct is its version and its fields, each by
## name; an enum value its variant, by name, and its payload's fields.
pub interface Writer:
    fn begin_struct(name: StrView, version: int) -> Result[void, SerError]
    fn field(name: StrView) -> Result[void, SerError]
    fn end_struct() -> Result[void, SerError]
    fn begin_variant(name: StrView) -> Result[void, SerError]
    fn end_variant() -> Result[void, SerError]
    fn begin_list(count: int) -> Result[void, SerError]
    fn end_list() -> Result[void, SerError]
    fn write_none() -> Result[void, SerError]
    fn write_int(value: i64) -> Result[void, SerError]
    fn write_float(value: f64) -> Result[void, SerError]
    fn write_bool(value: bool) -> Result[void, SerError]
    fn write_str(value: StrView) -> Result[void, SerError]

## `[SER-1]` — what a format reads a value from. A struct's fields come in any order, each by name,
## and one the type does not know is skipped.
pub interface Reader:
    ## The struct's version.
    fn begin_struct() -> Result[int, SerError]
    ## The next field's name, or `None` after the last.
    fn next_field() -> Result[Option[String], SerError]
    fn end_struct() -> Result[void, SerError]
    ## The variant's name.
    fn begin_variant() -> Result[String, SerError]
    fn end_variant() -> Result[void, SerError]
    fn begin_list() -> Result[void, SerError]
    ## Whether another element follows.
    fn next_element() -> Result[bool, SerError]
    ## Whether the next value is `None`, which is then read.
    fn read_none() -> Result[bool, SerError]
    fn read_int() -> Result[i64, SerError]
    fn read_float() -> Result[f64, SerError]
    fn read_bool() -> Result[bool, SerError]
    fn read_str() -> Result[String, SerError]
    ## Reads past the next value, whatever it is.
    fn skip_value() -> Result[void, SerError]

## `[SER-1]` — a value written through any `Writer`.
pub interface Serialize:
    [W: needs Writer]
    fn serialize(ref mut w: W) -> Result[void, SerError]

## `[SER-1]` — a value read through any `Reader`.
pub interface Deserialize:
    [R: needs Reader]
    static fn deserialize(ref mut r: R) -> Result[Self, SerError]

extend i64 implements Serialize:
    [W: needs Writer]
    fn serialize(ref mut w: W) -> Result[void, SerError]:
        return w.write_int(this)

extend i64 implements Deserialize:
    [R: needs Reader]
    static fn deserialize(ref mut r: R) -> Result[i64, SerError]:
        return r.read_int()

extend i32 implements Serialize:
    [W: needs Writer]
    fn serialize(ref mut w: W) -> Result[void, SerError]:
        return w.write_int(this as i64)

extend i32 implements Deserialize:
    [R: needs Reader]
    static fn deserialize(ref mut r: R) -> Result[i32, SerError]:
        value = r.read_int()?
        if value < -2147483648 or value > 2147483647:
            return Err(SerError.Malformed(f"{value} is no i32"))
        return Ok(value as i32)

extend u8 implements Serialize:
    [W: needs Writer]
    fn serialize(ref mut w: W) -> Result[void, SerError]:
        return w.write_int(this as i64)

extend u8 implements Deserialize:
    [R: needs Reader]
    static fn deserialize(ref mut r: R) -> Result[u8, SerError]:
        value = r.read_int()?
        if value < 0 or value > 255:
            return Err(SerError.Malformed(f"{value} is no u8"))
        return Ok(value as u8)

extend u32 implements Serialize:
    [W: needs Writer]
    fn serialize(ref mut w: W) -> Result[void, SerError]:
        return w.write_int(this as i64)

extend u32 implements Deserialize:
    [R: needs Reader]
    static fn deserialize(ref mut r: R) -> Result[u32, SerError]:
        value = r.read_int()?
        if value < 0 or value > 4294967295:
            return Err(SerError.Malformed(f"{value} is no u32"))
        return Ok(value as u32)

extend f64 implements Serialize:
    [W: needs Writer]
    fn serialize(ref mut w: W) -> Result[void, SerError]:
        return w.write_float(this)

extend f64 implements Deserialize:
    [R: needs Reader]
    static fn deserialize(ref mut r: R) -> Result[f64, SerError]:
        return r.read_float()

extend f32 implements Serialize:
    [W: needs Writer]
    fn serialize(ref mut w: W) -> Result[void, SerError]:
        return w.write_float(this as f64)

extend f32 implements Deserialize:
    [R: needs Reader]
    static fn deserialize(ref mut r: R) -> Result[f32, SerError]:
        return Ok(r.read_float()? as f32)

extend bool implements Serialize:
    [W: needs Writer]
    fn serialize(ref mut w: W) -> Result[void, SerError]:
        return w.write_bool(this)

extend bool implements Deserialize:
    [R: needs Reader]
    static fn deserialize(ref mut r: R) -> Result[bool, SerError]:
        return r.read_bool()

extend String implements Serialize:
    [W: needs Writer]
    fn serialize(ref mut w: W) -> Result[void, SerError]:
        return w.write_str(this.as_str())

extend String implements Deserialize:
    [R: needs Reader]
    static fn deserialize(ref mut r: R) -> Result[String, SerError]:
        return r.read_str()

extend[T: needs Serialize] Array[T] implements Serialize:
    [W: needs Writer]
    fn serialize(ref mut w: W) -> Result[void, SerError]:
        w.begin_list(len(this))?
        for x in this:
            x.serialize(ref mut w)?
        return w.end_list()

extend[T: needs Deserialize] Array[T] implements Deserialize:
    [R: needs Reader]
    static fn deserialize(ref mut r: R) -> Result[Array[T], SerError]:
        out: Array[T] = []
        r.begin_list()?
        while r.next_element()?:
            out.push(owns T.deserialize(ref mut r)?)
        return Ok(out)

extend[T: needs Serialize] Option[T] implements Serialize:
    [W: needs Writer]
    fn serialize(ref mut w: W) -> Result[void, SerError]:
        match this:
            Some(x):
                return x.serialize(ref mut w)
            None:
                return w.write_none()

extend[T: needs Deserialize] Option[T] implements Deserialize:
    [R: needs Reader]
    static fn deserialize(ref mut r: R) -> Result[Option[T], SerError]:
        if r.read_none()?:
            return Ok(None)
        return Ok(Some(T.deserialize(ref mut r)?))

## `[SER-1]` — what a derived `Deserialize` reads a field with: a value of `T`.
[T: needs Deserialize, R: needs Reader]
pub fn read(ref mut r: R) -> Result[T, SerError]:
    return T.deserialize(ref mut r)

## `[SER-2]` — a field the input lacks: its default with `@ser(default)` or `@ser(since=…)`, and
## otherwise `MissingField`.
[T]
pub fn required(owns value: Option[T], name: StrView) -> Result[T, SerError]:
    if value.is_none():
        return Err(SerError.MissingField(String.from(name)))
    return Ok(value.unwrap())

[T: needs Default]
pub fn or_default(owns value: Option[T]) -> T:
    if value.is_none():
        return T.default()
    return value.unwrap()

## `[RNG-10a]` — a range-typed field's value, checked: failure is `OutOfRange`.
[T, E]
pub fn in_range(owns value: Result[T, E], name: StrView) -> Result[T, SerError]:
    return match value:
        Ok(v) => Ok(v)
        Err(_) => Err(SerError.OutOfRange(String.from(name)))

## `[SER-1]` — what a derived `Serialize` writes a field with: its name, then its value.
[W: needs Writer, T: needs Serialize]
pub fn put(ref mut w: W, name: StrView, value: T) -> Result[void, SerError]:
    w.field(name)?
    return value.serialize(ref mut w)

## `[SER-1]` — what a derived `Deserialize` reads a field into: the slot its value fills.
[T: needs Deserialize, R: needs Reader]
pub fn take(ref mut r: R, ref mut slot: Option[T]) -> Result[void, SerError]:
    slot = Some(T.deserialize(ref mut r)?)
    return Ok(())
