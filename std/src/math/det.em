## `std.math.det` — maths functions that give the same bits on every target
## (`[DET-4]`).
##
## `sin`, `cos`, `tan`, `exp`, `log`, `pow`, `atan2` and `sqrt`, for `f32` and
## `f64`. `std.math`'s functions are the platform's C library, whose last bits
## differ between systems, so a program that must compute the same thing on
## every machine (lock-step networking, replays, golden images) uses these
## instead (`[DET-2]`).
##
## They are fdlibm's algorithms, as FreeBSD's `msun` has them (Sun
## Microsystems, 1993), written with nothing but `+`, `-`, `*`, `/`, square
## roots, comparisons and conversions between floats and integers: each of
## those is exact or correctly rounded on every target, and Ember never fuses
## a multiply and an add (`[TYP-9]`), so every step rounds the same way
## everywhere. Where fdlibm reads or writes a float's bits, this code scales by
## exact powers of two, finds exponents by comparison, and splits a value into
## high and low parts arithmetically (Veltkamp and Dekker). A huge argument of
## `sin`, `cos` or `tan` is reduced exactly, by Payne and Hanek's method, with
## integer arithmetic over the bits of 2/π.
##
## Accuracy: within one unit in the last place, and within 1.3 for `atan2`
## (fdlibm's bounds). The `f32` functions compute in `f64` and round once.

## -- the types these functions take --------------------------------------------

## `f32` and `f64`, each with its deterministic functions.
interface Deterministic:
    fn det_sin(self) -> Self
    fn det_cos(self) -> Self
    fn det_tan(self) -> Self
    fn det_exp(self) -> Self
    fn det_log(self) -> Self
    fn det_pow(self, e: Self) -> Self
    fn det_atan2(self, x: Self) -> Self
    fn det_sqrt(self) -> Self

extend f64 implements Deterministic:
    fn det_sin(self) -> f64:
        return sin64(self)

    fn det_cos(self) -> f64:
        return cos64(self)

    fn det_tan(self) -> f64:
        return tan64(self)

    fn det_exp(self) -> f64:
        return exp64(self)

    fn det_log(self) -> f64:
        return log64(self)

    fn det_pow(self, e: f64) -> f64:
        return pow64(self, e)

    fn det_atan2(self, x: f64) -> f64:
        return atan2_64(self, x)

    fn det_sqrt(self) -> f64:
        return self.sqrt()

extend f32 implements Deterministic:
    fn det_sin(self) -> f32:
        return sin64(self as f64) as f32

    fn det_cos(self) -> f32:
        return cos64(self as f64) as f32

    fn det_tan(self) -> f32:
        return tan64(self as f64) as f32

    fn det_exp(self) -> f32:
        return exp64(self as f64) as f32

    fn det_log(self) -> f32:
        return log64(self as f64) as f32

    fn det_pow(self, e: f32) -> f32:
        return pow64(self as f64, e as f64) as f32

    fn det_atan2(self, x: f32) -> f32:
        return atan2_64(self as f64, x as f64) as f32

    fn det_sqrt(self) -> f32:
        return self.sqrt()

## The sine of `x`, in radians.
pub fn sin[T: Deterministic](x: T) -> T:
    return x.det_sin()

## The cosine of `x`, in radians.
pub fn cos[T: Deterministic](x: T) -> T:
    return x.det_cos()

## The tangent of `x`, in radians.
pub fn tan[T: Deterministic](x: T) -> T:
    return x.det_tan()

## e to the power `x`.
pub fn exp[T: Deterministic](x: T) -> T:
    return x.det_exp()

## The natural logarithm of `x`: `nan` below 0, `-inf` at 0.
pub fn log[T: Deterministic](x: T) -> T:
    return x.det_log()

## `x` to the power `e`, as C's `pow` defines each special case.
pub fn pow[T: Deterministic](x: T, e: T) -> T:
    return x.det_pow(e)

## The angle of the point `(x, y)` from the positive x axis, in `[-π, π]`.
pub fn atan2[T: Deterministic](y: T, x: T) -> T:
    return y.det_atan2(x)

## The square root, correctly rounded (IEEE 754 requires it of every target).
pub fn sqrt[T: Deterministic](x: T) -> T:
    return x.det_sqrt()

