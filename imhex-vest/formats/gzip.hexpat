#pragma description gzip member header
#pragma MIME application/gzip
#pragma endian little

bitfield Flags {
    text : 1;
    hcrc : 1;
    extra : 1;
    name : 1;
    comment : 1;
    padding : 3;
};

enum Os : u8 {
    Fat = 0,
    Amiga = 1,
    Vms = 2,
    Unix = 3,
    Macintosh = 7,
    Ntfs = 11,
    Unknown = 255
};

struct Gzip {
    u8 id1;
    u8 id2;
    u8 compressionMethod;
    Flags flags;
    u32 mtime;
    u8 extraFlags;
    Os os;
};

Gzip gzip @ 0x00;
