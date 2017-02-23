#include <stdio.h>
#include <stdlib.h>

#include <editline/readline.h>
#include <histedit.h>

/* Declare input buffer */
static char input[2048];

int main(int argc, char** argv) {
	/* version and exit information */
	puts("Lispy version 0.0.0.0.1\n");
	puts("Press CTRL-C to exit\n");

	while (1) {
		char* input = readline("cispy >> ");

		add_history(input);

		printf("%s\n", input);

		/* readline does malloc */
		free(input);
	}
	return 0;
}