## -- exact powers of two, and exponents ------------------------------------------

## 2^n exactly, for -1074 ≤ n ≤ 1023: powers of two multiply exactly.
fn two_to(n: int) -> f64:
    base = 2.0
    if n < 0:
        base = 0.5
    m = abs(n)
    result = 1.0
    while m > 0:
        if m % 2 == 1:
            result *= base
        base *= base
        m //= 2
    return result

const P1023: f64 = 8.98846567431158e+307
const PM1022: f64 = 2.2250738585072014e-308
const PM969: f64 = PM1022 * 9007199254740992.0
const P54: f64 = 18014398509481984.0
const STEPS: [int; 10] = [512, 256, 128, 64, 32, 16, 8, 4, 2, 1]
const UP: [f64; 10] = [1.3407807929942597e+154, 1.157920892373162e+77, 3.402823669209385e+38, 18446744073709551616.0, 4294967296.0, 65536.0, 256.0, 16.0, 4.0, 2.0]
const DOWN: [f64; 10] = [7.458340731200207e-155, 8.636168555094445e-78, 2.938735877055719e-39, 5.421010862427522e-20, 2.3283064365386963e-10, 1.52587890625e-05, 0.00390625, 0.0625, 0.25, 0.5]

## `y * 2^n`, rounded once (musl's `scalbn`).
fn scalbn(y: f64, n: int) -> f64:
    v = y
    k = n
    if k > 1023:
        v *= P1023
        k -= 1023
        if k > 1023:
            v *= P1023
            k -= 1023
            if k > 1023:
                k = 1023
    elif k < -1022:
        # Keep the last scaling below -53 so a subnormal result rounds once.
        v *= PM969
        k += 969
        if k < -1022:
            v *= PM969
            k += 969
            if k < -1022:
                k = -1022
    return v * two_to(k)

## `(m, e)` with `x == m * 2^e` and `1 ≤ m < 2`, for a finite `x > 0`.
fn split_exponent(x: f64) -> (f64, int):
    v = x
    e = 0
    if v < PM1022:
        v *= P54
        e = -54
    for i in range(10):
        if v >= UP[i]:
            v *= DOWN[i]
            e += STEPS[i]
    for i in range(10):
        if v * UP[i] < 2.0:
            v *= UP[i]
            e -= STEPS[i]
    return (v, e)

## fdlibm's `(high word >> 20) & 0x7ff`: the biased exponent, 0 for 0 and
## for a subnormal.
fn biased_exponent(x: f64) -> int:
    if x == 0.0:
        return 0
    (_, e) = split_exponent(abs(x))
    return max(e + 1023, 0)

## `v`'s high part, with at most 26 significant bits (Veltkamp's split), so
## two such parts multiply exactly.
fn high_part(v: f64) -> f64:
    c = 134217729.0 * v
    return c - (c - v)

## `a * b` as `p + e` exactly (Dekker's product).
fn two_product(a: f64, b: f64) -> (f64, f64):
    p = a * b
    ah = high_part(a)
    al = a - ah
    bh = high_part(b)
    bl = b - bh
    e = ((ah * bh - p) + ah * bl + al * bh) + al * bl
    return (p, e)

## Whether `v`'s sign bit is set, `-0.0` included.
fn sign_bit(v: f64) -> bool:
    return (1.0).copysign(v) < 0.0

## -- exp ---------------------------------------------------------------------------

const O_THRESHOLD: f64 = 7.09782712893383973096e+02
const U_THRESHOLD: f64 = -7.45133219101941108420e+02
const LN2HI: f64 = 6.93147180369123816490e-01
const LN2LO: f64 = 1.90821492927058770002e-10
const INVLN2: f64 = 1.44269504088896338700e+00
const EP1: f64 = 1.66666666666666019037e-01
const EP2: f64 = -2.77777777770155933842e-03
const EP3: f64 = 6.61375632143793436117e-05
const EP4: f64 = -1.65339022054652515390e-06
const EP5: f64 = 4.13813679705723846039e-08

