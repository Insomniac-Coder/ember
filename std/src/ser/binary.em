## `std.ser.binary` — `[SER-1]`'s binary format, self-describing so that a reader can skip a field
## it does not know. Each value starts with a tag byte: 1 an integer (zigzag LEB128), 2 a float (its
## shortest text, as text is written), 3 a `bool` (one byte), 4 text (its length, then its UTF-8), 5 a
## list (its count, then the elements), 6 a struct (its version, then each field as 9, its name and
## its value, then 7), 8 an enum value (its variant's name, then its payload's fields as a struct's,
## then 7), 10 `None`.

from std.ser import Deserialize, Reader, SerError, Serialize, Writer

## Writes the binary format.
pub struct BinaryWriter:
    out: Array[u8]

    ## It allocates a new `Array`.
    pub static fn new() -> BinaryWriter:
        return BinaryWriter([])

    ## The bytes written.
    pub fn bytes() -> Span[u8]:
        return this.out.as_span()

    ## It allocates as the `Array` it adds to grows.
    fn varint(value: u64):
        v = value
        while v >= 128:
            this.out.push(((v % 128) + 128) as u8)
            v = v // 128
        this.out.push(v as u8)

    ## It allocates as the `Array` it adds to grows.
    fn text(value: StrView):
        bytes = value.as_bytes()
        this.varint(len(bytes) as u64)
        for b in bytes:
            this.out.push(b)

extend BinaryWriter implements Writer:
    ## It allocates as the `Array` it adds to grows.
    fn begin_struct(name: StrView, version: int) -> Result[void, SerError]:
        this.out.push(6)
        this.varint(version as u64)
        return Ok(())

    ## It allocates as the `Array` it adds to grows.
    fn field(name: StrView) -> Result[void, SerError]:
        this.out.push(9)
        this.text(name)
        return Ok(())

    ## It allocates as the `Array` it adds to grows.
    fn end_struct() -> Result[void, SerError]:
        this.out.push(7)
        return Ok(())

    ## It allocates as the `Array` it adds to grows.
    fn begin_variant(name: StrView) -> Result[void, SerError]:
        this.out.push(8)
        this.text(name)
        return Ok(())

    ## It allocates as the `Array` it adds to grows.
    fn end_variant() -> Result[void, SerError]:
        this.out.push(7)
        return Ok(())

    ## It allocates as the `Array` it adds to grows.
    fn begin_list(count: int) -> Result[void, SerError]:
        this.out.push(5)
        this.varint(count as u64)
        return Ok(())

    fn end_list() -> Result[void, SerError]:
        return Ok(())

    ## It allocates as the `Array` it adds to grows.
    fn write_none() -> Result[void, SerError]:
        this.out.push(10)
        return Ok(())

    ## It allocates as the `Array` it adds to grows.
    fn write_int(value: i64) -> Result[void, SerError]:
        this.out.push(1)
        zigzag = (value as u64) * 2 if value >= 0 else ((-(value + 1)) as u64) * 2 + 1
        this.varint(zigzag)
        return Ok(())

    ## It allocates as the `Array` it adds to grows.
    fn write_float(value: f64) -> Result[void, SerError]:
        this.out.push(2)
        this.text(f"{value}")
        return Ok(())

    ## It allocates as the `Array` it adds to grows.
    fn write_bool(value: bool) -> Result[void, SerError]:
        this.out.push(3)
        this.out.push(1 if value else 0)
        return Ok(())

    ## It allocates as the `Array` it adds to grows.
    fn write_str(value: StrView) -> Result[void, SerError]:
        this.out.push(4)
        this.text(value)
        return Ok(())

## Reads the binary format.
pub struct BinaryReader:
    data: Array[u8]
    at: int
    ## For each open list: how many elements are left.
    lists: Array[int]

    ## It allocates a new `Array`.
    pub static fn new(bytes: Span[u8]) -> BinaryReader:
        data: Array[u8] = []
        for b in bytes:
            data.push(b)
        return BinaryReader(data, 0, [])

    ## It allocates the error's message when the input is malformed.
    fn malformed(what: StrView) -> SerError:
        return SerError.Malformed(f"{what} at byte {this.at}")

    fn peek() -> int:
        if this.at < len(this.data):
            return this.data[this.at] as int
        return -1

    ## It allocates the error's message when the input is malformed.
    fn expect(tag: int, what: StrView) -> Result[void, SerError]:
        if this.peek() != tag:
            return Err(this.malformed(f"expected {what}"))
        this.at += 1
        return Ok(())

    ## It allocates the error's message when the input is malformed.
    fn varint() -> Result[u64, SerError]:
        value: u64 = 0
        scale: u64 = 1
        for _ in 0..10:
            if this.at >= len(this.data):
                return Err(this.malformed("a short number"))
            b = this.data[this.at] as u64
            this.at += 1
            value += (b % 128) * scale
            if b < 128:
                return Ok(value)
            scale = scale * 128
        return Err(this.malformed("a number too long"))

    ## It allocates a new `String`.
    fn text() -> Result[String, SerError]:
        n = this.varint()? as int
        if this.at + n > len(this.data):
            return Err(this.malformed("short text"))
        match this.data.as_span()[this.at..this.at + n].to_str():
            Ok(s):
                this.at += n
                return Ok(String.from(s))
            Err(_):
                return Err(this.malformed("text that is not UTF-8"))

