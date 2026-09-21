#pragma description Windows Bitmap (BMP) file and info headers
#pragma MIME image/bmp
#pragma endian little

struct BitmapFileHeader {
    char signature[2];
    u32 fileSize;
    u16 reserved1;
    u16 reserved2;
    u32 pixelDataOffset;
};

enum Compression : u32 {
    RGB = 0,
    RLE8 = 1,
    RLE4 = 2,
    Bitfields = 3,
    JPEG = 4,
    PNG = 5
};

struct BitmapInfoHeader {
    u32 headerSize;
    s32 width;
    s32 height;
    u16 planes;
    u16 bitsPerPixel;
    Compression compression;
    u32 imageSize;
    s32 xPixelsPerMeter;
    s32 yPixelsPerMeter;
    u32 colorsUsed;
    u32 importantColors;
};

struct Bmp {
    BitmapFileHeader file;
    BitmapInfoHeader info;
};

Bmp bmp @ 0x00;
