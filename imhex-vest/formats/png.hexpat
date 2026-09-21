#pragma description PNG signature and chunk stream
#pragma MIME image/png
#pragma endian big

import std.mem;

struct Signature {
    u8 highBit;
    char png[3];
    char dosLineEnding[2];
    char dosEof;
    char unixLineEnding;
};

enum ColorType : u8 {
    Grayscale = 0,
    RGB = 2,
    Palette = 3,
    GrayscaleAlpha = 4,
    RGBA = 6
};

struct Ihdr {
    u32 width;
    u32 height;
    u8 bitDepth;
    ColorType colorType;
    u8 compressionMethod;
    u8 filterMethod;
    u8 interlaceMethod;
};

struct Chunk {
    u32 length;
    char type[4];
    if (type == "IHDR") {
        Ihdr ihdr;
    } else {
        u8 data[length];
    }
    u32 crc;
};

struct Png {
    Signature signature;
    Chunk chunks[while(!std::mem::eof())];
};

Png png @ 0x00;
