#pragma description ZIP local file entries
#pragma MIME application/zip
#pragma endian little

import std.mem;

enum Method : u16 {
    Stored = 0,
    Deflated = 8,
    Bzip2 = 12,
    Lzma = 14,
    Zstd = 93
};

struct LocalFileHeader {
    char signature[4];
    u16 version;
    u16 flags;
    Method method;
    u16 modTime;
    u16 modDate;
    u32 crc32;
    u32 compressedSize;
    u32 uncompressedSize;
    u16 nameLength;
    u16 extraLength;
    char name[nameLength];
    u8 extra[extraLength];
    u8 data[compressedSize];
};

struct Zip {
    LocalFileHeader entries[while(!std::mem::eof())];
};

Zip zip @ 0x00;