fn exp64(x: f64) -> f64:
    if x != x:
        return x + x
    ax = abs(x)
    if ax >= 709.7822265625:
        if ax == f64.INF:
            if x > 0.0:
                return x
            return 0.0
        if x > O_THRESHOLD:
            return f64.INF
        if x < U_THRESHOLD:
            return 0.0
    k = 0
    hi = 0.0
    lo = 0.0
    r = x
    if ax >= 0.3465735912322998:
        # |x| > ln2 / 2: x = k ln2 + r, |r| ≤ ln2 / 2.
        if ax < 1.0397205352783203:
            if x >= 0.0:
                hi = x - LN2HI
                lo = LN2LO
                k = 1
            else:
                hi = x + LN2HI
                lo = -LN2LO
                k = -1
        else:
            half = 0.5
            if x < 0.0:
                half = -0.5
            k = (INVLN2 * x + half) as int
            t = k as f64
            hi = x - t * LN2HI
            lo = t * LN2LO
        r = hi - lo
    elif ax < 3.725290298461914e-09:
        return 1.0 + x
    t = r * r
    c = r - t * (EP1 + t * (EP2 + t * (EP3 + t * (EP4 + t * EP5))))
    if k == 0:
        return 1.0 - ((r * c) / (c - 2.0) - r)
    y = 1.0 - ((lo - (r * c) / (2.0 - c)) - hi)
    if k >= -1021:
        if k == 1024:
            return y * 2.0 * P1023
        return y * two_to(k)
    return y * two_to(k + 1000) * two_to(-1000)

## -- log ---------------------------------------------------------------------------

const LN2_HI: f64 = 6.93147180369123816490e-01
const LN2_LO: f64 = 1.90821492927058770002e-10
const LG1: f64 = 6.666666666666735130e-01
const LG2: f64 = 3.999999999940941908e-01
const LG3: f64 = 2.857142874366239149e-01
const LG4: f64 = 2.222219843214978396e-01
const LG5: f64 = 1.818357216161805012e-01
const LG6: f64 = 1.531383769920937332e-01
const LG7: f64 = 1.479819860511658591e-01

fn log64(x: f64) -> f64:
    if x != x:
        return x + x
    if x <= 0.0:
        if x == 0.0:
            return -f64.INF
        return f64.NAN
    if x == f64.INF:
        return x
    (m, k) = split_exponent(x)
    # The mantissa's top 20 bits, as fdlibm reads them.
    hx = ((m - 1.0) * 1048576.0) as int
    if hx >= 0x6A09C:
        # m ≥ √2: log(m) is log(m / 2) + log 2.
        m *= 0.5
        k += 1
    f = m - 1.0
    if -9.5367431640625e-07 <= f and f < 9.5367431640625e-07:
        if f == 0.0:
            if k == 0:
                return 0.0
            dk = k as f64
            return dk * LN2_HI + dk * LN2_LO
        r = f * f * (0.5 - 0.33333333333333333 * f)
        if k == 0:
            return f - r
        dk = k as f64
        return dk * LN2_HI - ((r - dk * LN2_LO) - f)
    s = f / (2.0 + f)
    dk = k as f64
    z = s * s
    w = z * z
    t1 = w * (LG2 + w * (LG4 + w * LG6))
    t2 = z * (LG1 + w * (LG3 + w * (LG5 + w * LG7)))
    r = t2 + t1
    if 0x6147A <= hx and hx <= 0x6B851:
        hfsq = 0.5 * f * f
        if k == 0:
            return f - (hfsq - s * (hfsq + r))
        return dk * LN2_HI - ((hfsq - (s * (hfsq + r) + dk * LN2_LO)) - f)
    if k == 0:
        return f - s * (f - r)
    return dk * LN2_HI - ((s * (f - r) - dk * LN2_LO) - f)

## -- sin, cos, tan -------------------------------------------------------------------

