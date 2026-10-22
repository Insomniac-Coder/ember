//! `[SIMD-1]`, `[SIMD-4]`, `[SIMD-8]`, `[SIMD-9]`, `[CG-LL-2]` — `std.simd`'s vectors and masks as
//! LLVM vectors (`<N x T>`), each operation one vector instruction or intrinsic where LLVM has one
//! that keeps the runtime header's rules (`ember_simd.h`), and lane by lane where the rule is an
//! order (a reduction adds lane 0, then each next lane) or a platform's estimate (`rsqrt_approx`
//! and `rcp_approx` of `f32` lanes are SSE's, as the C backend's are on x86-64):
//! - integer lanes check overflow (`llvm.*.with.overflow`, one panic after every lane), wrap, or
//!   saturate (`llvm.*.sat`, `llvm.*mul.fix.sat`) as the function's `@overflow` asks;
//! - float lanes are IEEE operations, one rounding each, never contracted;
//! - a checked load, store, gather or scatter panics naming the first lane out of bounds.

use crate::func::{FnCx, Val};
use crate::types::Sc;
use crate::{Result, unsupported};
use ember_mir::{BinOp, Operand, Place, SimdOp};
use ember_types::{OverflowPolicy, SimdLane, SimdShape, Ty, TyKind};

/// A vector type's IR: `<N x T>`, its lane type, and its shape.
#[derive(Clone)]
struct Vector {
    ir: String,
    lane: String,
    shape: SimdShape,
}

impl Vector {
    fn float(&self) -> bool {
        self.shape.lane == SimdLane::Float
    }

    fn signed(&self) -> bool {
        matches!(self.shape.lane, SimdLane::Signed | SimdLane::Mask)
    }

    fn lanes(&self) -> u32 {
        self.shape.lanes
    }

    /// The suffix an intrinsic names this type by (`v4f32`).
    fn suffix(&self) -> String {
        let lane = match (self.float(), self.shape.bits) {
            (true, 32) => "f32".to_string(),
            (true, _) => "f64".to_string(),
            (false, bits) => format!("i{bits}"),
        };
        format!("v{}{lane}", self.shape.lanes)
    }

    /// The vector of `i1` a comparison gives.
    fn bools(&self) -> String {
        format!("<{} x i1>", self.shape.lanes)
    }
}

fn vector_of(m: &crate::Module<'_>, ty: Ty) -> Result<Vector> {
    let table = m.ty.table;
    let ty = match table.kind(ty) {
        TyKind::Ref { inner, .. } => *inner,
        _ => ty,
    };
    let TyKind::Struct(id) = *table.kind(ty) else { return unsupported("a vector that is not a `std.simd` type") };
    let Some(shape) = SimdShape::of(table.struct_def(id).name.as_str()) else { return unsupported("a vector that is not a `std.simd` type") };
    let lane = match (shape.lane, shape.bits) {
        (SimdLane::Float, 32) => "float".to_string(),
        (SimdLane::Float, _) => "double".to_string(),
        (_, bits) => format!("i{bits}"),
    };
    Ok(Vector { ir: format!("<{} x {lane}>", shape.lanes), lane, shape })
}

