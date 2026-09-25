"""A model of `std.math.det` (`std/src/math/det.em`, `[DET-4]`, ADR-054).

The same operations in the same order, in Python, whose floats are IEEE
doubles and never fused: so this computes the bits the Ember module must, and
`tools/check_det.py` compares the two. Change both together. The algorithms
are fdlibm's (FreeBSD `msun`), with arithmetic where fdlibm reads a float's
bits. This model was measured against mpmath at 200 to 1400 bits: below one
unit in the last place for every function, `atan2` below 1.3.
"""

import math
import struct

INF = float("inf")
NAN = float("nan")


def f32(x):
    return struct.unpack("f", struct.pack("f", x))[0]


# -- exact powers of two, and the exponent ----------------------------------

def two_to(n):
    """2^n exactly, for -1074 <= n <= 1023."""
    base = 2.0 if n >= 0 else 0.5
    m = abs(n)
    result = 1.0
    while m > 0:
        if m % 2 == 1:
            result *= base
        base *= base
        m //= 2
    return result


P1023 = two_to(1023)
PM1022 = two_to(-1022)
P53 = two_to(53)
P54 = two_to(54)
STEPS = [512, 256, 128, 64, 32, 16, 8, 4, 2, 1]
UP = [two_to(s) for s in STEPS]
DOWN = [two_to(-s) for s in STEPS]


def scalbn(y, n):
    """y * 2^n with one rounding (musl's)."""
    if n > 1023:
        y *= P1023
        n -= 1023
        if n > 1023:
            y *= P1023
            n -= 1023
            if n > 1023:
                n = 1023
    elif n < -1022:
        y *= PM1022 * P53
        n += 1022 - 53
        if n < -1022:
            y *= PM1022 * P53
            n += 1022 - 53
            if n < -1022:
                n = -1022
    return y * two_to(n)


def split_exponent(x):
    """(m, e) with x = m * 2^e and 1 <= m < 2, for finite x > 0."""
    e = 0
    if x < PM1022:
        x *= P54
        e = -54
    for i in range(10):
        if x >= UP[i]:
            x *= DOWN[i]
            e += STEPS[i]
    for i in range(10):
        if x * UP[i] < 2.0:
            x *= UP[i]
            e -= STEPS[i]
    return x, e


def biased_exponent(x):
    """fdlibm's (high word >> 20) & 0x7ff: 0 for zero and subnormals."""
    if x == 0.0:
        return 0
    _, e = split_exponent(abs(x))
    return max(e + 1023, 0)


def split(a):
    """Veltkamp: a == hi + lo, hi with at most 26 significant bits."""
    c = 134217729.0 * a
    hi = c - (c - a)
    return hi, a - hi


def two_product(a, b):
    """Dekker: a * b == p + e exactly."""
    p = a * b
    ah, al = split(a)
    bh, bl = split(b)
    e = ((ah * bh - p) + ah * bl + al * bh) + al * bl
    return p, e


# -- exp ----------------------------------------------------------------------

O_THRESHOLD = 7.09782712893383973096e+02
U_THRESHOLD = -7.45133219101941108420e+02
LN2HI = 6.93147180369123816490e-01
LN2LO = 1.90821492927058770002e-10
INVLN2 = 1.44269504088896338700e+00
EP1 = 1.66666666666666019037e-01
EP2 = -2.77777777770155933842e-03
EP3 = 6.61375632143793436117e-05
EP4 = -1.65339022054652515390e-06
EP5 = 4.13813679705723846039e-08


def exp64(x):
    if x != x:
        return x + x
    ax = abs(x)
    if ax >= 709.7822265625:
        if ax == INF:
            return x if x > 0.0 else 0.0
        if x > O_THRESHOLD:
            return INF
        if x < U_THRESHOLD:
            return 0.0
    k = 0
    hi = 0.0
    lo = 0.0
    if ax >= 0.3465735912322998:
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
            k = int(INVLN2 * x + (0.5 if x >= 0.0 else -0.5))
            t = float(k)
            hi = x - t * LN2HI
            lo = t * LN2LO
        x = hi - lo
    elif ax < 3.725290298461914e-09:
        return 1.0 + x
    t = x * x
    c = x - t * (EP1 + t * (EP2 + t * (EP3 + t * (EP4 + t * EP5))))
    if k == 0:
        return 1.0 - ((x * c) / (c - 2.0) - x)
    y = 1.0 - ((lo - (x * c) / (2.0 - c)) - hi)
    if k >= -1021:
        if k == 1024:
            return y * 2.0 * P1023
        return y * two_to(k)
    return y * two_to(k + 1000) * two_to(-1000)


