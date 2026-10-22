//! `[BLD-1]`, `[BLD-2]` — the types a module's kept checked form names, as a later run finds them.
//!
//! A run's `Ty`, `StructId`, `EnumId`, `ClassId` and `RangeId` are numbers into its own table. While
//! a record is written ([`encode`]) each one it names is numbered in the record's [`Numbering`];
//! [`record`] then describes every numbered type by what it is ([`TypeRecord`]: a scalar's kind, a
//! declared type's name, a generic's instance by its generic and arguments, the definition of a type
//! the checker made for one body), and a later run finds each again (`ember_typeck`'s `reuse`) and
//! reads the record with its own numbers ([`decode`]).

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use ember_span::{Span, Symbol};

use crate::{ClassId, ConstNorm, EnumId, FieldDef, FieldVis, FloatTy, FnParamMode, IntTy, RangeId, StructDef, StructId, Ty, TyKind, TypeTable, UintTy};

/// One kind of number: those met, in the order met.
pub struct Numbered<T> {
    pub items: Vec<T>,
    index: HashMap<T, u32>,
}

impl<T> Default for Numbered<T> {
    fn default() -> Self {
        Numbered { items: Vec::new(), index: HashMap::new() }
    }
}

impl<T: Copy + Eq + Hash> Numbered<T> {
    pub fn number(&mut self, item: T) -> u32 {
        *self.index.entry(item).or_insert_with(|| {
            self.items.push(item);
            self.items.len() as u32 - 1
        })
    }
}

/// The types a record names, numbered as they were met.
#[derive(Default)]
pub struct Numbering {
    pub tys: Numbered<Ty>,
    pub structs: Numbered<StructId>,
    pub enums: Numbered<EnumId>,
    pub classes: Numbered<ClassId>,
    pub ranges: Numbered<RangeId>,
}

/// This run's number for each of a record's: what [`decode`] reads a record with. `None` for one
/// this run does not make again (a closure of a function checked again): reading it is an error.
#[derive(Clone, Default, Debug)]
pub struct Found {
    pub tys: Vec<Option<Ty>>,
    pub structs: Vec<Option<StructId>>,
    pub enums: Vec<Option<EnumId>>,
    pub classes: Vec<Option<ClassId>>,
    pub ranges: Vec<Option<RangeId>>,
}

enum Table {
    Encode(Numbering),
    Decode(Found),
}

thread_local! {
    static TABLE: RefCell<Option<Table>> = const { RefCell::new(None) };
}

/// Run `write` numbering each type it writes.
pub fn encode<R>(write: impl FnOnce() -> R) -> (R, Numbering) {
    let outer = TABLE.with(|table| table.replace(Some(Table::Encode(Numbering::default()))));
    let result = write();
    match TABLE.with(|table| table.replace(outer)) {
        Some(Table::Encode(numbering)) => (result, numbering),
        _ => unreachable!("the table set above"),
    }
}

/// Run `read` with each type number read as this run's.
pub fn decode<R>(found: Found, read: impl FnOnce() -> R) -> R {
    let outer = TABLE.with(|table| table.replace(Some(Table::Decode(found))));
    let result = read();
    TABLE.with(|table| table.replace(outer));
    result
}

macro_rules! numbered_id {
    ($ty:ty, $field:ident, $what:literal) => {
        impl Serialize for $ty {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let number = TABLE.with(|table| match table.borrow_mut().as_mut() {
                    Some(Table::Encode(numbering)) => numbering.$field.number(*self),
                    _ => panic!(concat!("a ", $what, " is written only inside `codec::encode`")),
                });
                serializer.serialize_u32(number)
            }
        }

        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let number = u32::deserialize(deserializer)? as usize;
                TABLE
                    .with(|table| match table.borrow().as_ref() {
                        Some(Table::Decode(found)) => found.$field.get(number).copied().flatten(),
                        _ => panic!(concat!("a ", $what, " is read only inside `codec::decode`")),
                    })
                    .ok_or_else(|| serde::de::Error::custom(concat!("a ", $what, " the record does not list")))
            }
        }
    };
}

numbered_id!(Ty, tys, "type");
numbered_id!(StructId, structs, "struct");
numbered_id!(EnumId, enums, "enum");
numbered_id!(ClassId, classes, "class");
numbered_id!(RangeId, ranges, "range type");

