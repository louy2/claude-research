#pragma description Quite OK Image header
#pragma MIME image/qoi
#pragma endian big

import type.magic;

enum Channels : u8 { RGB = 3, RGBA = 4 };
enum Colorspace : u8 { SRGB = 0, Linear = 1 };

struct Qoi {
    type::Magic<"qoif"> magic;
    u32 width;
    u32 height;
    Channels channels;
    Colorspace colorspace;
};

Qoi qoi @ 0x00;
