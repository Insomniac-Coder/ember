//! A compact binary encoding of the compiler's own records (`[BLD-1]`, `[BLD-2]`): what a module's
//! check left (its MIR, its diagnostics) kept between builds and read back.
//!
//! The format is not self-describing: a record is read only by the compiler that wrote it (its
//! identity is part of every cache key), into the same Rust types. Unsigned integers are LEB128,
//! signed ones zigzag LEB128, floats their little-endian bits, strings and sequences a length and
//! their items, an `Option` a tag byte, an enum its variant's index and its fields, a struct its
//! fields in order.
//!
//! A [`Span`] is written as its file's index in the record's file table and its offsets, so a
//! record does not depend on the order files were loaded in: [`FileTable`] numbers the files while
//! a record is written ([`encode`]) and maps the numbers back while one is read ([`decode`]). The
//! table also gathers the byte ranges the spans cover in each file (and in each part of a record,
//! [`scope`]), and a record is read with each file's offsets moved to where its text is now
//! ([`Relocation`]): a record stays true of a file edited elsewhere while the text its spans point
//! at is unchanged, wherever that text now is.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

use serde::de::{self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess, Visitor};
use serde::ser;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{FileId, Span, Symbol};

#[derive(Debug)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error(msg.to_string())
    }
}

/// `value`'s bytes.
pub fn to_bytes<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, Error> {
    let mut writer = Writer { out: Vec::new() };
    value.serialize(&mut writer)?;
    Ok(writer.out)
}

/// The value `bytes` hold; every byte must be read.
pub fn from_bytes<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T, Error> {
    let mut reader = Reader { input: bytes };
    let value = T::deserialize(&mut reader)?;
    if !reader.input.is_empty() {
        return Err(Error(format!("{} bytes left after the record", reader.input.len())));
    }
    Ok(value)
}

// ---------------------------------------------------------------------------------------------
// Files

/// The files a record's spans are in: numbered as a record is written, mapped back to this run's
/// files as one is read.
enum FileTable {
    Encode { index: HashMap<FileId, u32>, files: Vec<Extent>, scopes: Vec<Vec<Extent>> },
    Decode { files: Vec<(FileId, Relocation)>, lenient: bool },
}

/// How a record's offsets in one file are this run's: the pieces of the file its spans lie in,
/// each as its start then, its length (the same now: its text is) and its start now. Pieces do not
/// overlap. The identity when the file's offsets have not moved.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Relocation(Option<Vec<(u32, u32, u32)>>);

impl Relocation {
    pub fn identity() -> Relocation {
        Relocation(None)
    }

    pub fn pieces(mut pieces: Vec<(u32, u32, u32)>) -> Relocation {
        pieces.sort_unstable();
        Relocation(Some(pieces))
    }

    /// An offset then, now; `None` when no piece holds it.
    pub fn map(&self, offset: u32) -> Option<u32> {
        let Some(pieces) = &self.0 else { return Some(offset) };
        let after = pieces.partition_point(|&(start, _, _)| start <= offset);
        pieces[..after].last().filter(|&&(start, length, _)| offset <= start + length).map(|&(start, _, now)| offset - start + now)
    }
}

/// A file a record's spans are in, and the byte ranges they cover in it: sorted, apart, and none
/// empty (an empty span covers the byte it is at).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extent {
    pub file: FileId,
    pub ranges: Vec<(u32, u32)>,
}

impl Extent {
    fn cover(&mut self, start: u32, end: u32) {
        let end = end.max(start + 1);
        match self.ranges.last_mut() {
            // Spans come mostly nested or in order: most join the last range.
            Some(last) if start >= last.0 && start <= last.1 => last.1 = last.1.max(end),
            _ => self.ranges.push((start, end)),
        }
    }

    fn settle(&mut self) {
        self.ranges.sort_unstable();
        let mut merged: Vec<(u32, u32)> = Vec::with_capacity(self.ranges.len());
        for &(start, end) in &self.ranges {
            match merged.last_mut() {
                Some(last) if start <= last.1 => last.1 = last.1.max(end),
                _ => merged.push((start, end)),
            }
        }
        self.ranges = merged;
    }
}

