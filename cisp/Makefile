CFLAGS = -Wall -Wextra -std=c99 # Use variables for abstraction :)

.PHONY: all clean

all: main


prompt: prompt.c
	gcc prompt.c -lreadline -o prompt.o $(CFLAGS) 

parsing: parsing.c
	gcc parsing.c mpc.c -lm -lreadline -o parsing.o $(CFLAGS) 

#main: main.o lib.o
#	gcc -o $@ $< $(CFLAGS) # $@ = target, $< = input

#%.o: %.c
#	gcc -c $^ $(CFLAGS) # $^ = $< but takes each file at once.

clean:
	rm *.o

