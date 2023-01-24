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