thread_local! {
    static FILES: RefCell<Option<FileTable>> = const { RefCell::new(None) };
}

/// Run `write` with spans written by file number; the files, in number order, each with the
/// ranges the spans cover.
pub fn encode<R>(write: impl FnOnce() -> R) -> (R, Vec<Extent>) {
    let outer = FILES.with(|files| files.replace(Some(FileTable::Encode { index: HashMap::new(), files: Vec::new(), scopes: Vec::new() })));
    let result = write();
    let table = FILES.with(|files| files.replace(outer));
    match table {
        Some(FileTable::Encode { mut files, .. }) => {
            files.iter_mut().for_each(Extent::settle);
            (result, files)
        }
        _ => unreachable!("the file table set above"),
    }
}

/// Inside [`encode`]: run `write`, which writes part of the record; the ranges the spans it writes
/// cover, by file number (files are numbered as the whole record's are).
pub fn scope<R>(write: impl FnOnce() -> R) -> (R, Vec<(u32, Vec<(u32, u32)>)>) {
    FILES.with(|table| match table.borrow_mut().as_mut() {
        Some(FileTable::Encode { scopes, .. }) => scopes.push(Vec::new()),
        _ => panic!("a scope is written only inside `codec::encode`"),
    });
    let result = write();
    let scope = FILES.with(|table| match table.borrow_mut().as_mut() {
        Some(FileTable::Encode { scopes, .. }) => scopes.pop().expect("pushed above"),
        _ => unreachable!("the file table is the one the scope began in"),
    });
    let ranges = scope
        .into_iter()
        .enumerate()
        .filter(|(_, extent)| !extent.ranges.is_empty())
        .map(|(number, mut extent)| {
            extent.settle();
            (number as u32, extent.ranges)
        })
        .collect();
    (result, ranges)
}

/// Run `read` with file number `n` read as `files[n]`.
pub fn decode<R>(files: Vec<FileId>, read: impl FnOnce() -> R) -> R {
    decode_relocated(files.into_iter().map(|file| (file, Relocation::identity())).collect(), false, read)
}

/// Run `read` with file number `n` read as `files[n].0`, its offsets moved by `files[n].1`. A span
/// no piece holds is an error, or, `lenient`, the dummy span (for a part of a record that is read
/// whole while only some of it is used, the rest checked before it is).
pub fn decode_relocated<R>(files: Vec<(FileId, Relocation)>, lenient: bool, read: impl FnOnce() -> R) -> R {
    let outer = FILES.with(|table| table.replace(Some(FileTable::Decode { files, lenient })));
    let result = read();
    FILES.with(|table| table.replace(outer));
    result
}

const NO_FILE: u32 = u32::MAX;

impl Serialize for Span {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let file = if self.is_dummy() {
            NO_FILE
        } else {
            FILES.with(|table| match table.borrow_mut().as_mut() {
                Some(FileTable::Encode { index, files, scopes }) => {
                    let number = *index.entry(self.file).or_insert_with(|| {
                        files.push(Extent { file: self.file, ranges: Vec::new() });
                        files.len() as u32 - 1
                    });
                    files[number as usize].cover(self.start, self.end);
                    for scope in scopes.iter_mut() {
                        while scope.len() <= number as usize {
                            let file = files[scope.len()].file;
                            scope.push(Extent { file, ranges: Vec::new() });
                        }
                        scope[number as usize].cover(self.start, self.end);
                    }
                    number
                }
                _ => panic!("a span is written only inside `codec::encode`"),
            })
        };
        (file, self.start, self.end).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Span {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Span, D::Error> {
        let (file, start, end) = <(u32, u32, u32)>::deserialize(deserializer)?;
        if file == NO_FILE {
            return Ok(Span { file: FileId::DUMMY, start, end });
        }
        FILES.with(|table| match table.borrow().as_ref() {
            Some(FileTable::Decode { files, lenient }) => {
                let (file, relocation) = files.get(file as usize).ok_or_else(|| de::Error::custom("a span names a file the record does not list"))?;
                match (relocation.map(start), relocation.map(end)) {
                    (Some(start), Some(end)) => Ok(Span { file: *file, start, end }),
                    _ if *lenient => Ok(Span::DUMMY),
                    _ => Err(de::Error::custom("a span points at text that has changed")),
                }
            }
            _ => panic!("a span is read only inside `codec::decode`"),
        })
    }
}

impl Serialize for Symbol {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Symbol {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Symbol, D::Error> {
        struct Text;
        impl Visitor<'_> for Text {
            type Value = Symbol;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a name")
            }
            fn visit_str<E: de::Error>(self, text: &str) -> Result<Symbol, E> {
                Ok(Symbol::intern(text))
            }
        }
        deserializer.deserialize_str(Text)
    }
}

