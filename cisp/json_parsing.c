#include <stdio.h>
#include <stdlib.h>

#include <editline/readline.h>
#include <histedit.h>

#include "mpc.h"

int main(int argc, char **argv) {
    printf("generating parsers");
    /* Create some parsers */
    mpc_parser_t *Json      = mpc_new("json");
    mpc_parser_t *Object    = mpc_new("object");
    mpc_parser_t *Members   = mpc_new("members");
    mpc_parser_t *Pair      = mpc_new("pair");
    mpc_parser_t *String    = mpc_new("string");
    mpc_parser_t *Value     = mpc_new("value");
    mpc_parser_t *Array     = mpc_new("array");
    mpc_parser_t *Elements  = mpc_new("elements");
    mpc_parser_t *Number    = mpc_new("number");
    mpc_parser_t *Int       = mpc_new("int");
    mpc_parser_t *Frac      = mpc_new("frac");
    mpc_parser_t *Exp       = mpc_new("exp");
    mpc_parser_t *Digit     = mpc_new("digit");
    mpc_parser_t *Digit19  = mpc_new("digit19");
    mpc_parser_t *Digits    = mpc_new("digits");
    mpc_parser_t *Digit19s = mpc_new("digit19s");

    /* Define the parsers with the following language */
    mpca_lang(MPCA_LANG_DEFAULT,
              " \
    json      : /^/ <object> /$/; \
    object    : '{' <members>* '}'; \
    members   : <pair>+; \
    pair      : <string> ':' <value>; \
    string    : /[a-zA-Z]+/ \
    value     : <string> | <number> | <object> | <array> | \"true\" | \"false\" | \"null\"; \
    array     : '[' elements+ ']'; \
    elements  : <value>+; \
    number    : <int> <frac>? <exp>?; \
    int       : '-'? <digit19s> | <digit>; \
    frac      : '.' digits; \
    exp       : 'e' <digits>; \
    digit     : '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'; \
    digit19  : '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'; \
    digits    : digit+; \
    digit19s : <digit19> <digits>; \
    ",
              Json, Object, Members, Pair, String, Value, Array, Elements, Number, Int, Frac, Exp, Digit, Digit19, Digits, Digit19s
              );

    /* Version and exit information */
    puts("Lispy version 0.0.0.0.1\n");
    puts("Press CTRL-C to exit\n");

    puts("Example expression: * 2 2 or * (+ 1 5) (* 1 3 7)");

    while (1) {
        char *input = readline("json >> ");

        add_history(input);

        /* Attempt to parse the input */
        mpc_result_t r;
        if (mpc_parse("<stdin>", input, Json, &r)) {
            /* On success print and delete the AST */
            mpc_ast_print(r.output);
            mpc_ast_delete(r.output);
        } else {
            /* Not parsed. Print error */
            mpc_err_print(r.error);
            mpc_err_delete(r.error);
        }

        /* readline does malloc */
        free(input);
    }

    /* Undefine and delete the parsers */
    mpc_cleanup(16, Json, Object, Members, Pair, String, Value, Array, Elements, Number, Int, Frac, Exp, Digit, Digit19, Digits, Digit19s, Number);

    return 0;
}
