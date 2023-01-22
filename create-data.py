from random import randrange

MEGABYTES = 1024 * 1024
GIGABYTES = 1024 * MEGABYTES
size = 12 * MEGABYTES

window_size = 4 * 1024

with open("test.bin", "wb") as binary_file:
    for _ in range(0, size, window_size):
        binary_file.write(bytes([randrange(256) for _ in range(window_size)]))