// ---------------------------------------------------------------------------------------------
// Writing

struct Writer {
    out: Vec<u8>,
}

impl Writer {
    fn unsigned(&mut self, mut value: u128) {
        loop {
            let byte = (value & 0x7f) as u8;
            value >>= 7;
            if value == 0 {
                self.out.push(byte);
                return;
            }
            self.out.push(byte | 0x80);
        }
    }

    fn signed(&mut self, value: i128) {
        self.unsigned(((value << 1) ^ (value >> 127)) as u128);
    }

    fn length(&mut self, len: Option<usize>) -> Result<(), Error> {
        let len = len.ok_or_else(|| Error("a sequence of unknown length".to_string()))?;
        self.unsigned(len as u128);
        Ok(())
    }
}

impl<'a> Serializer for &'a mut Writer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<(), Error> {
        self.out.push(v as u8);
        Ok(())
    }
    fn serialize_i8(self, v: i8) -> Result<(), Error> {
        self.signed(v.into());
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<(), Error> {
        self.signed(v.into());
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result<(), Error> {
        self.signed(v.into());
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<(), Error> {
        self.signed(v.into());
        Ok(())
    }
    fn serialize_i128(self, v: i128) -> Result<(), Error> {
        self.signed(v);
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result<(), Error> {
        self.unsigned(v.into());
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<(), Error> {
        self.unsigned(v.into());
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<(), Error> {
        self.unsigned(v.into());
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<(), Error> {
        self.unsigned(v.into());
        Ok(())
    }
    fn serialize_u128(self, v: u128) -> Result<(), Error> {
        self.unsigned(v);
        Ok(())
    }
    fn serialize_f32(self, v: f32) -> Result<(), Error> {
        self.out.extend_from_slice(&v.to_bits().to_le_bytes());
        Ok(())
    }
    fn serialize_f64(self, v: f64) -> Result<(), Error> {
        self.out.extend_from_slice(&v.to_bits().to_le_bytes());
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<(), Error> {
        self.unsigned(u32::from(v).into());
        Ok(())
    }
    fn serialize_str(self, v: &str) -> Result<(), Error> {
        self.serialize_bytes(v.as_bytes())
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<(), Error> {
        self.unsigned(v.len() as u128);
        self.out.extend_from_slice(v);
        Ok(())
    }
    fn serialize_none(self) -> Result<(), Error> {
        self.out.push(0);
        Ok(())
    }
    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<(), Error> {
        self.out.push(1);
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), Error> {
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), Error> {
        Ok(())
    }
    fn serialize_unit_variant(self, _: &'static str, index: u32, _: &'static str) -> Result<(), Error> {
        self.unsigned(index.into());
        Ok(())
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(self, _: &'static str, value: &T) -> Result<(), Error> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(self, _: &'static str, index: u32, _: &'static str, value: &T) -> Result<(), Error> {
        self.unsigned(index.into());
        value.serialize(self)
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<Self, Error> {
        self.length(len)?;
        Ok(self)
    }
    fn serialize_tuple(self, _: usize) -> Result<Self, Error> {
        Ok(self)
    }
    fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Self, Error> {
        Ok(self)
    }
    fn serialize_tuple_variant(self, _: &'static str, index: u32, _: &'static str, _: usize) -> Result<Self, Error> {
        self.unsigned(index.into());
        Ok(self)
    }
    fn serialize_map(self, len: Option<usize>) -> Result<Self, Error> {
        self.length(len)?;
        Ok(self)
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self, Error> {
        Ok(self)
    }
    fn serialize_struct_variant(self, _: &'static str, index: u32, _: &'static str, _: usize) -> Result<Self, Error> {
        self.unsigned(index.into());
        Ok(self)
    }
    fn is_human_readable(&self) -> bool {
        false
    }
}

impl ser::SerializeSeq for &mut Writer {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl ser::SerializeTuple for &mut Writer {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl ser::SerializeTupleStruct for &mut Writer {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl ser::SerializeTupleVariant for &mut Writer {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl ser::SerializeMap for &mut Writer {
    type Ok = ();
    type Error = Error;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Error> {
        key.serialize(&mut **self)
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl ser::SerializeStruct for &mut Writer {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, _: &'static str, value: &T) -> Result<(), Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl ser::SerializeStructVariant for &mut Writer {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, _: &'static str, value: &T) -> Result<(), Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------
// Reading

struct Reader<'de> {
    input: &'de [u8],
}

impl<'de> Reader<'de> {
    fn byte(&mut self) -> Result<u8, Error> {
        let (&first, rest) = self.input.split_first().ok_or_else(|| Error("the record ends early".to_string()))?;
        self.input = rest;
        Ok(first)
    }

    fn unsigned(&mut self) -> Result<u128, Error> {
        let mut value = 0u128;
        let mut shift = 0;
        loop {
            let byte = self.byte()?;
            if shift >= 128 {
                return Err(Error("an integer longer than 128 bits".to_string()));
            }
            value |= u128::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
            shift += 7;
        }
    }

    fn signed(&mut self) -> Result<i128, Error> {
        let raw = self.unsigned()?;
        Ok((raw >> 1) as i128 ^ -((raw & 1) as i128))
    }

    fn fits<T: TryFrom<u128>>(&mut self) -> Result<T, Error> {
        T::try_from(self.unsigned()?).map_err(|_| Error("an integer out of range".to_string()))
    }

    fn fits_signed<T: TryFrom<i128>>(&mut self) -> Result<T, Error> {
        T::try_from(self.signed()?).map_err(|_| Error("an integer out of range".to_string()))
    }

    fn bytes(&mut self) -> Result<&'de [u8], Error> {
        let len: usize = self.fits()?;
        if len > self.input.len() {
            return Err(Error("the record ends early".to_string()));
        }
        let (taken, rest) = self.input.split_at(len);
        self.input = rest;
        Ok(taken)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        let mut out = [0u8; N];
        for byte in &mut out {
            *byte = self.byte()?;
        }
        Ok(out)
    }
}

impl<'de> Deserializer<'de> for &mut Reader<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, _: V) -> Result<V::Value, Error> {
        Err(Error("the record format is not self-describing".to_string()))
    }
    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.byte()? {
            0 => visitor.visit_bool(false),
            1 => visitor.visit_bool(true),
            other => Err(Error(format!("{other} is not a bool"))),
        }
    }
    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_i8(self.fits_signed()?)
    }
    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_i16(self.fits_signed()?)
    }
    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_i32(self.fits_signed()?)
    }
    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_i64(self.fits_signed()?)
    }
    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_i128(self.signed()?)
    }
    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_u8(self.fits()?)
    }
    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_u16(self.fits()?)
    }
    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_u32(self.fits()?)
    }
    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_u64(self.fits()?)
    }
    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_u128(self.unsigned()?)
    }
    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_f32(f32::from_bits(u32::from_le_bytes(self.array()?)))
    }
    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_f64(f64::from_bits(u64::from_le_bytes(self.array()?)))
    }
    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let code: u32 = self.fits()?;
        visitor.visit_char(char::from_u32(code).ok_or_else(|| Error(format!("{code} is not a character")))?)
    }
    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let bytes = self.bytes()?;
        visitor.visit_borrowed_str(std::str::from_utf8(bytes).map_err(|error| Error(error.to_string()))?)
    }
    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }
    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_borrowed_bytes(self.bytes()?)
    }
    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_bytes(visitor)
    }
    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.byte()? {
            0 => visitor.visit_none(),
            1 => visitor.visit_some(self),
            other => Err(Error(format!("{other} is not an option's tag"))),
        }
    }
    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }
    fn deserialize_unit_struct<V: Visitor<'de>>(self, _: &'static str, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }
    fn deserialize_newtype_struct<V: Visitor<'de>>(self, _: &'static str, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_newtype_struct(self)
    }
    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let len = self.fits()?;
        visitor.visit_seq(Items { reader: self, left: len })
    }
    fn deserialize_tuple<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_seq(Items { reader: self, left: len })
    }
    fn deserialize_tuple_struct<V: Visitor<'de>>(self, _: &'static str, len: usize, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_seq(Items { reader: self, left: len })
    }
    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let len = self.fits()?;
        visitor.visit_map(Items { reader: self, left: len })
    }
    fn deserialize_struct<V: Visitor<'de>>(self, _: &'static str, fields: &'static [&'static str], visitor: V) -> Result<V::Value, Error> {
        visitor.visit_seq(Items { reader: self, left: fields.len() })
    }
    fn deserialize_enum<V: Visitor<'de>>(self, _: &'static str, _: &'static [&'static str], visitor: V) -> Result<V::Value, Error> {
        visitor.visit_enum(self)
    }
    fn deserialize_identifier<V: Visitor<'de>>(self, _: V) -> Result<V::Value, Error> {
        Err(Error("the record format names no fields".to_string()))
    }
    fn deserialize_ignored_any<V: Visitor<'de>>(self, _: V) -> Result<V::Value, Error> {
        Err(Error("the record format is not self-describing".to_string()))
    }
    fn is_human_readable(&self) -> bool {
        false
    }
}

