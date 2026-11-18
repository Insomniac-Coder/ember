#$ test: run-pass
#$ rules: TYP-36
#$ stdout: 1
## `[TYP-36]` - the table's cells that carry an implementation: a bound needing it is met.

class K:
    n: int

type Pct = u8 in 0 ..= 100

[T: needs Clone]
fn need_Clone(x: T):
    pass

[T: needs Copy]
fn need_Copy(x: T):
    pass

[T: needs Eq]
fn need_Eq(x: T):
    pass

[T: needs Ord]
fn need_Ord(x: T):
    pass

[T: needs Hash]
fn need_Hash(x: T):
    pass

[T: needs Default]
fn need_Default(x: T):
    pass

[T: needs Display]
fn need_Display(x: T):
    pass

[T: needs Debug]
fn need_Debug(x: T):
    pass

fn cell0(v: int):
    need_Clone(v)

fn cell1(v: int):
    need_Copy(v)

fn cell2(v: int):
    need_Eq(v)

fn cell3(v: int):
    need_Ord(v)

fn cell4(v: int):
    need_Hash(v)

fn cell5(v: int):
    need_Default(v)

fn cell6(v: int):
    need_Display(v)

fn cell7(v: int):
    need_Debug(v)

fn cell8(v: f64):
    need_Clone(v)

fn cell9(v: f64):
    need_Copy(v)

fn cell10(v: f64):
    need_Eq(v)

fn cell11(v: f64):
    need_Ord(v)

fn cell13(v: f64):
    need_Default(v)

fn cell14(v: f64):
    need_Display(v)

fn cell15(v: f64):
    need_Debug(v)

fn cell16(v: bool):
    need_Clone(v)

fn cell17(v: bool):
    need_Copy(v)

fn cell18(v: bool):
    need_Eq(v)

fn cell19(v: bool):
    need_Ord(v)

fn cell20(v: bool):
    need_Hash(v)

fn cell21(v: bool):
    need_Default(v)

fn cell22(v: bool):
    need_Display(v)

fn cell23(v: bool):
    need_Debug(v)

fn cell24(v: char):
    need_Clone(v)

fn cell25(v: char):
    need_Copy(v)

fn cell26(v: char):
    need_Eq(v)

fn cell27(v: char):
    need_Ord(v)

fn cell28(v: char):
    need_Hash(v)

fn cell29(v: char):
    need_Default(v)

fn cell30(v: char):
    need_Display(v)

fn cell31(v: char):
    need_Debug(v)

fn cell32(v: void):
    need_Clone(v)

fn cell33(v: void):
    need_Copy(v)

fn cell34(v: void):
    need_Eq(v)

fn cell35(v: void):
    need_Ord(v)

fn cell36(v: void):
    need_Hash(v)

fn cell37(v: void):
    need_Default(v)

fn cell39(v: void):
    need_Debug(v)

fn cell40(v: StrView):
    need_Clone(v)

fn cell41(v: StrView):
    need_Copy(v)

fn cell42(v: StrView):
    need_Eq(v)

fn cell43(v: StrView):
    need_Ord(v)

fn cell44(v: StrView):
    need_Hash(v)

fn cell45(v: StrView):
    need_Default(v)

fn cell46(v: StrView):
    need_Display(v)

fn cell47(v: StrView):
    need_Debug(v)

fn cell48(v: String):
    need_Clone(v)

fn cell50(v: String):
    need_Eq(v)

fn cell51(v: String):
    need_Ord(v)

fn cell52(v: String):
    need_Hash(v)

fn cell53(v: String):
    need_Default(v)

fn cell54(v: String):
    need_Display(v)

fn cell55(v: String):
    need_Debug(v)

fn cell56(v: (int, int)):
    need_Clone(v)

fn cell57(v: (int, int)):
    need_Copy(v)

fn cell58(v: (int, int)):
    need_Eq(v)

fn cell59(v: (int, int)):
    need_Ord(v)

fn cell60(v: (int, int)):
    need_Hash(v)

fn cell61(v: (int, int)):
    need_Display(v)

fn cell62(v: (int, int)):
    need_Debug(v)

fn cell63(v: [int; 2]):
    need_Clone(v)

fn cell64(v: [int; 2]):
    need_Copy(v)

fn cell65(v: [int; 2]):
    need_Eq(v)

fn cell66(v: [int; 2]):
    need_Ord(v)

fn cell67(v: [int; 2]):
    need_Hash(v)

fn cell68(v: [int; 2]):
    need_Display(v)

fn cell69(v: [int; 2]):
    need_Debug(v)

fn cell70(v: Option[int]):
    need_Clone(v)

fn cell71(v: Option[int]):
    need_Copy(v)

fn cell72(v: Option[int]):
    need_Eq(v)

fn cell73(v: Option[int]):
    need_Hash(v)

fn cell74(v: Option[int]):
    need_Default(v)

fn cell75(v: Option[int]):
    need_Display(v)

fn cell76(v: Option[int]):
    need_Debug(v)

fn cell77(v: Result[int, int]):
    need_Clone(v)

fn cell78(v: Result[int, int]):
    need_Copy(v)

fn cell79(v: Result[int, int]):
    need_Eq(v)

fn cell80(v: Result[int, int]):
    need_Hash(v)

fn cell82(v: Result[int, int]):
    need_Display(v)

fn cell83(v: Result[int, int]):
    need_Debug(v)

fn cell84(v: Array[int]):
    need_Clone(v)

fn cell86(v: Array[int]):
    need_Eq(v)

fn cell87(v: Array[int]):
    need_Ord(v)

fn cell88(v: Array[int]):
    need_Hash(v)

fn cell89(v: Array[int]):
    need_Default(v)

fn cell90(v: Array[int]):
    need_Display(v)

fn cell91(v: Array[int]):
    need_Debug(v)

fn cell92(v: Map[int, int]):
    need_Clone(v)

fn cell94(v: Map[int, int]):
    need_Eq(v)

fn cell97(v: Map[int, int]):
    need_Default(v)

fn cell98(v: Map[int, int]):
    need_Debug(v)

fn cell99(v: Set[int]):
    need_Clone(v)

fn cell101(v: Set[int]):
    need_Eq(v)

fn cell104(v: Set[int]):
    need_Default(v)

fn cell105(v: Set[int]):
    need_Debug(v)

fn cell106(v: K):
    need_Eq(v)

fn cell108(v: K):
    need_Hash(v)

fn cell111(v: K):
    need_Debug(v)

fn cell112(v: Pct):
    need_Clone(v)

fn cell113(v: Pct):
    need_Copy(v)

fn cell114(v: Pct):
    need_Eq(v)

fn cell115(v: Pct):
    need_Ord(v)

fn cell116(v: Pct):
    need_Hash(v)

fn cell118(v: Pct):
    need_Display(v)

fn cell119(v: Pct):
    need_Debug(v)

fn main():
    println(1)