impl<'a> FnCx<'_, 'a> {
    /// A vector operand's value.
    fn vector(&mut self, operand: &Operand, v: &Vector) -> Result<String> {
        let value = self.operand(operand)?;
        let bytes = u64::from(v.shape.bytes());
        let (at, align) = self.spill(&value, bytes, bytes);
        Ok(self.value(format!("load {}, ptr {at}, align {}", v.ir, align.min(bytes))))
    }

    /// A vector value written to `dest`.
    fn store_vector(&mut self, dest: &Place, v: &Vector, value: &str) -> Result<()> {
        let Some((at, align)) = self.dest_of(dest)? else { return Ok(()) };
        self.emit(format!("store {} {value}, ptr {at}, align {}", v.ir, align.min(u64::from(v.shape.bytes()))));
        Ok(())
    }

    /// Every lane `scalar`.
    fn splat(&mut self, v: &Vector, scalar: &str) -> String {
        let one = self.value(format!("insertelement {} poison, {} {scalar}, i64 0", v.ir, v.lane));
        self.value(format!("shufflevector {} {one}, {} poison, <{} x i32> zeroinitializer", v.ir, v.ir, v.lanes()))
    }

    /// A lane's value as `T`: a `bool` operand made all ones or zero, an integer its width.
    fn lane_scalar(&mut self, operand: &Operand, v: &Vector) -> Result<String> {
        let Val::Imm(sc, value) = self.operand(operand)? else { return unsupported("a vector lane held in memory") };
        Ok(match (sc, v.shape.lane) {
            (Sc::Bool, _) => self.value(format!("select i1 {value}, {} -1, {} 0", v.lane, v.lane)),
            (Sc::I(from), SimdLane::Signed | SimdLane::Unsigned | SimdLane::Mask) if from != v.shape.bits => {
                self.int_convert(&value, from, v.shape.bits, v.signed())
            }
            _ => value,
        })
    }

    fn intrinsic(&mut self, name: &str, ret: &str, params: &[&str]) -> String {
        let declared = name.to_string();
        let line = format!("declare {ret} @{name}({})", params.join(", "));
        self.m.declare(&declared, || line);
        format!("@{name}")
    }

    /// `[SIMD-8]` — a lane-wise `+ - *` on integer lanes under `policy`; checked, one panic after
    /// every lane, naming the operator.
    fn int_arith(&mut self, op: BinOp, policy: OverflowPolicy, v: &Vector, a: &str, b: &str, span: ember_span::Span) -> Result<String> {
        let (plain, checked, symbol) = match op {
            BinOp::Add => ("add", "add", "+"),
            BinOp::Sub => ("sub", "sub", "-"),
            BinOp::Mul => ("mul", "mul", "*"),
            _ => return unsupported("an integer vector operator other than `+ - *`"),
        };
        let sign = if v.signed() { "s" } else { "u" };
        Ok(match policy {
            OverflowPolicy::Wrap => self.value(format!("{plain} {} {a}, {b}", v.ir)),
            OverflowPolicy::Saturate => {
                if op == BinOp::Mul {
                    let name = format!("llvm.{sign}mul.fix.sat.{}", v.suffix());
                    let f = self.intrinsic(&name, &v.ir, &[&v.ir, &v.ir, "i32"]);
                    self.value(format!("call {} {f}({} {a}, {} {b}, i32 0)", v.ir, v.ir, v.ir))
                } else {
                    let name = format!("llvm.{sign}{checked}.sat.{}", v.suffix());
                    let f = self.intrinsic(&name, &v.ir, &[&v.ir, &v.ir]);
                    self.value(format!("call {} {f}({} {a}, {} {b})", v.ir, v.ir, v.ir))
                }
            }
            OverflowPolicy::Panic => {
                let pair = format!("{{ {}, {} }}", v.ir, v.bools());
                let name = format!("llvm.{sign}{checked}.with.overflow.{}", v.suffix());
                let f = self.intrinsic(&name, &pair, &[&v.ir, &v.ir]);
                let both = self.value(format!("call {pair} {f}({} {a}, {} {b})", v.ir, v.ir));
                let result = self.value(format!("extractvalue {pair} {both}, 0"));
                let flags = self.value(format!("extractvalue {pair} {both}, 1"));
                let any = self.any_lane(&flags, v);
                self.panic_overflow_if(&any, symbol, span);
                result
            }
        })
    }

    /// Whether any lane of a vector of `i1` is set.
    fn any_lane(&mut self, flags: &str, v: &Vector) -> String {
        let name = format!("llvm.vector.reduce.or.v{}i1", v.lanes());
        let bools = v.bools();
        let f = self.intrinsic(&name, "i1", &[&bools]);
        self.value(format!("call i1 {f}({bools} {flags})"))
    }

    fn panic_overflow_if(&mut self, fails: &str, symbol: &str, span: ember_span::Span) {
        let (fail, ok) = (self.label(), self.label());
        self.emit(format!("br i1 {fails}, label %{fail}, label %{ok}"));
        self.start(&fail);
        let text = self.m.bytes(symbol.as_bytes());
        let loc = self.loc(span);
        self.call_rt("panic_overflow", vec![Val::Imm(Sc::Ptr, text), loc], None);
        self.start(&ok);
    }

    /// Lane `index` (a constant) of `vector`.
    fn lane(&mut self, vector: &str, v: &Vector, index: u32) -> String {
        self.value(format!("extractelement {} {vector}, i64 {index}", v.ir))
    }

    /// `a < b ? a : b` (or `>` for `max`) lane by lane: the second operand where the first is
    /// not smaller, NaN and signed zeros included.
    fn lane_min_max(&mut self, v: &Vector, a: &str, b: &str, max: bool) -> String {
        let compare = match (v.float(), max, v.signed()) {
            (true, false, _) => format!("fcmp olt {} {a}, {b}", v.ir),
            (true, true, _) => format!("fcmp ogt {} {a}, {b}", v.ir),
            (false, false, true) => format!("icmp slt {} {a}, {b}", v.ir),
            (false, false, false) => format!("icmp ult {} {a}, {b}", v.ir),
            (false, true, true) => format!("icmp sgt {} {a}, {b}", v.ir),
            (false, true, false) => format!("icmp ugt {} {a}, {b}", v.ir),
        };
        let first = self.value(compare);
        self.value(format!("select {} {first}, {} {a}, {} {b}", v.bools(), v.ir, v.ir))
    }

    /// `[SIMD-9]` — the first lane of `bad` (a vector of `i1`) out of bounds panics, naming the lane,
    /// its index and the view's length.
    fn check_lanes(&mut self, bad: &str, v: &Vector, indices: &[String], len: &str, span: ember_span::Span) {
        let any = self.any_lane(bad, v);
        let (fail, ok) = (self.label(), self.label());
        self.emit(format!("br i1 {any}, label %{fail}, label %{ok}"));
        self.start(&fail);
        let lanes = v.lanes();
        let bits = self.value(format!("bitcast {} {bad} to i{lanes}", v.bools()));
        let name = format!("llvm.cttz.i{lanes}");
        let ty = format!("i{lanes}");
        let f = self.intrinsic(&name, &ty, &[&ty, "i1"]);
        let first = self.value(format!("call i{lanes} {f}(i{lanes} {bits}, i1 true)"));
        let lane = self.int_convert(&first, lanes, 64, false);
        // The first bad lane's index: a select down the lanes.
        let mut index = indices[0].clone();
        for (at, candidate) in indices.iter().enumerate().skip(1) {
            let here = self.value(format!("icmp eq i64 {lane}, {at}"));
            index = self.value(format!("select i1 {here}, i64 {candidate}, i64 {index}"));
        }
        let loc = self.loc(span);
        self.call_rt(
            "panic_lane_bounds",
            vec![Val::Imm(Sc::I(64), lane), Val::Imm(Sc::I(64), index), Val::Imm(Sc::I(64), len.to_string()), loc],
            None,
        );
        self.start(&ok);
    }

    /// A gather's or scatter's index lanes, each as a 64-bit index, and the vector of lanes out of
    /// bounds (negative, or at least `len`).
    fn index_lanes(&mut self, operand: &Operand, index: Ty, len: &str) -> Result<(Vec<String>, String, Vector)> {
        let iv = vector_of(self.m, index)?;
        let indices = self.vector(operand, &iv)?;
        let mut wide = Vec::with_capacity(iv.lanes() as usize);
        for lane in 0..iv.lanes() {
            let value = self.lane(&indices, &iv, lane);
            wide.push(self.int_convert(&value, iv.shape.bits, 64, iv.signed()));
        }
        let extend = if iv.signed() { "sext" } else { "zext" };
        let as_words = if iv.shape.bits == 64 {
            indices.clone()
        } else {
            self.value(format!("{extend} {} {indices} to <{} x i64>", iv.ir, iv.lanes()))
        };
        let limit = self.splat(&Vector { ir: format!("<{} x i64>", iv.lanes()), lane: "i64".to_string(), shape: iv.shape }, len);
        // Unsigned against the length: a negative index is a huge one, as `(uint64_t)x` makes it.
        let bad = self.value(format!("icmp uge <{} x i64> {as_words}, {limit}", iv.lanes()));
        Ok((wide, bad, iv))
    }

    /// `[SIMD-1]` — a `std.simd` operation.
    pub(crate) fn simd(&mut self, op: &SimdOp, vector: Ty, args: &[Operand], dest: &Place, span: ember_span::Span) -> Result<()> {
        let v = vector_of(self.m, vector)?;
        match op {
            SimdOp::Splat | SimdOp::MaskSplat => {
                let scalar = self.lane_scalar(&args[0], &v)?;
                let value = self.splat(&v, &scalar);
                self.store_vector(dest, &v, &value)
            }
            SimdOp::Load { checked } | SimdOp::Store { checked } => {
                let view = if matches!(op, SimdOp::Load { .. }) { &args[0] } else { &args[1] };
                let (words, align) = self.view_words(view)?;
                let pointer = self.load(Sc::Ptr, &words, align.min(8));
                let len_at = self.gep(&words, 8);
                let len = self.load(Sc::I(64), &len_at, align.min(8));
                if *checked {
                    let short = self.value(format!("icmp ult i64 {len}, {}", v.lanes()));
                    let (fail, ok) = (self.label(), self.label());
                    self.emit(format!("br i1 {short}, label %{fail}, label %{ok}"));
                    self.start(&fail);
                    let loc = self.loc(span);
                    let word = |len: &str| Val::Imm(Sc::I(64), len.to_string());
                    self.call_rt("panic_lane_bounds", vec![word(&len), word(&len), word(&len), loc], None);
                    self.start(&ok);
                }
                let lane_align = v.shape.bits / 8;
                if matches!(op, SimdOp::Load { .. }) {
                    let value = self.value(format!("load {}, ptr {pointer}, align {lane_align}", v.ir));
                    return self.store_vector(dest, &v, &value);
                }
                let value = self.vector(&args[0], &v)?;
                self.emit(format!("store {} {value}, ptr {pointer}, align {lane_align}", v.ir));
                Ok(())
            }
            SimdOp::Gather { checked, index } | SimdOp::Scatter { checked, index } => {
                let gather = matches!(op, SimdOp::Gather { .. });
                let (view, indices) = if gather { (&args[0], &args[1]) } else { (&args[1], &args[2]) };
                let (words, align) = self.view_words(view)?;
                let pointer = self.load(Sc::Ptr, &words, align.min(8));
                let len_at = self.gep(&words, 8);
                let len = self.load(Sc::I(64), &len_at, align.min(8));
                let (lanes, bad, iv) = self.index_lanes(indices, *index, &len)?;
                if *checked {
                    self.check_lanes(&bad, &iv, &lanes, &len, span);
                }
                let lane_align = v.shape.bits / 8;
                if gather {
                    let mut value = "poison".to_string();
                    for (at, index) in lanes.iter().enumerate() {
                        let element = self.value(format!("getelementptr {}, ptr {pointer}, i64 {index}", v.lane));
                        let loaded = self.value(format!("load {}, ptr {element}, align {lane_align}", v.lane));
                        value = self.value(format!("insertelement {} {value}, {} {loaded}, i64 {at}", v.ir, v.lane));
                    }
                    return self.store_vector(dest, &v, &value);
                }
                // In lane order: a later lane's store wins where two indices are one.
                let value = self.vector(&args[0], &v)?;
                for (at, index) in lanes.iter().enumerate() {
                    let element = self.value(format!("getelementptr {}, ptr {pointer}, i64 {index}", v.lane));
                    let lane = self.lane(&value, &v, at as u32);
                    self.emit(format!("store {} {lane}, ptr {element}, align {lane_align}", v.lane));
                }
                Ok(())
            }
            SimdOp::Binary { op: binary, policy } => {
                let (a, b) = (self.vector(&args[0], &v)?, self.vector(&args[1], &v)?);
                let value = match (binary, v.float()) {
                    (BinOp::BitAnd, _) => self.value(format!("and {} {a}, {b}", v.ir)),
                    (BinOp::BitOr, _) => self.value(format!("or {} {a}, {b}", v.ir)),
                    (BinOp::BitXor, _) => self.value(format!("xor {} {a}, {b}", v.ir)),
                    (BinOp::Add, true) => self.value(format!("fadd {} {a}, {b}", v.ir)),
                    (BinOp::Sub, true) => self.value(format!("fsub {} {a}, {b}", v.ir)),
                    (BinOp::Mul, true) => self.value(format!("fmul {} {a}, {b}", v.ir)),
                    (BinOp::Div, true) => self.value(format!("fdiv {} {a}, {b}", v.ir)),
                    (op, false) => self.int_arith(*op, *policy, &v, &a, &b, span)?,
                    _ => return unsupported("a vector operator"),
                };
                self.store_vector(dest, &v, &value)
            }
            SimdOp::Neg { policy } => {
                let a = self.vector(&args[0], &v)?;
                let value = if v.float() {
                    self.value(format!("fneg {} {a}", v.ir))
                } else {
                    self.int_arith(BinOp::Sub, *policy, &v, "zeroinitializer", &a, span)?
                };
                self.store_vector(dest, &v, &value)
            }
            SimdOp::Compare { op: compare } => {
                let (a, b) = (self.vector(&args[0], &v)?, self.vector(&args[1], &v)?);
                let cond = match (compare, v.float(), v.signed()) {
                    (BinOp::Eq, true, _) => "fcmp oeq",
                    (BinOp::Ne, true, _) => "fcmp une",
                    (BinOp::Lt, true, _) => "fcmp olt",
                    (BinOp::Le, true, _) => "fcmp ole",
                    (BinOp::Gt, true, _) => "fcmp ogt",
                    (_, true, _) => "fcmp oge",
                    (BinOp::Eq, false, _) => "icmp eq",
                    (BinOp::Ne, false, _) => "icmp ne",
                    (BinOp::Lt, false, true) => "icmp slt",
                    (BinOp::Lt, false, false) => "icmp ult",
                    (BinOp::Le, false, true) => "icmp sle",
                    (BinOp::Le, false, false) => "icmp ule",
                    (BinOp::Gt, false, true) => "icmp sgt",
                    (BinOp::Gt, false, false) => "icmp ugt",
                    (_, false, true) => "icmp sge",
                    (_, false, false) => "icmp uge",
                };
                let holds = self.value(format!("{cond} {} {a}, {b}", v.ir));
                // A mask lane is all ones where the comparison holds.
                let mask = vector_of(self.m, self.place_ty(dest))?;
                let value = self.value(format!("sext {} {holds} to {}", v.bools(), mask.ir));
                self.store_vector(dest, &mask, &value)
            }
            SimdOp::ReduceAdd { policy } => {
                let a = self.vector(&args[0], &v)?;
                let value = if v.float() {
                    // Ordered: lane 0, then each next lane (an LLVM reduction without `reassoc`).
                    let name = format!("llvm.vector.reduce.fadd.{}", v.suffix());
                    let f = self.intrinsic(&name, &v.lane.clone(), &[&v.lane.clone(), &v.ir.clone()]);
                    // `-0.0 + x` is `x` for every `x`, a negative zero too.
                    self.value(format!("call {} {f}({} -0.0, {} {a})", v.lane, v.lane, v.ir))
                } else {
                    match policy {
                        OverflowPolicy::Wrap => {
                            let name = format!("llvm.vector.reduce.add.{}", v.suffix());
                            let f = self.intrinsic(&name, &v.lane.clone(), &[&v.ir.clone()]);
                            self.value(format!("call {} {f}({} {a})", v.lane, v.ir))
                        }
                        OverflowPolicy::Saturate | OverflowPolicy::Panic => {
                            let sign = if v.signed() { "s" } else { "u" };
                            let lane = v.lane.clone();
                            let mut sum = self.lane(&a, &v, 0);
                            let mut overflow = "false".to_string();
                            for index in 1..v.lanes() {
                                let next = self.lane(&a, &v, index);
                                if *policy == OverflowPolicy::Saturate {
                                    let name = format!("llvm.{sign}add.sat.{lane}");
                                    let f = self.intrinsic(&name, &lane, &[&lane, &lane]);
                                    sum = self.value(format!("call {lane} {f}({lane} {sum}, {lane} {next})"));
                                } else {
                                    let pair = format!("{{ {lane}, i1 }}");
                                    let name = format!("llvm.{sign}add.with.overflow.{lane}");
                                    let f = self.intrinsic(&name, &pair, &[&lane, &lane]);
                                    let both = self.value(format!("call {pair} {f}({lane} {sum}, {lane} {next})"));
                                    sum = self.value(format!("extractvalue {pair} {both}, 0"));
                                    let flag = self.value(format!("extractvalue {pair} {both}, 1"));
                                    overflow = self.value(format!("or i1 {overflow}, {flag}"));
                                }
                            }
                            if *policy == OverflowPolicy::Panic {
                                self.panic_overflow_if(&overflow, "+", span);
                            }
                            sum
                        }
                    }
                };
                let sc = if v.float() { if v.shape.bits == 32 { Sc::F32 } else { Sc::F64 } } else { Sc::I(v.shape.bits) };
                self.store_result(dest, &value, sc, v.signed())
            }
            SimdOp::ReduceMin | SimdOp::ReduceMax => {
                let a = self.vector(&args[0], &v)?;
                let max = matches!(op, SimdOp::ReduceMax);
                let compare = match (v.float(), max, v.signed()) {
                    (true, false, _) => "fcmp olt",
                    (true, true, _) => "fcmp ogt",
                    (false, false, true) => "icmp slt",
                    (false, false, false) => "icmp ult",
                    (false, true, true) => "icmp sgt",
                    (false, true, false) => "icmp ugt",
                };
                // Lane 0, then each lane that is smaller (larger): NaN never replaces.
                let mut best = self.lane(&a, &v, 0);
                for index in 1..v.lanes() {
                    let next = self.lane(&a, &v, index);
                    let better = self.value(format!("{compare} {} {next}, {best}", v.lane));
                    best = self.value(format!("select i1 {better}, {} {next}, {} {best}", v.lane, v.lane));
                }
                let sc = if v.float() { if v.shape.bits == 32 { Sc::F32 } else { Sc::F64 } } else { Sc::I(v.shape.bits) };
                self.store_result(dest, &best, sc, v.signed())
            }
            SimdOp::Shuffle => {
                let a = self.vector(&args[0], &v)?;
                let (lanes_at, align) = match self.operand(&args[1])? {
                    Val::Mem(at, align) => (at, align),
                    _ => return unsupported("a shuffle whose lanes are not an array"),
                };
                let mut value = "poison".to_string();
                for index in 0..v.lanes() {
                    let at = self.gep(&lanes_at, 8 * u64::from(index));
                    let from = self.load(Sc::I(64), &at, align.min(8));
                    let lane = self.value(format!("extractelement {} {a}, i64 {from}", v.ir));
                    value = self.value(format!("insertelement {} {value}, {} {lane}, i64 {index}", v.ir, v.lane));
                }
                self.store_vector(dest, &v, &value)
            }
            SimdOp::Fma => {
                let (a, b, c) = (self.vector(&args[0], &v)?, self.vector(&args[1], &v)?, self.vector(&args[2], &v)?);
                let name = format!("llvm.fma.{}", v.suffix());
                let ir = v.ir.clone();
                let f = self.intrinsic(&name, &ir, &[&ir, &ir, &ir]);
                let value = self.value(format!("call {ir} {f}({ir} {a}, {ir} {b}, {ir} {c})"));
                self.store_vector(dest, &v, &value)
            }
            SimdOp::Min | SimdOp::Max => {
                let (a, b) = (self.vector(&args[0], &v)?, self.vector(&args[1], &v)?);
                let value = self.lane_min_max(&v, &a, &b, matches!(op, SimdOp::Max));
                self.store_vector(dest, &v, &value)
            }
            SimdOp::Abs { policy } => {
                let a = self.vector(&args[0], &v)?;
                let value = if v.float() {
                    // `a < 0 ? -a : a + 0`: a negative zero made positive, NaN kept as it is.
                    let negative = self.value(format!("fcmp olt {} {a}, zeroinitializer", v.ir));
                    let negated = self.value(format!("fneg {} {a}", v.ir));
                    let kept = self.value(format!("fadd {} {a}, zeroinitializer", v.ir));
                    self.value(format!("select {} {negative}, {} {negated}, {} {kept}", v.bools(), v.ir, v.ir))
                } else {
                    // `max(a, 0 - min(a, 0))` checked; `max(a, 0 - a)` wrapping or saturating.
                    let negated = match policy {
                        OverflowPolicy::Panic => {
                            let low = self.lane_min_max(&v, &a, "zeroinitializer", false);
                            self.int_arith(BinOp::Sub, *policy, &v, "zeroinitializer", &low, span)?
                        }
                        policy => self.int_arith(BinOp::Sub, *policy, &v, "zeroinitializer", &a, span)?,
                    };
                    self.lane_min_max(&v, &a, &negated, true)
                };
                self.store_vector(dest, &v, &value)
            }
            SimdOp::Sqrt | SimdOp::RsqrtApprox | SimdOp::RcpApprox => {
                let a = self.vector(&args[0], &v)?;
                let ir = v.ir.clone();
                let one = self.splat(&v, "1.0");
                let value = match op {
                    SimdOp::Sqrt => {
                        let name = format!("llvm.sqrt.{}", v.suffix());
                        let f = self.intrinsic(&name, &ir, &[&ir]);
                        self.value(format!("call {ir} {f}({ir} {a})"))
                    }
                    // `f32` lanes: SSE's 12-bit estimates, four lanes at a time, as `rsqrtps` and
                    // `rcpps` give them to the C backend on x86-64.
                    _ if v.shape.bits == 32 => {
                        let name = if matches!(op, SimdOp::RsqrtApprox) { "llvm.x86.sse.rsqrt.ps" } else { "llvm.x86.sse.rcp.ps" };
                        let f = self.intrinsic(name, "<4 x float>", &["<4 x float>"]);
                        let mut value = "poison".to_string();
                        for chunk in 0..v.lanes() / 4 {
                            let picks: Vec<String> = (0..4).map(|lane| format!("i32 {}", chunk * 4 + lane)).collect();
                            let part = self.value(format!("shufflevector {ir} {a}, {ir} poison, <4 x i32> <{}>", picks.join(", ")));
                            let estimate = self.value(format!("call <4 x float> {f}(<4 x float> {part})"));
                            for lane in 0..4 {
                                let element = self.value(format!("extractelement <4 x float> {estimate}, i64 {lane}"));
                                value = self.value(format!("insertelement {ir} {value}, float {element}, i64 {}", chunk * 4 + lane));
                            }
                        }
                        value
                    }
                    // `f64` lanes: exactly, `1 / sqrt(x)` and `1 / x`.
                    SimdOp::RsqrtApprox => {
                        let name = format!("llvm.sqrt.{}", v.suffix());
                        let f = self.intrinsic(&name, &ir, &[&ir]);
                        let root = self.value(format!("call {ir} {f}({ir} {a})"));
                        self.value(format!("fdiv {ir} {one}, {root}"))
                    }
                    _ => self.value(format!("fdiv {ir} {one}, {a}")),
                };
                self.store_vector(dest, &v, &value)
            }
            SimdOp::Any | SimdOp::All => {
                let m = self.vector(&args[0], &v)?;
                let set = self.value(format!("icmp ne {} {m}, zeroinitializer", v.ir));
                let which = if matches!(op, SimdOp::Any) { "or" } else { "and" };
                let name = format!("llvm.vector.reduce.{which}.v{}i1", v.lanes());
                let bools = v.bools();
                let f = self.intrinsic(&name, "i1", &[&bools]);
                let value = self.value(format!("call i1 {f}({bools} {set})"));
                self.store_result(dest, &value, Sc::Bool, false)
            }
            // `m.select(a, b)`, named for `a`'s vector type.
            SimdOp::Select => {
                let mask_ty = self.operand_ty(&args[0]).ok_or_else(|| crate::Unsupported("a select of an untyped mask".to_string()))?;
                let mask = vector_of(self.m, mask_ty)?;
                let m = self.vector(&args[0], &mask)?;
                let (a, b) = (self.vector(&args[1], &v)?, self.vector(&args[2], &v)?);
                let set = self.value(format!("icmp ne {} {m}, zeroinitializer", mask.ir));
                let value = self.value(format!("select {} {set}, {} {a}, {} {b}", v.bools(), v.ir, v.ir));
                self.store_vector(dest, &v, &value)
            }
        }
    }
}
