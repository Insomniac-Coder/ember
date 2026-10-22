/* The C side of FFI-11/accept_out_parameters_beside_a_contracted_result.em: objects made with an
   `out` parameter beside them. */
#ifndef TOKENS_H
#define TOKENS_H

typedef struct Token Token;

Token* token_make(int n, int* id);
Token* token_maybe(int n, int* id);
void token_free(Token* token);
int token_live(void);
#endif