extend BinaryReader:
    ## A struct's or an enum value's fields, read past, and its end.
    ## It allocates the error's message when the input is malformed.
    fn skip_fields() -> Result[void, SerError]:
        while this.peek() == 9:
            this.at += 1
            this.text()?
            this.skip_value()?
        return this.expect(7, "the end of a struct")

extend BinaryReader implements Reader:
    ## It allocates the error's message when the input is malformed.
    fn begin_struct() -> Result[int, SerError]:
        this.expect(6, "a struct")?
        return Ok(this.varint()? as int)

    ## It allocates the error's message when the input is malformed.
    fn next_field() -> Result[Option[String], SerError]:
        if this.peek() == 7:
            return Ok(None)
        this.expect(9, "a field")?
        return Ok(Some(this.text()?))

    ## It allocates the error's message when the input is malformed.
    fn end_struct() -> Result[void, SerError]:
        return this.expect(7, "the end of a struct")

    ## It allocates the error's message when the input is malformed.
    fn begin_variant() -> Result[String, SerError]:
        this.expect(8, "an enum value")?
        return this.text()

    ## It allocates the error's message when the input is malformed.
    fn end_variant() -> Result[void, SerError]:
        return this.expect(7, "the end of an enum value")

    ## It allocates as the `Array` it adds to grows.
    fn begin_list() -> Result[void, SerError]:
        this.expect(5, "a list")?
        n = this.varint()? as int
        this.lists.push(n)
        return Ok(())

    ## It allocates the error's message when the input is malformed.
    fn next_element() -> Result[bool, SerError]:
        n = len(this.lists)
        if n == 0:
            return Err(this.malformed("an element outside a list"))
        if this.lists[n - 1] == 0:
            _ = this.lists.pop()
            return Ok(false)
        this.lists[n - 1] -= 1
        return Ok(true)

    fn read_none() -> Result[bool, SerError]:
        if this.peek() == 10:
            this.at += 1
            return Ok(true)
        return Ok(false)

    ## It allocates the error's message when the input is malformed.
    fn read_int() -> Result[i64, SerError]:
        this.expect(1, "an integer")?
        z = this.varint()?
        return Ok((z // 2) as i64 if z % 2 == 0 else -((z // 2) as i64) - 1)

    ## It allocates the error's message when the input is malformed.
    fn read_float() -> Result[f64, SerError]:
        this.expect(2, "a float")?
        text = this.text()?
        match text.as_str().parse[f64]():
            Ok(x):
                return Ok(x)
            Err(_):
                return Err(this.malformed("a float"))

    ## It allocates the error's message when the input is malformed.
    fn read_bool() -> Result[bool, SerError]:
        this.expect(3, "a bool")?
        if this.at >= len(this.data):
            return Err(this.malformed("a short bool"))
        b = this.data[this.at]
        this.at += 1
        return Ok(b != 0)

    ## It allocates the error's message when the input is malformed.
    fn read_str() -> Result[String, SerError]:
        this.expect(4, "text")?
        return this.text()

    ## It allocates a new `String`.
    fn skip_value() -> Result[void, SerError]:
        tag = this.peek()
        if tag == 1:
            _ = this.read_int()?
        elif tag == 2:
            _ = this.read_float()?
        elif tag == 3:
            _ = this.read_bool()?
        elif tag == 4:
            _ = this.read_str()?
        elif tag == 5:
            this.at += 1
            n = this.varint()?
            for _ in 0..n:
                this.skip_value()?
        elif tag == 6:
            this.at += 1
            this.varint()?
            this.skip_fields()?
        elif tag == 8:
            this.at += 1
            this.text()?
            this.skip_fields()?
        elif tag == 10:
            this.at += 1
        else:
            return Err(this.malformed("an unknown tag"))
        return Ok(())

## `value` in the binary format.
[T: needs Serialize]
pub fn to_bytes(value: T) -> Result[Array[u8], SerError]:
    w = BinaryWriter.new()
    value.serialize(w)?
    out: Array[u8] = []
    for b in w.bytes():
        out.push(b)
    return Ok(out)

## A `T` read from the binary format.
[T: needs Deserialize]
pub fn from_bytes(bytes: Span[u8]) -> Result[T, SerError]:
    r = BinaryReader.new(bytes)
    return T.deserialize(r)