const S1: f64 = -1.66666666666666324348e-01
const S2: f64 = 8.33333333332248946124e-03
const S3: f64 = -1.98412698298579493134e-04
const S4: f64 = 2.75573137070700676789e-06
const S5: f64 = -2.50507602534068634195e-08
const S6: f64 = 1.58969099521155010221e-10
const C1: f64 = 4.16666666666666019037e-02
const C2: f64 = -1.38888888888741095749e-03
const C3: f64 = 2.48015872894767294178e-05
const C4: f64 = -2.75573143513906633035e-07
const C5: f64 = 2.08757232129817482790e-09
const C6: f64 = -1.13596475577881948265e-11
const T: [f64; 13] = [
    3.33333333333334091986e-01,
    1.33333333333201242699e-01,
    5.39682539762260521377e-02,
    2.18694882948595424599e-02,
    8.86323982359930005737e-03,
    3.59207910759131235356e-03,
    1.45620945432529025516e-03,
    5.88041240820264096874e-04,
    2.46463134818469906812e-04,
    7.81794442939557092300e-05,
    7.14072491382608190305e-05,
    -1.85586374855275456654e-05,
    2.59073051863633712884e-05,
]
const PIO4: f64 = 7.85398163397448278999e-01
const PIO4LO: f64 = 3.06161699786838301793e-17

## sin(x + y) for |x + y| ≤ π/4, `y` the tail of `x` (0 when `iy` is 0).
fn k_sin(x: f64, y: f64, iy: int) -> f64:
    z = x * x
    w = z * z
    r = S2 + z * (S3 + z * S4) + z * w * (S5 + z * S6)
    v = z * x
    if iy == 0:
        return x + v * (S1 + z * r)
    return x - ((z * (0.5 * y - v * r) - y) - v * S1)

## cos(x + y) for |x + y| ≤ π/4.
fn k_cos(x: f64, y: f64) -> f64:
    z = x * x
    w = z * z
    r = z * (C1 + z * (C2 + z * C3)) + w * w * (C4 + z * (C5 + z * C6))
    hz = 0.5 * z
    w = 1.0 - hz
    return w + (((1.0 - w) - hz) + (z * r - x * y))

## tan(x + y) for |x + y| ≤ π/4, and -1/tan(x + y) when `iy` is -1.
fn k_tan(x0: f64, y0: f64, iy: int) -> f64:
    x = x0
    y = y0
    big = abs(x) >= 0.6743354797363281
    negative = x < 0.0
    if big:
        # Near π/4: tan(π/4 - x) is computed instead, and turned back.
        if negative:
            x = -x
            y = -y
        z = PIO4 - x
        w = PIO4LO - y
        x = z + w
        y = 0.0
    z = x * x
    w = z * z
    r = T[1] + w * (T[3] + w * (T[5] + w * (T[7] + w * (T[9] + w * T[11]))))
    v = z * (T[2] + w * (T[4] + w * (T[6] + w * (T[8] + w * (T[10] + w * T[12])))))
    s = z * x
    r = y + z * (s * (r + v) + y)
    r += T[0] * s
    w = x + r
    if big:
        v = iy as f64
        sign = 1.0
        if negative:
            sign = -1.0
        return sign * (v - 2.0 * (x - (w * w / (w + v) - r)))
    if iy == 1:
        return w
    # -1 / (x + r), accurately: the high parts of `w` and of its reciprocal
    # multiply exactly.
    hz = high_part(w)
    v = r - (hz - x)
    a = -1.0 / w
    ht = high_part(a)
    s = 1.0 + ht * hz
    return ht + a * (s + ht * v)

const INVPIO2: f64 = 6.36619772367581382433e-01
const PIO2_1: f64 = 1.57079632673412561417e+00
const PIO2_1T: f64 = 6.07710050650619224932e-11
const PIO2_2: f64 = 6.07710050630396597660e-11
const PIO2_2T: f64 = 2.02226624879595063154e-21
const PIO2_3: f64 = 2.02226624871116645580e-21
const PIO2_3T: f64 = 8.47842766036889956997e-32
const TOINT: f64 = 6755399441055744.0
const PIO2_HI: f64 = 1.5707963267948966
const PIO2_LO: f64 = 6.123233995736766e-17

