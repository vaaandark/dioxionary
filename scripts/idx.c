// Usage:
//   gcc idx.c -o idx
//   ./idx path/to/xxx.idx
//
// Then you'll get the entries of the index file, just like:
//   a | 0 | 2214592512
//   A and B agglutinogens | 2214592512 | 402653184
//   A AND NOT B gate | 2617245696 | 318767104
//   A as well as B | 2936012800 | 469762048
//   A B. S. pill | 3405774848 | 754974720
//   a back number | 4160749568 | 486539264
//   a bad actor | 352387072 | 335544320
//   a bad egg | 687931392 | 100663296
//   a bad hat | 788594688 | 100663296
//   a bad job | 889257984 | 251658240

#include <stdio.h>
int main(int argc, char *argv[])
{
    char word[256];
    unsigned offset, size;
    FILE *fp = fopen(argv[1], "rb");
    int pos = 0;
    int ch;
    while ((ch = fgetc(fp)) != EOF) {
        word[pos++] = ch;
        if (ch == '\0') {
            pos = 0;
            fread(&offset, sizeof(unsigned), 1, fp);
            fread(&size, sizeof(unsigned), 1, fp);
            printf("%s | %u | %u\n", word, offset, size);
        }
    }
    return 0;
}
