CFLAGS = -Wall -Wextra -std=c99 # Use variables for abstraction :)

.PHONY: all clean

pretty:
	uncrustify -c ~/unc_config --replace --no-backup *.c

all: *.c
	gcc -c $^ $(CFLAGS) 

prompt: prompt.c
	gcc prompt.c -lreadline -o prompt.o $(CFLAGS) 

parsing: parsing.c
	gcc parsing.c mpc.c -lm -lreadline -o parsing.o $(CFLAGS) 

error_handling: error_handling.c
	gcc error_handling.c mpc.c -lm -lreadline -o error_handling.o $(CFLAGS) 

error_handling_orig: error_handling_orig.c
	gcc error_handling_orig.c mpc.c -lm -lreadline -o error_handling_orig.o $(CFLAGS) 
#main: main.o lib.o
#	gcc -o $@ $< $(CFLAGS) # $@ = target, $< = input

#%.o: %.c
#	gcc -c $^ $(CFLAGS) # $^ = $< but takes each file at once.

clean:
	rm *.o

