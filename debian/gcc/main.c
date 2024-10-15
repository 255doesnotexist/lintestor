#include <stdio.h>
#include "functions.h"

int main() {
    printf("This program tests the availability of make and gcc.\n");
    printf("If you can see this message, both make and gcc are working!\n\n");
    
    int a = 10, b = 5;
    printf("Testing add function: %d + %d = %d\n", a, b, add(a, b));
    printf("Testing subtract function: %d - %d = %d\n", a, b, subtract(a, b));
    
    return 0;
}