## The bits of 2/π, 64 to a word, after a word of zeros: word `L` holds bits
## `64 L - 63` to `64 L` after the binary point.
const TWO_OVER_PI: [u64; 24] = [
    0x0, 0xa2f9836e4e441529, 0xfc2757d1f534ddc0, 0xdb6295993c439041, 0xfe5163abdebbc561,
    0xb7246e3a424dd2e0, 0x06492eea09d1921c, 0xfe1deb1cb129a73e, 0xe88235f52ebb4484,
    0xe99c7026b45f7e41, 0x3991d639835339f4, 0x9c845f8bbdf9283b, 0x1ff897ffde05980f,
    0xef2f118b5a0a6d1f, 0x6d367ecf27cb09b7, 0x4f463f669e5fea2d, 0x7527bac7ebe5f17b,
    0x3d0739f78a5292ea, 0x6bfb5fb11f8d5d08, 0x56033046fc7b6bab, 0xf0cfbc209af4361d,
    0xa9e391615ee61b08, 0x6599855f14a06840, 0x8dffd8804d732731,
]

## `(n, y0, y1)`: `x - n π/2 == y0 + y1`, with `|y0 + y1| ≤ π/4`, for a
## finite `|x| > π/4`. Below 2^20 π/2, π/2 in three parts is subtracted
## (Cody and Waite, fdlibm's "medium" case); above, `rem_pio2_large`.
fn rem_pio2(x: f64) -> (int, f64, f64):
    if abs(x) >= 1647099.0:
        return rem_pio2_large(x)
    fn_ = x * INVPIO2 + TOINT
    fn_ = fn_ - TOINT
    n = fn_ as int
    r = x - fn_ * PIO2_1
    w = fn_ * PIO2_1T
    y0 = r - w
    j = biased_exponent(x)
    if j - biased_exponent(y0) > 16:
        t = r
        w = fn_ * PIO2_2
        r = t - w
        w = fn_ * PIO2_2T - ((t - r) - w)
        y0 = r - w
        if j - biased_exponent(y0) > 49:
            t = r
            w = fn_ * PIO2_3
            r = t - w
            w = fn_ * PIO2_3T - ((t - r) - w)
            y0 = r - w
    y1 = (r - y0) - w
    return (n, y0, y1)

## Payne and Hanek's reduction: with `|x| = M 2^E` (`M` the 53-bit
## mantissa), the bits of 2/π before bit `E - 1` make whole multiples of 4
## and drop out, so `x 2/π mod 4` is `M` times the next 192 bits, mod
## 2^192: two bits of quadrant and 190 of fraction.
fn rem_pio2_large(x: f64) -> (int, f64, f64):
    m64: u64 = 0xffffffffffffffff
    (m, e) = split_exponent(abs(x))
    mant = (m * 4503599627370496.0) as u128
    g0 = e - 53 + 63
    q = g0 // 64
    r = g0 % 64
    l0 = TWO_OVER_PI[q]
    l1 = TWO_OVER_PI[q + 1]
    l2 = TWO_OVER_PI[q + 2]
    l3 = TWO_OVER_PI[q + 3]
    w2 = l0
    w1 = l1
    w0 = l2
    if r != 0:
        w2 = (l0 << r) | (l1 >> (64 - r))
        w1 = (l1 << r) | (l2 >> (64 - r))
        w0 = (l2 << r) | (l3 >> (64 - r))
    p = mant * (w0 as u128)
    p0 = (p & (m64 as u128)) as u64
    p = mant * (w1 as u128) + (p >> 64)
    p1 = (p & (m64 as u128)) as u64
    p = mant * (w2 as u128) + (p >> 64)
    p2 = (p & (m64 as u128)) as u64
    n = (p2 >> 62) as int
    fhi = (((p2 & 0x3fffffffffffffff) as u128) << 64) | (p1 as u128)
    flo = p0 as u128
    negative = false
    one: u128 = 1
    if fhi >= (one << 125):
        # The fraction is at least a half: round to the next quadrant.
        n += 1
        negative = true
        if flo == 0:
            fhi = (one << 126) - fhi
        else:
            fhi = (one << 126) - fhi - 1
            flo = (one << 64) - flo
    hi = fhi as f64
    rest = (fhi as i128) - (hi as i128)
    lo = (rest as f64) + (flo as f64) * 5.421010862427522e-20
    hi *= 1.1754943508222875e-38
    lo *= 1.1754943508222875e-38
    t = hi + lo
    lo = lo - (t - hi)
    hi = t
    (pp, err) = two_product(hi, PIO2_HI)
    err += hi * PIO2_LO + lo * PIO2_HI
    y0 = pp + err
    y1 = err - (y0 - pp)
    if negative:
        y0 = -y0
        y1 = -y1
    if x < 0.0:
        return (-n, -y0, -y1)
    return (n, y0, y1)