/// A type's kind, its parts by their numbers in the record.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KindRecord {
    Bool,
    Char,
    Int(IntTy),
    Uint(UintTy),
    Float(FloatTy),
    Void,
    Never,
    Str,
    CLongDouble,
    VaList,
    CStr,
    Span { elem: u32, mutable: bool },
    Opaque(Symbol),
    Struct(u32),
    Class(u32),
    ClassInterface(Symbol),
    Enum(u32),
    Range(u32),
    Tuple(Vec<u32>),
    Ref { mutable: bool, inner: u32 },
    Ptr { mutable: bool, inner: u32 },
    Array { elem: u32, len: u64 },
    ArrayN { elem: u32, len: ConstNorm },
    Const(ConstNorm),
    Vec { elem: u32, text: bool },
    Fn { abi: Option<Symbol>, latebound: bool, stateless: bool, is_unsafe: bool, once: bool, params: Vec<(u32, FnParamMode)>, ret: u32 },
    Dyn { interfaces: Vec<Symbol> },
    Param { index: u32, name: Symbol },
    Assoc { name: Symbol },
}

/// A nominal type (a struct, an enum or a class): found again by its name, made again as its
/// generic's instance over its arguments, or — for one the checker made for a body, which no name
/// finds (a closure's environment) — defined again as it was.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NominalRecord {
    pub name: Symbol,
    pub origin: Option<(Symbol, Vec<u32>)>,
    pub made: Option<MadeRecord>,
    /// What checking a body marked the definition with (a scope's end joins its threads, a guard
    /// releases its lock): `has_drop`, and a struct's `origin` as the definition held it. A build
    /// that does not check that body again marks it so.
    pub has_drop: bool,
    pub marked: Option<(Symbol, Vec<u32>)>,
}

/// The definition of a type the checker made for one body: a closure's environment, with what
/// calling it calls.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MadeRecord {
    Struct {
        fields: Vec<FieldRecord>,
        span: Span,
        derives_copy: bool,
        has_drop: bool,
        drops_fields: bool,
        declaring_module: u64,
        call: Option<CallRecord>,
    },
}

/// What calling a closure's environment calls: the closure's function, by its symbol, and that
/// function's signature, as the checker knows it.
#[derive(Clone, Debug)]
pub struct MadeCall {
    pub symbol: String,
    pub once: bool,
    pub owned: bool,
    pub params: Vec<(Symbol, Ty, FnParamMode, Span)>,
    pub ret: Ty,
    pub is_unsafe: bool,
    pub overflow: crate::OverflowPolicy,
    pub fp: crate::FpMode,
    pub decl: Span,
}

/// [`MadeCall`], its types by their numbers in the record.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CallRecord {
    pub symbol: String,
    pub once: bool,
    pub owned: bool,
    pub params: Vec<(Symbol, u32, FnParamMode, Span)>,
    pub ret: u32,
    pub is_unsafe: bool,
    pub overflow: crate::OverflowPolicy,
    pub fp: crate::FpMode,
    pub decl: Span,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FieldRecord {
    pub name: Symbol,
    pub ty: u32,
    pub span: Span,
    pub ty_span: Span,
    pub has_default: bool,
    pub is_let: bool,
    pub read_only_outside: bool,
    pub vis: FieldVis,
}

/// A range type (`[RNG-1]`): found again by its name and where it is declared.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RangeRecord {
    pub name: Symbol,
    pub span: Span,
}

/// Every type a record numbered, described.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TypeRecord {
    pub tys: Vec<KindRecord>,
    pub structs: Vec<NominalRecord>,
    pub enums: Vec<NominalRecord>,
    pub classes: Vec<NominalRecord>,
    pub ranges: Vec<RangeRecord>,
}

/// Why a type cannot be recorded: only what a checked program holds can be.
#[derive(Debug)]
pub struct Unrecordable(pub String);