# -- log ----------------------------------------------------------------------

LN2_HI = 6.93147180369123816490e-01
LN2_LO = 1.90821492927058770002e-10
LG1 = 6.666666666666735130e-01
LG2 = 3.999999999940941908e-01
LG3 = 2.857142874366239149e-01
LG4 = 2.222219843214978396e-01
LG5 = 1.818357216161805012e-01
LG6 = 1.531383769920937332e-01
LG7 = 1.479819860511658591e-01


def log64(x):
    if x != x:
        return x + x
    if x <= 0.0:
        if x == 0.0:
            return -INF
        return NAN
    if x == INF:
        return x
    m, k = split_exponent(x)
    hx = int((m - 1.0) * 1048576.0)
    if hx >= 0x6A09C:
        m *= 0.5
        k += 1
    f = m - 1.0
    if -9.5367431640625e-07 <= f < 9.5367431640625e-07:
        if f == 0.0:
            if k == 0:
                return 0.0
            dk = float(k)
            return dk * LN2_HI + dk * LN2_LO
        r = f * f * (0.5 - 0.33333333333333333 * f)
        if k == 0:
            return f - r
        dk = float(k)
        return dk * LN2_HI - ((r - dk * LN2_LO) - f)
    s = f / (2.0 + f)
    dk = float(k)
    z = s * s
    w = z * z
    t1 = w * (LG2 + w * (LG4 + w * LG6))
    t2 = z * (LG1 + w * (LG3 + w * (LG5 + w * LG7)))
    r = t2 + t1
    if 0x6147A <= hx <= 0x6B851:
        hfsq = 0.5 * f * f
        if k == 0:
            return f - (hfsq - s * (hfsq + r))
        return dk * LN2_HI - ((hfsq - (s * (hfsq + r) + dk * LN2_LO)) - f)
    if k == 0:
        return f - s * (f - r)
    return dk * LN2_HI - ((s * (f - r) - dk * LN2_LO) - f)


# -- sin, cos, tan --------------------------------------------------------------

