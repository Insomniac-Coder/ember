/* The C twin of text_loop_varied_text.em (D-550): the same strings from the same generator, and
 * the same reduction, hand-written. Input: `<mode> <count>` (modes as the probe's). */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#if defined(_MSC_VER) && !defined(__clang__)
#define NOINLINE __declspec(noinline)
#else
#define NOINLINE __attribute__((noinline))
#endif

static const char* ASCII_WORDS[] = {"the", "quick", "brown", "fox", "jumps", "over", "a", "lazy", "dog", "and", "runs",
                                    "far", "away", "from", "home", "today", "while", "it", "rains", "in", "spring", "I"};
static const char* LATIN_WORDS[] = {"h\xc3\xa9llo", "w\xc3\xb6rld", "caf\xc3\xa9", "na\xc3\xafve", "se\xc3\xb1or",
                                    "d\xc3\xa9j\xc3\xa0", "\xc3\x96" "fen", "zw\xc3\xb6lf", "cr\xc3\xa8me", "br\xc3\xbbl\xc3\xa9" "e"};
static const char* CJK_WORDS[] = {"\xe6\x95\x8f\xe6\x8d\xb7", "\xe6\xa3\x95\xe8\x89\xb2", "\xe7\x8b\x90\xe7\x8b\xb8",
                                  "\xe8\xb7\xb3\xe8\xbf\x87", "\xe6\x87\x92\xe7\x8b\x97", "\xe6\xa3\xae\xe6\x9e\x97",
                                  "\xe6\x9c\x8b\xe5\x8f\x8b", "\xe4\xbc\x91\xe6\x81\xaf", "\xe4\xb8\x80\xe8\xb5\xb7",
                                  "\xe7\x8e\xa9\xe8\x80\x8d"};
#define COUNT(a) ((int64_t)(sizeof(a) / sizeof((a)[0])))

typedef struct { char* bytes; size_t len, cap; } Text;

static void push(Text* t, const char* s) {
    size_t n = strlen(s);
    if (t->len + n > t->cap) {
        t->cap = (t->len + n) * 2 + 16;
        t->bytes = (char*)realloc(t->bytes, t->cap);
    }
    memcpy(t->bytes + t->len, s, n);
    t->len += n;
}

static int64_t next(int64_t* seed) { return *seed = (*seed * 1103515245 + 12345) % 2147483648; }
static const char* pick(int64_t seed, const char** words, int64_t n) { return words[(seed / 65536) % n]; }

NOINLINE static int64_t reduce(const unsigned char* p, size_t len, int64_t initial) {
    int64_t total = initial;
    for (size_t at = 0; at < len;) {
        unsigned char b = p[at];
        uint32_t c;
        size_t w;
        if (b < 0x80) { c = b; w = 1; }
        else if (b < 0xE0) { c = ((uint32_t)(b & 0x1F) << 6) | (p[at + 1] & 0x3F); w = 2; }
        else if (b < 0xF0) { c = ((uint32_t)(b & 0x0F) << 12) | ((uint32_t)(p[at + 1] & 0x3F) << 6) | (p[at + 2] & 0x3F); w = 3; }
        else { c = ((uint32_t)(b & 0x07) << 18) | ((uint32_t)(p[at + 1] & 0x3F) << 12) | ((uint32_t)(p[at + 2] & 0x3F) << 6) | (p[at + 3] & 0x3F); w = 4; }
        total += (int64_t)at ^ (int64_t)c;
        at += w;
    }
    return total;
}

int main(void) {
    int mode;
    long long n;
    if (scanf("%d %lld", &mode, &n) != 2) return 1;
    int64_t seed = 12345, acc = 0;
    if (mode >= 2) {
        Text* strings = (Text*)calloc(10000, sizeof(Text));
        for (int k = 0; k < 10000; k++) {
            Text s = {0};
            int64_t count = (next(&seed) / 65536) % 4;
            if (mode >= 3) count = 1 + (seed / 65536) % 3;
            if (mode >= 5) count = 4 + (seed / 65536) % 27;
            for (int64_t w = 0; w < count; w++) {
                if (w > 0) push(&s, " ");
                int64_t kind = (next(&seed) / 65536) % 10;
                next(&seed);
                if (kind < 7 || mode == 3 || mode == 6) push(&s, pick(seed, ASCII_WORDS, COUNT(ASCII_WORDS)));
                else if (kind < 9) push(&s, pick(seed, LATIN_WORDS, COUNT(LATIN_WORDS)));
                else push(&s, pick(seed, CJK_WORDS, COUNT(CJK_WORDS)));
            }
            strings[k] = s;
        }
        for (long long round = 0; round < n; round++)
            for (int k = 0; k < 10000; k++)
                acc ^= reduce((const unsigned char*)strings[k].bytes, strings[k].len, round & 1023);
    } else {
        Text text = {0};
        for (int k = 0; k < 200000; k++) {
            int64_t kind = (next(&seed) / 65536) % 100;
            next(&seed);
            if (mode == 0) {
                push(&text, kind < 95 ? pick(seed, ASCII_WORDS, COUNT(ASCII_WORDS)) : pick(seed, LATIN_WORDS, COUNT(LATIN_WORDS)));
                push(&text, " ");
            } else if (kind < 90) {
                push(&text, pick(seed, CJK_WORDS, COUNT(CJK_WORDS)));
            } else {
                push(&text, pick(seed, ASCII_WORDS, COUNT(ASCII_WORDS)));
                push(&text, " ");
            }
        }
        for (long long round = 0; round < n; round++)
            acc ^= reduce((const unsigned char*)text.bytes, text.len, round & 1023);
    }
    printf("%lld\n", (long long)acc);
    return 0;
}