/// Describe every type `numbering` holds, numbering the parts each names too. `made` — for each
/// struct the checker made for a body, which no name finds again, what calling it calls (when it
/// is a closure's environment); `None` for every other struct.
pub fn record(types: &TypeTable, numbering: &mut Numbering, made: &dyn Fn(StructId) -> Option<Option<MadeCall>>) -> Result<TypeRecord, Unrecordable> {
    let mut out = TypeRecord::default();
    loop {
        let mut progress = false;
        while out.tys.len() < numbering.tys.items.len() {
            let ty = numbering.tys.items[out.tys.len()];
            let kind = kind_record(types.kind(ty), numbering)?;
            out.tys.push(kind);
            progress = true;
        }
        while out.structs.len() < numbering.structs.items.len() {
            let id = numbering.structs.items[out.structs.len()];
            let def = types.struct_def(id);
            let origin = types.struct_instance(id).map(|(generic, args)| (generic, args.iter().map(|ty| numbering.tys.number(*ty)).collect()));
            let made = made(id).filter(|_| origin.is_none()).map(|call| MadeRecord::Struct {
                fields: def.fields.iter().map(|field| field_record(field, numbering)).collect(),
                span: def.span,
                derives_copy: def.derives_copy,
                has_drop: def.has_drop,
                drops_fields: def.drops_fields,
                declaring_module: def.declaring_module as u64,
                call: call.map(|call| CallRecord {
                    symbol: call.symbol,
                    once: call.once,
                    owned: call.owned,
                    params: call.params.iter().map(|(name, ty, mode, span)| (*name, numbering.tys.number(*ty), *mode, *span)).collect(),
                    ret: numbering.tys.number(call.ret),
                    is_unsafe: call.is_unsafe,
                    overflow: call.overflow,
                    fp: call.fp,
                    decl: call.decl,
                }),
            });
            let marked = def.origin.as_ref().map(|(generic, args)| (*generic, args.iter().map(|ty| numbering.tys.number(*ty)).collect()));
            out.structs.push(NominalRecord { name: def.name, origin, made, has_drop: def.has_drop, marked });
            progress = true;
        }
        while out.enums.len() < numbering.enums.items.len() {
            let id = numbering.enums.items[out.enums.len()];
            let def = types.enum_def(id);
            let origin = types.enum_instance(id).map(|(generic, args)| (generic, args.iter().map(|ty| numbering.tys.number(*ty)).collect()));
            out.enums.push(NominalRecord { name: def.name, origin, made: None, has_drop: def.has_drop, marked: None });
            progress = true;
        }
        while out.classes.len() < numbering.classes.items.len() {
            let id = numbering.classes.items[out.classes.len()];
            let def = types.class_def(id);
            let origin = types.class_instance(id).map(|(generic, args)| (generic, args.iter().map(|ty| numbering.tys.number(*ty)).collect()));
            out.classes.push(NominalRecord { name: def.name, origin, made: None, has_drop: def.has_drop, marked: None });
            progress = true;
        }
        while out.ranges.len() < numbering.ranges.items.len() {
            let def = types.range_def(numbering.ranges.items[out.ranges.len()]);
            out.ranges.push(RangeRecord { name: def.name, span: def.span });
            progress = true;
        }
        if !progress {
            return Ok(out);
        }
    }
}

fn field_record(field: &FieldDef, numbering: &mut Numbering) -> FieldRecord {
    FieldRecord {
        name: field.name,
        ty: numbering.tys.number(field.ty),
        span: field.span,
        ty_span: field.ty_span,
        has_default: field.has_default,
        is_let: field.is_let,
        read_only_outside: field.read_only_outside,
        vis: field.vis,
    }
}

fn kind_record(kind: &TyKind, numbering: &mut Numbering) -> Result<KindRecord, Unrecordable> {
    let mut ty = |ty: Ty| numbering.tys.number(ty);
    Ok(match kind {
        TyKind::Bool => KindRecord::Bool,
        TyKind::Char => KindRecord::Char,
        TyKind::Int(int) => KindRecord::Int(*int),
        TyKind::Uint(uint) => KindRecord::Uint(*uint),
        TyKind::Float(float) => KindRecord::Float(*float),
        TyKind::Void => KindRecord::Void,
        TyKind::Never => KindRecord::Never,
        TyKind::Str => KindRecord::Str,
        TyKind::CLongDouble => KindRecord::CLongDouble,
        TyKind::VaList => KindRecord::VaList,
        TyKind::CStr => KindRecord::CStr,
        TyKind::Span { elem, mutable } => KindRecord::Span { elem: ty(*elem), mutable: *mutable },
        TyKind::Opaque(name) => KindRecord::Opaque(*name),
        TyKind::Struct(id) => KindRecord::Struct(numbering.structs.number(*id)),
        TyKind::Class(id) => KindRecord::Class(numbering.classes.number(*id)),
        TyKind::ClassInterface(name) => KindRecord::ClassInterface(*name),
        TyKind::Enum(id) => KindRecord::Enum(numbering.enums.number(*id)),
        TyKind::Range(id) => KindRecord::Range(numbering.ranges.number(*id)),
        TyKind::Tuple(items) => KindRecord::Tuple(items.iter().map(|item| ty(*item)).collect()),
        TyKind::Ref { mutable, inner } => KindRecord::Ref { mutable: *mutable, inner: ty(*inner) },
        TyKind::Ptr { mutable, inner } => KindRecord::Ptr { mutable: *mutable, inner: ty(*inner) },
        TyKind::Array { elem, len } => KindRecord::Array { elem: ty(*elem), len: *len },
        TyKind::ArrayN { elem, len } => KindRecord::ArrayN { elem: ty(*elem), len: len.clone() },
        TyKind::Const(value) => KindRecord::Const(value.clone()),
        TyKind::Vec { elem, text } => KindRecord::Vec { elem: ty(*elem), text: *text },
        TyKind::Fn { abi, latebound, stateless, is_unsafe, once, params, ret } => KindRecord::Fn {
            abi: *abi,
            latebound: *latebound,
            stateless: *stateless,
            is_unsafe: *is_unsafe,
            once: *once,
            params: params.iter().map(|param| (ty(param.ty), param.mode)).collect(),
            ret: ty(*ret),
        },
        TyKind::Dyn { interfaces } => KindRecord::Dyn { interfaces: interfaces.clone() },
        TyKind::Param { index, name } => KindRecord::Param { index: *index, name: *name },
        TyKind::Assoc { name } => KindRecord::Assoc { name: *name },
        TyKind::Infer(_) | TyKind::IntLit | TyKind::FloatLit | TyKind::Error => {
            return Err(Unrecordable(format!("an unresolved type ({kind:?}) in a checked program")));
        }
    })
}