S1 = -1.66666666666666324348e-01
S2 = 8.33333333332248946124e-03
S3 = -1.98412698298579493134e-04
S4 = 2.75573137070700676789e-06
S5 = -2.50507602534068634195e-08
S6 = 1.58969099521155010221e-10
C1 = 4.16666666666666019037e-02
C2 = -1.38888888888741095749e-03
C3 = 2.48015872894767294178e-05
C4 = -2.75573143513906633035e-07
C5 = 2.08757232129817482790e-09
C6 = -1.13596475577881948265e-11
T = [
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
PIO4 = 7.85398163397448278999e-01
PIO4LO = 3.06161699786838301793e-17


def k_sin(x, y, iy):
    z = x * x
    w = z * z
    r = S2 + z * (S3 + z * S4) + z * w * (S5 + z * S6)
    v = z * x
    if iy == 0:
        return x + v * (S1 + z * r)
    return x - ((z * (0.5 * y - v * r) - y) - v * S1)


def k_cos(x, y):
    z = x * x
    w = z * z
    r = z * (C1 + z * (C2 + z * C3)) + w * w * (C4 + z * (C5 + z * C6))
    hz = 0.5 * z
    w = 1.0 - hz
    return w + (((1.0 - w) - hz) + (z * r - x * y))


def k_tan(x, y, iy):
    """tan(x + y) for |x + y| <= pi/4; -1/tan when iy is -1."""
    big = abs(x) >= 0.6743354797363281
    negative = x < 0.0
    if big:
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
        v = float(iy)
        sign = -1.0 if negative else 1.0
        return sign * (v - 2.0 * (x - (w * w / (w + v) - r)))
    if iy == 1:
        return w
    # -1 / (x + r), accurately: w's and its reciprocal's high parts multiply
    # exactly.
    z, _ = split(w)
    v = r - (z - x)
    a = -1.0 / w
    t, _ = split(a)
    s = 1.0 + t * z
    return t + a * (s + t * v)


INVPIO2 = 6.36619772367581382433e-01
PIO2_1 = 1.57079632673412561417e+00
PIO2_1T = 6.07710050650619224932e-11
PIO2_2 = 6.07710050630396597660e-11
PIO2_2T = 2.02226624879595063154e-21
PIO2_3 = 2.02226624871116645580e-21
PIO2_3T = 8.47842766036889956997e-32
TOINT = 6755399441055744.0
PIO2_HI = 1.5707963267948966
PIO2_LO = 6.123233995736766e-17
TWO_OVER_PI = [int(h, 16) for h in [
    '0x0', '0xa2f9836e4e441529', '0xfc2757d1f534ddc0', '0xdb6295993c439041', '0xfe5163abdebbc561',
    '0xb7246e3a424dd2e0', '0x6492eea09d1921c', '0xfe1deb1cb129a73e', '0xe88235f52ebb4484',
    '0xe99c7026b45f7e41', '0x3991d639835339f4', '0x9c845f8bbdf9283b', '0x1ff897ffde05980f',
    '0xef2f118b5a0a6d1f', '0x6d367ecf27cb09b7', '0x4f463f669e5fea2d', '0x7527bac7ebe5f17b',
    '0x3d0739f78a5292ea', '0x6bfb5fb11f8d5d08', '0x56033046fc7b6bab', '0xf0cfbc209af4361d',
    '0xa9e391615ee61b08', '0x6599855f14a06840', '0x8dffd8804d732731']]
M64 = (1 << 64) - 1


def rem_pio2(x):
    """(n, y0, y1): x - n * pi/2 == y0 + y1, |y0 + y1| <= pi/4, for finite
    |x| > pi/4."""
    ax = abs(x)
    if ax < 1647099.0:
        fn = x * INVPIO2 + TOINT
        fn = fn - TOINT
        n = int(fn)
        r = x - fn * PIO2_1
        w = fn * PIO2_1T
        y0 = r - w
        j = biased_exponent(x)
        if j - biased_exponent(y0) > 16:
            t = r
            w = fn * PIO2_2
            r = t - w
            w = fn * PIO2_2T - ((t - r) - w)
            y0 = r - w
            if j - biased_exponent(y0) > 49:
                t = r
                w = fn * PIO2_3
                r = t - w
                w = fn * PIO2_3T - ((t - r) - w)
                y0 = r - w
        y1 = (r - y0) - w
        return n, y0, y1
    return rem_pio2_large(x)


def rem_pio2_large(x):
    """Payne and Hanek's reduction: x = M * 2^E, and x * 2/pi mod 4 is
    M times a 192-bit window of 2/pi's bits, mod 2^192."""
    m, e = split_exponent(abs(x))
    mant = int(m * 4503599627370496.0)
    s = e - 52 - 1
    g0 = s + 63
    q = g0 // 64
    r = g0 % 64
    l0, l1, l2, l3 = TWO_OVER_PI[q], TWO_OVER_PI[q + 1], TWO_OVER_PI[q + 2], TWO_OVER_PI[q + 3]
    if r == 0:
        w2, w1, w0 = l0, l1, l2
    else:
        w2 = ((l0 << r) & M64) | (l1 >> (64 - r))
        w1 = ((l1 << r) & M64) | (l2 >> (64 - r))
        w0 = ((l2 << r) & M64) | (l3 >> (64 - r))
    p = mant * w0
    p0 = p & M64
    p = mant * w1 + (p >> 64)
    p1 = p & M64
    p = mant * w2 + (p >> 64)
    p2 = p & M64
    n = p2 >> 62
    fhi = ((p2 & ((1 << 62) - 1)) << 64) | p1
    flo = p0
    negative = False
    if fhi >= (1 << 125):
        n += 1
        negative = True
        if flo == 0:
            fhi = (1 << 126) - fhi
        else:
            fhi = (1 << 126) - fhi - 1
            flo = (1 << 64) - flo
    hi = float(fhi)
    rest = fhi - int(hi)
    lo = float(rest) + float(flo) * two_to(-64)
    hi *= two_to(-126)
    lo *= two_to(-126)
    t = hi + lo
    lo = lo - (t - hi)
    hi = t
    p, err = two_product(hi, PIO2_HI)
    err += hi * PIO2_LO + lo * PIO2_HI
    y0 = p + err
    y1 = err - (y0 - p)
    if negative:
        y0 = -y0
        y1 = -y1
    if x < 0.0:
        return -n, -y0, -y1
    return n, y0, y1


def sin64(x):
    ax = abs(x)
    if ax < 0.7853984832763672:
        if ax < 1.4901161193847656e-08:
            return x
        return k_sin(x, 0.0, 0)
    if x != x or ax == INF:
        return x - x
    n, y0, y1 = rem_pio2(x)
    q = n % 4
    if q == 0:
        return k_sin(y0, y1, 1)
    if q == 1:
        return k_cos(y0, y1)
    if q == 2:
        return -k_sin(y0, y1, 1)
    return -k_cos(y0, y1)


def cos64(x):
    ax = abs(x)
    if ax < 0.7853984832763672:
        if ax < 1.0536709282860102e-08:
            return 1.0
        return k_cos(x, 0.0)
    if x != x or ax == INF:
        return x - x
    n, y0, y1 = rem_pio2(x)
    q = n % 4
    if q == 0:
        return k_cos(y0, y1)
    if q == 1:
        return -k_sin(y0, y1, 1)
    if q == 2:
        return -k_cos(y0, y1)
    return k_sin(y0, y1, 1)


def tan64(x):
    ax = abs(x)
    if ax < 0.7853984832763672:
        if ax < 7.450580596923828e-09:
            return x
        return k_tan(x, 0.0, 1)
    if x != x or ax == INF:
        return x - x
    n, y0, y1 = rem_pio2(x)
    return k_tan(y0, y1, 1 - (n % 2) * 2)


# -- atan, atan2 ------------------------------------------------------------------

ATANHI = [4.63647609000806093515e-01, 7.85398163397448278999e-01, 9.82793723247329054082e-01, 1.57079632679489655800e+00]
ATANLO = [2.26987774529616870924e-17, 3.06161699786838301793e-17, 1.39033110312309984516e-17, 6.12323399573676603587e-17]
AT = [
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


def atan64(x):
    ax = abs(x)
    if ax >= 7.378697629483821e+19:
        if x != x:
            return x + x
        if x > 0.0:
            return ATANHI[3] + ATANLO[3]
        return -ATANHI[3] - ATANLO[3]
    if ax < 0.4375:
        if ax < 7.450580596923828e-09:
            return x
        ident = -1
    else:
        negative = x < 0.0
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
    return -z if negative else z


PI_O_4 = 7.8539816339744827900E-01
PI_O_2 = 1.5707963267948965580E+00
PI = 3.1415926535897931160E+00
PI_LO = 1.2246467991473531772E-16


def signbit(v):
    return math.copysign(1.0, v) < 0.0


def atan2_64(y, x):
    if x != x or y != y:
        return x + y
    if x == 1.0:
        return atan64(y)
    m = (1 if signbit(y) else 0) | (2 if signbit(x) else 0)
    if y == 0.0:
        if m <= 1:
            return y
        if m == 2:
            return PI
        return -PI
    if x == 0.0:
        return -PI_O_2 if y < 0.0 else PI_O_2
    if abs(x) == INF:
        if abs(y) == INF:
            return [PI_O_4, -PI_O_4, 3.0 * PI_O_4, -3.0 * PI_O_4][m]
        return [0.0, -0.0, PI, -PI][m]
    if abs(y) == INF:
        return -PI_O_2 if y < 0.0 else PI_O_2
    k = biased_exponent(y) - biased_exponent(x)
    if k > 60:
        z = PI_O_2 + 0.5 * PI_LO
        m &= 1
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


# -- pow ------------------------------------------------------------------------

BP = [1.0, 1.5]
DP_H = [0.0, 5.84962487220764160156e-01]
DP_L = [0.0, 1.35003920212974897128e-08]
TWO53 = 9007199254740992.0
THRD = 3.3333333333333331e-01
L1 = 5.99999999999994648725e-01
L2 = 4.28571428578550184252e-01
L3 = 3.33333329818377432918e-01
L4 = 2.72728123808534006489e-01
L5 = 2.30660745775561754067e-01
L6 = 2.06975017800338417784e-01
POW_LG2 = 6.93147180559945286227e-01
POW_LG2_H = 6.93147182464599609375e-01
POW_LG2_L = -1.90465429995776804525e-09
OVT = 8.0085662595372944372e-17
CP = 9.61796693925975554329e-01
CP_H = 9.61796700954437255859e-01
CP_L = -7.02846165095275826516e-09
IVLN2 = 1.44269504088896338700e+00
IVLN2_H = 1.44269502162933349609e+00
IVLN2_L = 1.92596299112661746887e-08


def hi26(v):
    return split(v)[0]


def pow64(x, y):
    if y == 0.0:
        return 1.0
    if x == 1.0:
        return 1.0
    if x != x or y != y:
        return x + y
    ax = abs(x)
    ay = abs(y)
    negative = signbit(x)
    # Whether y is an odd (1) or even (2) integer, for a negative x.
    yisint = 0
    if negative:
        if ay >= TWO53:
            yisint = 2
        elif ay >= 1.0 and math.floor(ay) == ay:
            half = ay * 0.5
            yisint = 1 if math.floor(half) != half else 2
    if ay == INF:
        if ax == 1.0:
            return 1.0
        if ax > 1.0:
            return y if y > 0.0 else 0.0
        return -y if y < 0.0 else 0.0
    if y == 1.0:
        return x
    if y == -1.0:
        return 1.0 / x if x != 0.0 else math.copysign(INF, x)
    if y == 2.0:
        return x * x
    if y == 0.5 and not negative:
        return math.sqrt(x)
    if ax == INF or ax == 0.0 or ax == 1.0:
        z = ax
        if y < 0.0:
            z = 1.0 / z if z != 0.0 else INF
        if negative:
            if ax == 1.0 and yisint == 0:
                z = NAN
            elif yisint == 1:
                z = -z
        return z
    if negative and yisint == 0:
        return NAN
    s = -1.0 if negative and yisint == 1 else 1.0
    if ay >= 2147485696.0:
        if ay >= 1.8446761665895596e+19:
            if ax < 1.0:
                return INF if y < 0.0 else 0.0
            return INF if y > 0.0 else 0.0
        if ax < 0.9999995231628418:
            return s * INF if y < 0.0 else s * 0.0
        if ax >= 1.0000009536743164:
            return s * INF if y > 0.0 else s * 0.0
        # |1 - x| <= 2^-20: log(x) by x - x^2/2 + x^3/3 - x^4/4.
        t = ax - 1.0
        w = (t * t) * (0.5 - t * (THRD - t * 0.25))
        u = IVLN2_H * t
        v = t * IVLN2_L - w * IVLN2
        t1 = hi26(u + v)
        t2 = v - (t1 - u)
    else:
        m, n = split_exponent(ax)
        j = int((m - 1.0) * 1048576.0)
        if j <= 0x3988E:
            k = 0
        elif j < 0xBB67A:
            k = 1
        else:
            k = 0
            n += 1
            m *= 0.5
        ax = m
        u = ax - BP[k]
        v = 1.0 / (ax + BP[k])
        ss = u * v
        s_h = hi26(ss)
        t_h = hi26(ax + BP[k])
        t_l = ax - (t_h - BP[k])
        s_l = v * ((u - s_h * t_h) - s_h * t_l)
        s2 = ss * ss
        r = s2 * s2 * (L1 + s2 * (L2 + s2 * (L3 + s2 * (L4 + s2 * (L5 + s2 * L6)))))
        r += s_l * (s_h + ss)
        s2 = s_h * s_h
        t_h = hi26(3.0 + s2 + r)
        t_l = r - ((t_h - 3.0) - s2)
        u = s_h * t_h
        v = s_l * t_h + t_l * ss
        p_h = hi26(u + v)
        p_l = v - (p_h - u)
        z_h = CP_H * p_h
        z_l = CP_L * p_h + p_l * CP + DP_L[k]
        t = float(n)
        t1 = hi26(((z_h + z_l) + DP_H[k]) + t)
        t2 = z_l - (((t1 - t) - DP_H[k]) - z_h)
    # (y1 + y2) * (t1 + t2), y1 the high part of y.
    y1 = hi26(y)
    p_l = (y - y1) * t1 + y * t2
    p_h = y1 * t1
    z = p_l + p_h
    if z >= 1024.0:
        if z > 1024.0 or p_l + OVT > z - p_h:
            return s * INF
    elif z <= -1075.0:
        if z < -1075.0 or p_l <= z - p_h:
            return s * 0.0
    # 2^(p_h + p_l): the nearest integer n, and the rest.
    n = 0
    if abs(z) > 0.5:
        t = math.floor(abs(z) + 0.5)
        if z < 0.0:
            t = -t
        n = int(t)
        p_h -= t
    t = hi26(p_l + p_h)
    u = t * POW_LG2_H
    v = (p_l - (t - p_h)) * POW_LG2 + t * POW_LG2_L
    z = u + v
    w = v - (z - u)
    t = z * z
    t1 = z - t * (EP1 + t * (EP2 + t * (EP3 + t * (EP4 + t * EP5))))
    r = (z * t1) / (t1 - 2.0) - (w + z * w)
    z = 1.0 - (r - z)
    return s * scalbn(z, n)
