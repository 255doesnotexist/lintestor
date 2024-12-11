CC = gcc
CFLAGS = -Wall -Wextra -std=c99

all: test_program

test_program: main.o functions.o
	$(CC) $(CFLAGS) -o test_program main.o functions.o

main.o: main.c functions.h
	$(CC) $(CFLAGS) -c main.c

functions.o: functions.c functions.h
	$(CC) $(CFLAGS) -c functions.c

clean:
	rm -f *.o test_program