/// This run's number for a record's: [`Found`], or what of it has been found so far.
pub trait Lookup {
    fn ty(&self, number: u32) -> Option<Ty>;
    fn struct_id(&self, number: u32) -> Option<StructId>;
    fn enum_id(&self, number: u32) -> Option<EnumId>;
    fn class_id(&self, number: u32) -> Option<ClassId>;
    fn range_id(&self, number: u32) -> Option<RangeId>;
}

impl Lookup for Found {
    fn ty(&self, number: u32) -> Option<Ty> {
        self.tys.get(number as usize).copied().flatten()
    }
    fn struct_id(&self, number: u32) -> Option<StructId> {
        self.structs.get(number as usize).copied().flatten()
    }
    fn enum_id(&self, number: u32) -> Option<EnumId> {
        self.enums.get(number as usize).copied().flatten()
    }
    fn class_id(&self, number: u32) -> Option<ClassId> {
        self.classes.get(number as usize).copied().flatten()
    }
    fn range_id(&self, number: u32) -> Option<RangeId> {
        self.ranges.get(number as usize).copied().flatten()
    }
}

impl KindRecord {
    /// The kind, its parts this run's (`tys`, `structs`, … by the record's numbers).
    pub fn kind(&self, found: &impl Lookup) -> Option<TyKind> {
        let ty = |number: &u32| found.ty(*number);
        Some(match self {
            KindRecord::Bool => TyKind::Bool,
            KindRecord::Char => TyKind::Char,
            KindRecord::Int(int) => TyKind::Int(*int),
            KindRecord::Uint(uint) => TyKind::Uint(*uint),
            KindRecord::Float(float) => TyKind::Float(*float),
            KindRecord::Void => TyKind::Void,
            KindRecord::Never => TyKind::Never,
            KindRecord::Str => TyKind::Str,
            KindRecord::CLongDouble => TyKind::CLongDouble,
            KindRecord::VaList => TyKind::VaList,
            KindRecord::CStr => TyKind::CStr,
            KindRecord::Span { elem, mutable } => TyKind::Span { elem: ty(elem)?, mutable: *mutable },
            KindRecord::Opaque(name) => TyKind::Opaque(*name),
            KindRecord::Struct(id) => TyKind::Struct(found.struct_id(*id)?),
            KindRecord::Class(id) => TyKind::Class(found.class_id(*id)?),
            KindRecord::ClassInterface(name) => TyKind::ClassInterface(*name),
            KindRecord::Enum(id) => TyKind::Enum(found.enum_id(*id)?),
            KindRecord::Range(id) => TyKind::Range(found.range_id(*id)?),
            KindRecord::Tuple(items) => TyKind::Tuple(items.iter().map(ty).collect::<Option<_>>()?),
            KindRecord::Ref { mutable, inner } => TyKind::Ref { mutable: *mutable, inner: ty(inner)? },
            KindRecord::Ptr { mutable, inner } => TyKind::Ptr { mutable: *mutable, inner: ty(inner)? },
            KindRecord::Array { elem, len } => TyKind::Array { elem: ty(elem)?, len: *len },
            KindRecord::ArrayN { elem, len } => TyKind::ArrayN { elem: ty(elem)?, len: len.clone() },
            KindRecord::Const(value) => TyKind::Const(value.clone()),
            KindRecord::Vec { elem, text } => TyKind::Vec { elem: ty(elem)?, text: *text },
            KindRecord::Fn { abi, latebound, stateless, is_unsafe, once, params, ret } => TyKind::Fn {
                abi: *abi,
                latebound: *latebound,
                stateless: *stateless,
                is_unsafe: *is_unsafe,
                once: *once,
                params: params.iter().map(|(param, mode)| Some(crate::FnParam { ty: ty(param)?, mode: *mode })).collect::<Option<_>>()?,
                ret: ty(ret)?,
            },
            KindRecord::Dyn { interfaces } => TyKind::Dyn { interfaces: interfaces.clone() },
            KindRecord::Param { index, name } => TyKind::Param { index: *index, name: *name },
            KindRecord::Assoc { name } => TyKind::Assoc { name: *name },
        })
    }

