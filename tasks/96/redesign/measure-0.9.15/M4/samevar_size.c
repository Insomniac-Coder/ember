/* samevar_size.c: the code-size side of verdict 88 (R4 exception 1, [BRW-11] exception 1): how
   much object code the copies with one name cost. Compiled only (`-c`), never run; the sizes are read
   with llvm-objdump -h from the object (build.txt). Each function is in its own section
   (-ffunction-sections), so each copy's bytes are listed separately.

   Part 1: `blend` from model_samevar.c, its normal copy and its one-name copy.
   Part 2: the worst case for a function with four `ref mut` spans of one type, `mix4(ref mut a, ref
   mut b, ref mut c, ref mut d)`: a program may name one whole variable for any group of them, so it
   may need one copy per way of grouping four names, the Bell number B(4) = 15 (the normal copy is the
   grouping where every name stands alone). Body:
       for k in 0..a.len():
           a[k] = a[k] * 0.5 + b[k]
           b[k] = b[k] * 0.25 + c[k]
           c[k] = c[k] * 0.75 + d[k]
           d[k] = d[k] * 0.5 + a[k]
   Each copy keeps the [OPT-2] entry test for the names it still has (merged names have one length).
   -DONLY_NORMAL leaves out every copy but the normal ones (the object a program with no such call
   has). The 15 copies of part 2 were written out by a short script from the body above. */
#include <stdint.h>
#define RESTRICT __restrict
#if defined(_MSC_VER) && !defined(__clang__)
#define NORETURN __declspec(noreturn)
#else
#define NORETURN __attribute__((noreturn))
#endif
NORETURN void e_panic(const char *m);   /* the runtime's panic, outside this object */

/* Part 1 */
void blend(float *RESTRICT dst, int64_t dst_len, float *RESTRICT src, int64_t src_len) {
    if (src_len >= dst_len) {
        for (int64_t k = 0; k < dst_len; k++) { dst[k] = dst[k] * 0.5f + src[k] * 0.25f; src[k] = src[k] * 0.75f + 0.125f; }
    } else {
        for (int64_t k = 0; k < dst_len; k++) {
            if (k >= src_len) e_panic("index out of bounds");
            dst[k] = dst[k] * 0.5f + src[k] * 0.25f; src[k] = src[k] * 0.75f + 0.125f;
        }
    }
}
#ifndef ONLY_NORMAL
void blend__one(float *RESTRICT s, int64_t s_len) {
    for (int64_t k = 0; k < s_len; k++) { s[k] = s[k] * 0.5f + s[k] * 0.25f; s[k] = s[k] * 0.75f + 0.125f; }
}
#endif

/* Part 2 */
#define MIX4_STEP(A, B, C, D, K)     A[K] = A[K] * 0.5f + B[K]; B[K] = B[K] * 0.25f + C[K]; C[K] = C[K] * 0.75f + D[K]; D[K] = D[K] * 0.5f + A[K];

/* grouping a | b | c | d */
void mix4(float *RESTRICT a, int64_t a_len, float *RESTRICT b, int64_t b_len, float *RESTRICT c, int64_t c_len, float *RESTRICT d, int64_t d_len) {
    if (b_len >= a_len && c_len >= a_len && d_len >= a_len) {
        for (int64_t k = 0; k < a_len; k++) { MIX4_STEP(a, b, c, d, k) }
    } else {
        for (int64_t k = 0; k < a_len; k++) {
            if (k >= b_len) e_panic("index out of bounds");
            if (k >= c_len) e_panic("index out of bounds");
            if (k >= d_len) e_panic("index out of bounds");
            MIX4_STEP(a, b, c, d, k)
        }
    }
}

