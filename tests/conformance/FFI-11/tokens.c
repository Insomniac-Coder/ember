#include <stdlib.h>
#include "tokens.h"

struct Token { int n; };

static int live;

Token* token_make(int n, int* id) {
    Token* token = malloc(sizeof *token);
    token->n = n;
    *id = n * 10;
    live += 1;
    return token;
}

Token* token_maybe(int n, int* id) {
    *id = n;
    return n < 0 ? NULL : token_make(n, id);
}

void token_free(Token* token) {
    live -= 1;
    free(token);
}

int token_live(void) { return live; }