fn sin64(x: f64) -> f64:
    ax = abs(x)
    if ax < 0.7853984832763672:
        if ax < 1.4901161193847656e-08:
            return x
        return k_sin(x, 0.0, 0)
    if x != x or ax == f64.INF:
        return x - x
    (n, y0, y1) = rem_pio2(x)
    q = n % 4
    if q == 0:
        return k_sin(y0, y1, 1)
    if q == 1:
        return k_cos(y0, y1)
    if q == 2:
        return -k_sin(y0, y1, 1)
    return -k_cos(y0, y1)

fn cos64(x: f64) -> f64:
    ax = abs(x)
    if ax < 0.7853984832763672:
        if ax < 1.0536709282860102e-08:
            return 1.0
        return k_cos(x, 0.0)
    if x != x or ax == f64.INF:
        return x - x
    (n, y0, y1) = rem_pio2(x)
    q = n % 4
    if q == 0:
        return k_cos(y0, y1)
    if q == 1:
        return -k_sin(y0, y1, 1)
    if q == 2:
        return -k_cos(y0, y1)
    return k_sin(y0, y1, 1)

fn tan64(x: f64) -> f64:
    ax = abs(x)
    if ax < 0.7853984832763672:
        if ax < 7.450580596923828e-09:
            return x
        return k_tan(x, 0.0, 1)
    if x != x or ax == f64.INF:
        return x - x
    (n, y0, y1) = rem_pio2(x)
    return k_tan(y0, y1, 1 - (n % 2) * 2)

## -- atan, atan2 ---------------------------------------------------------------------

const ATANHI: [f64; 4] = [4.63647609000806093515e-01, 7.85398163397448278999e-01, 9.82793723247329054082e-01, 1.57079632679489655800e+00]
const ATANLO: [f64; 4] = [2.26987774529616870924e-17, 3.06161699786838301793e-17, 1.39033110312309984516e-17, 6.12323399573676603587e-17]
const AT: [f64; 11] = [
    3.33333333333329318027e-01,
    -1.99999999998764832476e-01,
    1.42857142725034663711e-01,
    -1.11111104054623557880e-01,
    9.09088713343650656196e-02,
    -7.69187620504482999495e-02,
    6.66107313738753120669e-02,
    -5.83357013379057348645e-02,
    4.97687799461593236017e-02,
    -3.65315727442169155270e-02,
    1.62858201153657823623e-02,
]

fn atan64(x0: f64) -> f64:
    x = x0
    ax = abs(x)
    if ax >= 7.378697629483821e+19:
        if x != x:
            return x + x
        if x > 0.0:
            return ATANHI[3] + ATANLO[3]
        return -ATANHI[3] - ATANLO[3]
    ident = -1
    negative = x < 0.0
    if ax < 0.4375:
        if ax < 7.450580596923828e-09:
            return x
    else:
        x = ax
        if ax < 1.1875:
            if ax < 0.6875:
                ident = 0
                x = (2.0 * x - 1.0) / (2.0 + x)
            else:
                ident = 1
                x = (x - 1.0) / (x + 1.0)
        elif ax < 2.4375:
            ident = 2
            x = (x - 1.5) / (1.0 + 1.5 * x)
        else:
            ident = 3
            x = -1.0 / x
    z = x * x
    w = z * z
    s1 = z * (AT[0] + w * (AT[2] + w * (AT[4] + w * (AT[6] + w * (AT[8] + w * AT[10])))))
    s2 = w * (AT[1] + w * (AT[3] + w * (AT[5] + w * (AT[7] + w * AT[9]))))
    if ident < 0:
        return x - x * (s1 + s2)
    z = ATANHI[ident] - ((x * (s1 + s2) - ATANLO[ident]) - x)
    if negative:
        return -z
    return z

