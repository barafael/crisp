CFLAGS = -Wall -Wextra -std=c99 # Use variables for abstraction :)

.PHONY: all clean

pretty:
	uncrustify -c ./uncrustify_config --replace --no-backup *.c

all: *.c
	gcc -c $^ $(CFLAGS) 

prompt: prompt.c
	gcc prompt.c -lreadline -o prompt.o $(CFLAGS) 

parsing: parsing.c
	gcc -ggdb parsing.c mpc.c -lm -lreadline -o parsing.o $(CFLAGS) 

error_handling: error_handling.c
	gcc error_handling.c mpc.c -lm -lreadline -o error_handling.o $(CFLAGS) 

error_handling_orig: error_handling_orig.c
	gcc error_handling_orig.c mpc.c -lm -lreadline -o error_handling_orig.o $(CFLAGS) 

s_expressions: s_expressions.c
	gcc s_expressions.c -g mpc.c -lm -lreadline -o s_expressions.o $(CFLAGS)

s_expressions.orig: s_expressions.orig.c
	gcc s_expressions.orig.c -g mpc.c -lm -lreadline -o s_expressions.orig.o $(CFLAGS)

q_expressions: q_expressions.c
	gcc q_expressions.c -g mpc.c -lm -lreadline -o q_expressions.o $(CFLAGS)

variables: variables.c
	gcc variables.c -g mpc.c -lm -lreadline -o variables.o $(CFLAGS)

#main: main.o lib.o
#	gcc -o $@ $< $(CFLAGS) # $@ = target, $< = input

#%.o: %.c
#	gcc -c $^ $(CFLAGS) # $^ = $< but takes each file at once.

clean:
	rm *.o