    /// The record numbers of the types this kind names directly, which are found before it.
    pub fn parts(&self) -> Vec<Part> {
        match self {
            KindRecord::Span { elem, .. } | KindRecord::Array { elem, .. } | KindRecord::ArrayN { elem, .. } | KindRecord::Vec { elem, .. } => {
                vec![Part::Ty(*elem)]
            }
            KindRecord::Ref { inner, .. } | KindRecord::Ptr { inner, .. } => vec![Part::Ty(*inner)],
            KindRecord::Tuple(items) => items.iter().map(|item| Part::Ty(*item)).collect(),
            KindRecord::Fn { params, ret, .. } => params.iter().map(|(param, _)| Part::Ty(*param)).chain([Part::Ty(*ret)]).collect(),
            KindRecord::Struct(id) => vec![Part::Struct(*id)],
            KindRecord::Class(id) => vec![Part::Class(*id)],
            KindRecord::Enum(id) => vec![Part::Enum(*id)],
            KindRecord::Range(id) => vec![Part::Range(*id)],
            _ => Vec::new(),
        }
    }
}

/// One numbered type a kind or a nominal record names.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Part {
    Ty(u32),
    Struct(u32),
    Enum(u32),
    Class(u32),
    Range(u32),
}

impl MadeRecord {
    /// Where the closure the struct is the environment of is written.
    pub fn span(&self) -> Span {
        let MadeRecord::Struct { span, .. } = self;
        *span
    }

    /// The struct's definition, its field types this run's.
    pub fn struct_def(&self, name: Symbol, found: &impl Lookup) -> Option<StructDef> {
        let MadeRecord::Struct { fields, span, derives_copy, has_drop, drops_fields, declaring_module, .. } = self;
        Some(StructDef {
            name,
            fields: fields
                .iter()
                .map(|field| {
                    Some(FieldDef {
                        name: field.name,
                        ty: found.ty(field.ty)?,
                        span: field.span,
                        ty_span: field.ty_span,
                        has_default: field.has_default,
                        is_let: field.is_let,
                        read_only_outside: field.read_only_outside,
                        vis: field.vis,
                    })
                })
                .collect::<Option<_>>()?,
            span: *span,
            derives_copy: *derives_copy,
            has_drop: *has_drop,
            drops_fields: *drops_fields,
            origin: None,
            declaring_module: *declaring_module as usize,
        })
    }

    /// The record numbers of its fields' types.
    pub fn field_types(&self) -> Vec<u32> {
        let MadeRecord::Struct { fields, .. } = self;
        fields.iter().map(|field| field.ty).collect()
    }

    /// The record numbers of the types its call names.
    pub fn call_types(&self) -> Vec<u32> {
        let MadeRecord::Struct { call, .. } = self;
        call.iter().flat_map(|call| call.params.iter().map(|(_, ty, _, _)| *ty).chain([call.ret])).collect()
    }

    /// What calling it calls, its types this run's.
    pub fn call(&self, found: &impl Lookup) -> Option<Option<MadeCall>> {
        let MadeRecord::Struct { call, .. } = self;
        let Some(call) = call else { return Some(None) };
        Some(Some(MadeCall {
            symbol: call.symbol.clone(),
            once: call.once,
            owned: call.owned,
            params: call.params.iter().map(|(name, ty, mode, span)| Some((*name, found.ty(*ty)?, *mode, *span))).collect::<Option<_>>()?,
            ret: found.ty(call.ret)?,
            is_unsafe: call.is_unsafe,
            overflow: call.overflow,
            fp: call.fp,
            decl: call.decl,
        }))
    }
}
