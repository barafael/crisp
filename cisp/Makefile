prompt: prompt.c
	cc -std=c99 -Wall prompt.c -lreadline -o prompt

parsing: parsing.c
	cc -std=c99 -Wall parsing.c mpc.c -lm -lreadline -o parsing

clean:
	rm prompt parsing