#ifndef ONLY_NORMAL
/* grouping a | b | cd */
void mix4__a_b_cd(float *RESTRICT a, int64_t a_len, float *RESTRICT b, int64_t b_len, float *RESTRICT c, int64_t c_len) {
    if (b_len >= a_len && c_len >= a_len) {
        for (int64_t k = 0; k < a_len; k++) { MIX4_STEP(a, b, c, c, k) }
    } else {
        for (int64_t k = 0; k < a_len; k++) {
            if (k >= b_len) e_panic("index out of bounds");
            if (k >= c_len) e_panic("index out of bounds");
            MIX4_STEP(a, b, c, c, k)
        }
    }
}
#endif

#ifndef ONLY_NORMAL
/* grouping a | bc | d */
void mix4__a_bc_d(float *RESTRICT a, int64_t a_len, float *RESTRICT b, int64_t b_len, float *RESTRICT d, int64_t d_len) {
    if (b_len >= a_len && d_len >= a_len) {
        for (int64_t k = 0; k < a_len; k++) { MIX4_STEP(a, b, b, d, k) }
    } else {
        for (int64_t k = 0; k < a_len; k++) {
            if (k >= b_len) e_panic("index out of bounds");
            if (k >= d_len) e_panic("index out of bounds");
            MIX4_STEP(a, b, b, d, k)
        }
    }
}
#endif

#ifndef ONLY_NORMAL
/* grouping a | c | bd */
void mix4__a_bd_c(float *RESTRICT a, int64_t a_len, float *RESTRICT b, int64_t b_len, float *RESTRICT c, int64_t c_len) {
    if (b_len >= a_len && c_len >= a_len) {
        for (int64_t k = 0; k < a_len; k++) { MIX4_STEP(a, b, c, b, k) }
    } else {
        for (int64_t k = 0; k < a_len; k++) {
            if (k >= b_len) e_panic("index out of bounds");
            if (k >= c_len) e_panic("index out of bounds");
            MIX4_STEP(a, b, c, b, k)
        }
    }
}
#endif

#ifndef ONLY_NORMAL
/* grouping ab | c | d */
void mix4__ab_c_d(float *RESTRICT a, int64_t a_len, float *RESTRICT c, int64_t c_len, float *RESTRICT d, int64_t d_len) {
    if (c_len >= a_len && d_len >= a_len) {
        for (int64_t k = 0; k < a_len; k++) { MIX4_STEP(a, a, c, d, k) }
    } else {
        for (int64_t k = 0; k < a_len; k++) {
            if (k >= c_len) e_panic("index out of bounds");
            if (k >= d_len) e_panic("index out of bounds");
            MIX4_STEP(a, a, c, d, k)
        }
    }
}
#endif

#ifndef ONLY_NORMAL
/* grouping b | ac | d */
void mix4__ac_b_d(float *RESTRICT a, int64_t a_len, float *RESTRICT b, int64_t b_len, float *RESTRICT d, int64_t d_len) {
    if (b_len >= a_len && d_len >= a_len) {
        for (int64_t k = 0; k < a_len; k++) { MIX4_STEP(a, b, a, d, k) }
    } else {
        for (int64_t k = 0; k < a_len; k++) {
            if (k >= b_len) e_panic("index out of bounds");
            if (k >= d_len) e_panic("index out of bounds");
            MIX4_STEP(a, b, a, d, k)
        }
    }
}
#endif

#ifndef ONLY_NORMAL
/* grouping b | c | ad */
void mix4__ad_b_c(float *RESTRICT a, int64_t a_len, float *RESTRICT b, int64_t b_len, float *RESTRICT c, int64_t c_len) {
    if (b_len >= a_len && c_len >= a_len) {
        for (int64_t k = 0; k < a_len; k++) { MIX4_STEP(a, b, c, a, k) }
    } else {
        for (int64_t k = 0; k < a_len; k++) {
            if (k >= b_len) e_panic("index out of bounds");
            if (k >= c_len) e_panic("index out of bounds");
            MIX4_STEP(a, b, c, a, k)
        }
    }
}
#endif