const PI_O_4: f64 = 7.8539816339744827900e-01
const PI_O_2: f64 = 1.5707963267948965580e+00
const PI: f64 = 3.1415926535897931160e+00
const PI_LO: f64 = 1.2246467991473531772e-16

fn atan2_64(y: f64, x: f64) -> f64:
    if x != x or y != y:
        return x + y
    if x == 1.0:
        return atan64(y)
    # Two times the sign of `x` plus the sign of `y`, as bits.
    m = 0
    if sign_bit(y):
        m += 1
    if sign_bit(x):
        m += 2
    if y == 0.0:
        if m <= 1:
            return y
        if m == 2:
            return PI
        return -PI
    if x == 0.0:
        if y < 0.0:
            return -PI_O_2
        return PI_O_2
    if abs(x) == f64.INF:
        if abs(y) == f64.INF:
            if m == 0:
                return PI_O_4
            if m == 1:
                return -PI_O_4
            if m == 2:
                return 3.0 * PI_O_4
            return -3.0 * PI_O_4
        if m == 0:
            return 0.0
        if m == 1:
            return -0.0
        if m == 2:
            return PI
        return -PI
    if abs(y) == f64.INF:
        if y < 0.0:
            return -PI_O_2
        return PI_O_2
    k = biased_exponent(y) - biased_exponent(x)
    z = 0.0
    if k > 60:
        # |y / x| > 2^60: the angle is π/2 to within the last place.
        z = PI_O_2 + 0.5 * PI_LO
        m = m % 2
    elif x < 0.0 and k < -60:
        z = 0.0
    else:
        z = atan64(abs(y / x))
    if m == 0:
        return z
    if m == 1:
        return -z
    if m == 2:
        return PI - (z - PI_LO)
    return (z - PI_LO) - PI

## -- pow -------------------------------------------------------------------------------

const BP: [f64; 2] = [1.0, 1.5]
const DP_H: [f64; 2] = [0.0, 5.84962487220764160156e-01]
const DP_L: [f64; 2] = [0.0, 1.35003920212974897128e-08]
const TWO53: f64 = 9007199254740992.0
const THRD: f64 = 3.3333333333333331e-01
const L1: f64 = 5.99999999999994648725e-01
const L2: f64 = 4.28571428578550184252e-01
const L3: f64 = 3.33333329818377432918e-01
const L4: f64 = 2.72728123808534006489e-01
const L5: f64 = 2.30660745775561754067e-01
const L6: f64 = 2.06975017800338417784e-01
const POW_LG2: f64 = 6.93147180559945286227e-01
const POW_LG2_H: f64 = 6.93147182464599609375e-01
const POW_LG2_L: f64 = -1.90465429995776804525e-09
const OVT: f64 = 8.0085662595372944372e-17
const CP: f64 = 9.61796693925975554329e-01
const CP_H: f64 = 9.61796700954437255859e-01
const CP_L: f64 = -7.02846165095275826516e-09
const IVLN2: f64 = 1.44269504088896338700e+00
const IVLN2_H: f64 = 1.44269502162933349609e+00
const IVLN2_L: f64 = 1.92596299112661746887e-08

