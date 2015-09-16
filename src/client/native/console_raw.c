#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <errno.h>
#include "minimal.h"

/*
 * A function to listen to user input on console. If Arrow_Up or Arrow_Down were
 * pressed by the user the function will return 0 (1). Otherwise if no escape char
 * was given the function will return the pressed key. If there is an unexpected
 * char after the escape char or the second expected char (as part of the arrow keys)
 * the return value will be -1.
 */
int key(void)
{
    const int UP = 0;
    const int DOWN = 1;
    const int OTHER = -1;

    int c;
    int val = 0;

    // Initialize terminal for raw-input
    if (terminal_init()) {
        if (errno == ENOTTY)
            fprintf(stderr, "This program requires a terminal.\n");
        else
            fprintf(stderr, "Cannot initialize terminal: %s.\n", strerror(errno));
        return EXIT_FAILURE;
    }

    // Listen to user input
    c = getc(stdin);
    if (c == 27) {
        // if it starts with an escape-char get next char
        c = getc(stdin);

        if (c == 91) {
            c = getc(stdin);

            if (c == 65) {
               val = UP;
            }
            else if (c == 66) {
               val = DOWN;
            }
            else val = OTHER;
        }
        else val = OTHER;
    }
    else val = c;

    // Restore original terminal state
    terminal_done();

    return val;
}
