/* The C twin of text_loop_scripts.em (D-550 follow-up): the same strings from the same generator,
 * and the same reduction, hand-written. Input: `<mode> <count>` (modes as the probe's). */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#if defined(_MSC_VER) && !defined(__clang__)
#define NOINLINE __declspec(noinline)
#else
#define NOINLINE __attribute__((noinline))
#endif

static const char* ASCII_WORDS[] = {"the", "quick", "brown", "fox", "jumps", "over", "a", "lazy", "dog", "and", "runs", "far", "away", "from", "home", "today", "while", "it", "rains", "in", "spring", "I"};
static const char* SCRIPTS[][10] = {
    {"\xd0\xbf\xd1\x80\xd0\xb8\xd0\xb2\xd0\xb5\xd1\x82", "\xd0\xbc\xd0\xb8\xd1\x80", "\xd0\xb1\xd1\x8b\xd1\x81\xd1\x82\xd1\x80\xd1\x8b\xd0\xb9", "\xd0\xbb\xd0\xb8\xd1\x81\xd0\xb0", "\xd1\x81\xd0\xbe\xd0\xb1\xd0\xb0\xd0\xba\xd0\xb0", "\xd0\xb4\xd0\xbe\xd0\xbc", "\xd1\x80\xd0\xb5\xd0\xba\xd0\xb0", "\xd0\xbb\xd0\xb5\xd1\x81", "\xd0\xb4\xd1\x80\xd1\x83\xd0\xb3", "\xd0\xb2\xd0\xbe\xd0\xb4\xd0\xb0"},
    {"\xce\xb3\xce\xb5\xce\xb9\xce\xb1", "\xce\xba\xcf\x8c\xcf\x83\xce\xbc\xce\xbf\xcf\x82", "\xce\xb3\xcf\x81\xce\xae\xce\xb3\xce\xbf\xcf\x81\xce\xb7", "\xce\xb1\xce\xbb\xce\xb5\xcf\x80\xce\xbf\xcf\x8d", "\xcf\x83\xce\xba\xcf\x8d\xce\xbb\xce\xbf\xcf\x82", "\xcf\x83\xcf\x80\xce\xaf\xcf\x84\xce\xb9", "\xcf\x80\xce\xbf\xcf\x84\xce\xac\xce\xbc\xce\xb9", "\xce\xb4\xce\xac\xcf\x83\xce\xbf\xcf\x82", "\xcf\x86\xce\xaf\xce\xbb\xce\xbf\xcf\x82", "\xce\xbd\xce\xb5\xcf\x81\xcf\x8c"},
    {"\xd9\x85\xd8\xb1\xd8\xad\xd8\xa8\xd8\xa7", "\xd8\xb9\xd8\xa7\xd9\x84\xd9\x85", "\xd8\xb3\xd8\xb1\xd9\x8a\xd8\xb9", "\xd8\xab\xd8\xb9\xd9\x84\xd8\xa8", "\xd9\x83\xd9\x84\xd8\xa8", "\xd8\xa8\xd9\x8a\xd8\xaa", "\xd9\x86\xd9\x87\xd8\xb1", "\xd8\xba\xd8\xa7\xd8\xa8\xd8\xa9", "\xd8\xb5\xd8\xaf\xd9\x8a\xd9\x82", "\xd9\x85\xd8\xa7\xd8\xa1"},
    {"\xe0\xa4\xa8\xe0\xa4\xae\xe0\xa4\xb8\xe0\xa5\x8d\xe0\xa4\xa4\xe0\xa5\x87", "\xe0\xa4\xa6\xe0\xa5\x81\xe0\xa4\xa8\xe0\xa4\xbf\xe0\xa4\xaf\xe0\xa4\xbe", "\xe0\xa4\xa4\xe0\xa5\x87\xe0\xa4\x9c\xe0\xa4\xbc", "\xe0\xa4\xb2\xe0\xa5\x8b\xe0\xa4\xae\xe0\xa4\xa1\xe0\xa4\xbc\xe0\xa5\x80", "\xe0\xa4\x95\xe0\xa5\x81\xe0\xa4\xa4\xe0\xa5\x8d\xe0\xa4\xa4\xe0\xa4\xbe", "\xe0\xa4\x98\xe0\xa4\xb0", "\xe0\xa4\xa8\xe0\xa4\xa6\xe0\xa5\x80", "\xe0\xa4\x9c\xe0\xa4\x82\xe0\xa4\x97\xe0\xa4\xb2", "\xe0\xa4\xa6\xe0\xa5\x8b\xe0\xa4\xb8\xe0\xa5\x8d\xe0\xa4\xa4", "\xe0\xa4\xaa\xe0\xa4\xbe\xe0\xa4\xa8\xe0\xa5\x80"},
    {"\xe3\x81\x93\xe3\x82\x93\xe3\x81\xab\xe3\x81\xa1\xe3\x81\xaf", "\xe4\xb8\x96\xe7\x95\x8c", "\xe9\x80\x9f\xe3\x81\x84", "\xe7\x8b\x90", "\xe7\x8a\xac", "\xe5\xae\xb6", "\xe5\xb7\x9d", "\xe6\xa3\xae", "\xe5\x8f\x8b\xe9\x81\x94", "\xe6\xb0\xb4"},
    {"\xec\x95\x88\xeb\x85\x95\xed\x95\x98\xec\x84\xb8\xec\x9a\x94", "\xec\x84\xb8\xea\xb3\x84", "\xeb\xb9\xa0\xeb\xa5\xb8", "\xec\x97\xac\xec\x9a\xb0", "\xea\xb0\x9c", "\xec\xa7\x91", "\xea\xb0\x95", "\xec\x88\xb2", "\xec\xb9\x9c\xea\xb5\xac", "\xeb\xac\xbc"},
    {"\xf0\x9f\x98\x80", "\xf0\x9f\x8e\x89", "\xf0\x9f\x9a\x80", "\xf0\x9f\x90\xb6", "\xf0\x9f\x8c\xb2", "\xf0\x9f\x8f\xa0", "\xf0\x9f\x92\xa7", "\xf0\x9f\x94\xa5", "\xe2\xad\x90", "\xf0\x9f\x8d\x95"}
};
static const int SPACED[] = {1, 1, 1, 1, 0, 1, 1};
static const int64_t ASCII_SHARE[] = {0, 0, 0, 0, 0, 0, 5};

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
    int script = mode / 3, kind = mode % 3;
    int64_t seed = 12345, acc = 0;
    int count = kind == 0 ? 1 : 10000;
    Text* strings = (Text*)calloc((size_t)count, sizeof(Text));
    for (int k = 0; k < count; k++) {
        Text s = {0};
        next(&seed);
        int64_t words = 200000;
        if (kind == 1) words = (seed / 65536) % 4;
        else if (kind == 2) words = 4 + (seed / 65536) % 27;
        for (int64_t w = 0; w < words; w++) {
            if (w > 0 && SPACED[script]) push(&s, " ");
            int64_t which = (next(&seed) / 65536) % 10;
            next(&seed);
            if (which < ASCII_SHARE[script]) push(&s, ASCII_WORDS[(seed / 65536) % 22]);
            else push(&s, SCRIPTS[script][(seed / 65536) % 10]);
        }
        strings[k] = s;
    }
    for (long long round = 0; round < n; round++)
        for (int k = 0; k < count; k++)
            acc ^= reduce((const unsigned char*)strings[k].bytes, strings[k].len, round & 1023);
    printf("%lld\n", (long long)acc);
    return 0;
}
