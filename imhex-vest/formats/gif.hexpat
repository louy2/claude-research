#pragma description GIF header and logical screen descriptor
#pragma MIME image/gif
#pragma endian little

bitfield ScreenFlags {
    globalColorTableSize : 3;
    sortFlag : 1;
    colorResolution : 3;
    globalColorTable : 1;
};

struct Header {
    char signature[3];
    char version[3];
};

struct LogicalScreenDescriptor {
    u16 width;
    u16 height;
    ScreenFlags flags;
    u8 backgroundColorIndex;
    u8 pixelAspectRatio;
};

struct Gif {
    Header header;
    LogicalScreenDescriptor screen;
};

Gif gif @ 0x00;
