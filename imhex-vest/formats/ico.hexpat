#pragma description Windows icon directory
#pragma MIME image/x-icon
#pragma endian little

enum ImageType : u16 { Icon = 1, Cursor = 2 };

struct Header {
    u16 reserved;
    ImageType type;
    u16 count;
};

struct DirEntry {
    u8 width;
    u8 height;
    u8 colorCount;
    u8 reserved;
    u16 planes;
    u16 bitCount;
    u32 size;
    u32 offset;
};

struct Ico {
    Header header;
    DirEntry entries[header.count];
};

Ico ico @ 0x00;