struct Items<'a, 'de> {
    reader: &'a mut Reader<'de>,
    left: usize,
}

impl<'de> SeqAccess<'de> for Items<'_, 'de> {
    type Error = Error;
    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>, Error> {
        if self.left == 0 {
            return Ok(None);
        }
        self.left -= 1;
        seed.deserialize(&mut *self.reader).map(Some)
    }
    fn size_hint(&self) -> Option<usize> {
        Some(self.left)
    }
}

impl<'de> MapAccess<'de> for Items<'_, 'de> {
    type Error = Error;
    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>, Error> {
        if self.left == 0 {
            return Ok(None);
        }
        self.left -= 1;
        seed.deserialize(&mut *self.reader).map(Some)
    }
    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Error> {
        seed.deserialize(&mut *self.reader)
    }
    fn size_hint(&self) -> Option<usize> {
        Some(self.left)
    }
}

impl<'de> EnumAccess<'de> for &mut Reader<'de> {
    type Error = Error;
    type Variant = Self;
    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self), Error> {
        let index: u32 = self.fits()?;
        let value = seed.deserialize(index.into_deserializer())?;
        Ok((value, self))
    }
}

impl<'de> VariantAccess<'de> for &mut Reader<'de> {
    type Error = Error;
    fn unit_variant(self) -> Result<(), Error> {
        Ok(())
    }
    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Error> {
        seed.deserialize(self)
    }
    fn tuple_variant<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_seq(Items { reader: self, left: len })
    }
    fn struct_variant<V: Visitor<'de>>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value, Error> {
        visitor.visit_seq(Items { reader: self, left: fields.len() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Shape {
        Dot,
        Line(i64, u8),
        Named { label: String, at: Option<Span>, weight: f64 },
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Record {
        shapes: Vec<Shape>,
        name: Symbol,
        extreme: (i128, u128, char, bool),
        map: std::collections::BTreeMap<u32, String>,
    }

    #[test]
    fn a_record_reads_back_as_written_with_its_files_renumbered() {
        let first = FileId(7);
        let second = FileId(3);
        let record = Record {
            shapes: vec![
                Shape::Dot,
                Shape::Line(-5, 200),
                Shape::Named { label: "é".into(), at: Some(Span::new(first, 1, 4)), weight: -0.5 },
                Shape::Named { label: String::new(), at: Some(Span::new(second, 0, 0)), weight: f64::INFINITY },
                Shape::Named { label: "x".into(), at: Some(Span::DUMMY), weight: 1.0 },
            ],
            name: Symbol::intern("position"),
            extreme: (i128::MIN, u128::MAX, '\u{10FFFF}', true),
            map: [(1, "one".to_string()), (300, "three hundred".to_string())].into_iter().collect(),
        };
        let (bytes, extents) = encode(|| to_bytes(&record).expect("writes"));
        assert_eq!(
            extents,
            vec![Extent { file: first, ranges: vec![(1, 4)] }, Extent { file: second, ranges: vec![(0, 1)] }],
            "files are numbered as first met, each with the bytes its spans cover"
        );
        let files: Vec<FileId> = extents.iter().map(|extent| extent.file).collect();
        // Read back in a run where the files have other ids.
        let read: Record = decode(vec![FileId(0), FileId(1)], || from_bytes(&bytes).expect("reads"));
        assert_eq!(read.shapes[2], Shape::Named { label: "é".into(), at: Some(Span::new(FileId(0), 1, 4)), weight: -0.5 });
        assert_eq!(read.shapes[3], Shape::Named { label: String::new(), at: Some(Span::new(FileId(1), 0, 0)), weight: f64::INFINITY });
        assert_eq!(read.shapes[4], Shape::Named { label: "x".into(), at: Some(Span::DUMMY), weight: 1.0 });
        assert_eq!(read.name, record.name);
        assert_eq!(read.extreme, record.extreme);
        assert_eq!(read.map, record.map);
        // A truncated record and one with bytes left over are refused.
        assert!(decode(files.clone(), || from_bytes::<Record>(&bytes[..bytes.len() - 1])).is_err());
        let mut longer = bytes.clone();
        longer.push(0);
        assert!(decode(files, || from_bytes::<Record>(&longer)).is_err());
    }

    #[test]
    fn a_record_reads_back_with_its_spans_moved_to_where_their_text_is() {
        let file = FileId(4);
        let spans = vec![Span::new(file, 10, 20), Span::new(file, 25, 30), Span::new(file, 40, 41)];
        let ((bytes, (_, scoped)), extents) = encode(|| {
            let bytes = to_bytes(&spans).expect("writes");
            (bytes, scope(|| to_bytes(&spans[1]).expect("writes")))
        });
        assert_eq!(extents, vec![Extent { file, ranges: vec![(10, 20), (25, 30), (40, 41)] }]);
        assert_eq!(scoped, vec![(0, vec![(25, 30)])], "a scope gathers what is written inside it");
        // The text from 0 to 24 kept its place; from 24 to 35 it moved 6 bytes on; 40..41 3 back.
        let relocation = Relocation::pieces(vec![(0, 24, 0), (24, 11, 30), (40, 1, 37)]);
        let read: Vec<Span> = decode_relocated(vec![(FileId(9), relocation.clone())], false, || from_bytes(&bytes).expect("reads"));
        assert_eq!(read, vec![Span::new(FileId(9), 10, 20), Span::new(FileId(9), 31, 36), Span::new(FileId(9), 37, 38)]);
        // A span no piece holds: an error, or the dummy span read leniently.
        let partial = Relocation::pieces(vec![(0, 24, 0)]);
        assert!(decode_relocated(vec![(file, partial.clone())], false, || from_bytes::<Vec<Span>>(&bytes)).is_err());
        let lenient: Vec<Span> = decode_relocated(vec![(file, partial)], true, || from_bytes(&bytes).expect("reads"));
        assert_eq!(lenient, vec![Span::new(file, 10, 20), Span::DUMMY, Span::DUMMY]);
    }

    #[test]
    fn a_files_extent_is_the_bytes_its_spans_cover_merged() {
        let file = FileId(2);
        let spans = vec![Span::new(file, 10, 20), Span::new(file, 12, 15), Span::new(file, 30, 30), Span::new(file, 5, 11), Span::new(file, 40, 44), Span::new(file, 44, 46)];
        let ((), extents) = encode(|| {
            to_bytes(&spans).expect("writes");
        });
        assert_eq!(extents, vec![Extent { file, ranges: vec![(5, 20), (30, 31), (40, 46)] }]);
    }
}