## `x^y`: log2(x) to about 70 bits as `t1 + t2`, times `y`, then 2 to that
## power; the special cases are C's.
fn pow64(x: f64, y: f64) -> f64:
    if y == 0.0:
        return 1.0
    if x == 1.0:
        return 1.0
    if x != x or y != y:
        return x + y
    ax = abs(x)
    ay = abs(y)
    negative = sign_bit(x)
    # For a negative `x`: whether `y` is an odd (1) or an even (2) integer.
    yisint = 0
    if negative:
        if ay >= TWO53:
            yisint = 2
        elif ay >= 1.0 and ay.floor() == ay:
            half = ay * 0.5
            if half.floor() != half:
                yisint = 1
            else:
                yisint = 2
    if ay == f64.INF:
        if ax == 1.0:
            return 1.0
        if ax > 1.0:
            if y > 0.0:
                return y
            return 0.0
        if y < 0.0:
            return -y
        return 0.0
    if y == 1.0:
        return x
    if y == -1.0:
        return 1.0 / x
    if y == 2.0:
        return x * x
    if y == 0.5 and not negative:
        return x.sqrt()
    if ax == f64.INF or ax == 0.0 or ax == 1.0:
        z = ax
        if y < 0.0:
            z = 1.0 / z
        if negative:
            if ax == 1.0 and yisint == 0:
                z = f64.NAN
            elif yisint == 1:
                z = -z
        return z
    if negative and yisint == 0:
        return f64.NAN
    s = 1.0
    if negative and yisint == 1:
        s = -1.0
    t1 = 0.0
    t2 = 0.0
    if ay >= 2147485696.0:
        if ay >= 1.8446761665895596e+19:
            if ax < 1.0:
                if y < 0.0:
                    return f64.INF
                return 0.0
            if y > 0.0:
                return f64.INF
            return 0.0
        if ax < 0.9999995231628418:
            if y < 0.0:
                return s * f64.INF
            return s * 0.0
        if ax >= 1.0000009536743164:
            if y > 0.0:
                return s * f64.INF
            return s * 0.0
        # |1 - x| ≤ 2^-20: log(x) by x - x²/2 + x³/3 - x⁴/4.
        t = ax - 1.0
        w = (t * t) * (0.5 - t * (THRD - t * 0.25))
        u = IVLN2_H * t
        v = t * IVLN2_L - w * IVLN2
        t1 = high_part(u + v)
        t2 = v - (t1 - u)
    else:
        (m, n) = split_exponent(ax)
        j = ((m - 1.0) * 1048576.0) as int
        k = 0
        if j <= 0x3988E:
            k = 0
        elif j < 0xBB67A:
            k = 1
        else:
            n += 1
            m *= 0.5
        # ss = s_h + s_l = (m - 1) / (m + 1), or (m - 1.5) / (m + 1.5).
        u = m - BP[k]
        v = 1.0 / (m + BP[k])
        ss = u * v
        s_h = high_part(ss)
        t_h = high_part(m + BP[k])
        t_l = m - (t_h - BP[k])
        s_l = v * ((u - s_h * t_h) - s_h * t_l)
        # log(m).
        s2 = ss * ss
        r = s2 * s2 * (L1 + s2 * (L2 + s2 * (L3 + s2 * (L4 + s2 * (L5 + s2 * L6)))))
        r += s_l * (s_h + ss)
        s2 = s_h * s_h
        t_h = high_part(3.0 + s2 + r)
        t_l = r - ((t_h - 3.0) - s2)
        u = s_h * t_h
        v = s_l * t_h + t_l * ss
        # 2 / (3 log 2) * (ss + …), plus n and log2(1.5).
        p_h = high_part(u + v)
        p_l = v - (p_h - u)
        z_h = CP_H * p_h
        z_l = CP_L * p_h + p_l * CP + DP_L[k]
        t = n as f64
        t1 = high_part(((z_h + z_l) + DP_H[k]) + t)
        t2 = z_l - (((t1 - t) - DP_H[k]) - z_h)
    # (y1 + y2) * (t1 + t2), `y1` the high part of `y`.
    y1 = high_part(y)
    p_l = (y - y1) * t1 + y * t2
    p_h = y1 * t1
    z = p_l + p_h
    if z >= 1024.0:
        if z > 1024.0 or p_l + OVT > z - p_h:
            return s * f64.INF
    elif z <= -1075.0:
        if z < -1075.0 or p_l <= z - p_h:
            return s * 0.0
    # 2^(p_h + p_l): the nearest integer `n`, and 2 to the rest.
    n = 0
    if abs(z) > 0.5:
        t = (abs(z) + 0.5).floor()
        if z < 0.0:
            t = -t
        n = t as int
        p_h -= t
    t = high_part(p_l + p_h)
    u = t * POW_LG2_H
    v = (p_l - (t - p_h)) * POW_LG2 + t * POW_LG2_L
    z = u + v
    w = v - (z - u)
    t = z * z
    t1 = z - t * (EP1 + t * (EP2 + t * (EP3 + t * (EP4 + t * EP5))))
    r = (z * t1) / (t1 - 2.0) - (w + z * w)
    z = 1.0 - (r - z)
    return s * scalbn(z, n)