#ifndef ONLY_NORMAL
/* grouping a | bcd */
void mix4__a_bcd(float *RESTRICT a, int64_t a_len, float *RESTRICT b, int64_t b_len) {
    if (b_len >= a_len) {
        for (int64_t k = 0; k < a_len; k++) { MIX4_STEP(a, b, b, b, k) }
    } else {
        for (int64_t k = 0; k < a_len; k++) {
            if (k >= b_len) e_panic("index out of bounds");
            MIX4_STEP(a, b, b, b, k)
        }
    }
}
#endif

#ifndef ONLY_NORMAL
/* grouping ab | cd */
void mix4__ab_cd(float *RESTRICT a, int64_t a_len, float *RESTRICT c, int64_t c_len) {
    if (c_len >= a_len) {
        for (int64_t k = 0; k < a_len; k++) { MIX4_STEP(a, a, c, c, k) }
    } else {
        for (int64_t k = 0; k < a_len; k++) {
            if (k >= c_len) e_panic("index out of bounds");
            MIX4_STEP(a, a, c, c, k)
        }
    }
}
#endif

#ifndef ONLY_NORMAL
/* grouping abc | d */
void mix4__abc_d(float *RESTRICT a, int64_t a_len, float *RESTRICT d, int64_t d_len) {
    if (d_len >= a_len) {
        for (int64_t k = 0; k < a_len; k++) { MIX4_STEP(a, a, a, d, k) }
    } else {
        for (int64_t k = 0; k < a_len; k++) {
            if (k >= d_len) e_panic("index out of bounds");
            MIX4_STEP(a, a, a, d, k)
        }
    }
}
#endif

#ifndef ONLY_NORMAL
/* grouping ac | bd */
void mix4__ac_bd(float *RESTRICT a, int64_t a_len, float *RESTRICT b, int64_t b_len) {
    if (b_len >= a_len) {
        for (int64_t k = 0; k < a_len; k++) { MIX4_STEP(a, b, a, b, k) }
    } else {
        for (int64_t k = 0; k < a_len; k++) {
            if (k >= b_len) e_panic("index out of bounds");
            MIX4_STEP(a, b, a, b, k)
        }
    }
}
#endif

#ifndef ONLY_NORMAL
/* grouping b | acd */
void mix4__acd_b(float *RESTRICT a, int64_t a_len, float *RESTRICT b, int64_t b_len) {
    if (b_len >= a_len) {
        for (int64_t k = 0; k < a_len; k++) { MIX4_STEP(a, b, a, a, k) }
    } else {
        for (int64_t k = 0; k < a_len; k++) {
            if (k >= b_len) e_panic("index out of bounds");
            MIX4_STEP(a, b, a, a, k)
        }
    }
}
#endif

#ifndef ONLY_NORMAL
/* grouping bc | ad */
void mix4__ad_bc(float *RESTRICT a, int64_t a_len, float *RESTRICT b, int64_t b_len) {
    if (b_len >= a_len) {
        for (int64_t k = 0; k < a_len; k++) { MIX4_STEP(a, b, b, a, k) }
    } else {
        for (int64_t k = 0; k < a_len; k++) {
            if (k >= b_len) e_panic("index out of bounds");
            MIX4_STEP(a, b, b, a, k)
        }
    }
}
#endif

#ifndef ONLY_NORMAL
/* grouping c | abd */
void mix4__abd_c(float *RESTRICT a, int64_t a_len, float *RESTRICT c, int64_t c_len) {
    if (c_len >= a_len) {
        for (int64_t k = 0; k < a_len; k++) { MIX4_STEP(a, a, c, a, k) }
    } else {
        for (int64_t k = 0; k < a_len; k++) {
            if (k >= c_len) e_panic("index out of bounds");
            MIX4_STEP(a, a, c, a, k)
        }
    }
}
#endif

#ifndef ONLY_NORMAL
/* grouping abcd */
void mix4__abcd(float *RESTRICT a, int64_t a_len) {
    for (int64_t k = 0; k < a_len; k++) { MIX4_STEP(a, a, a, a, k) }
}
#endif
