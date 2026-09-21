#pragma description ELF identification and file header (32/64-bit)
#pragma MIME application/x-elf
#pragma endian little

enum Class : u8 { None = 0, Elf32 = 1, Elf64 = 2 };
enum Data : u8 { None = 0, Lsb = 1, Msb = 2 };
enum ObjectType : u16 { None = 0, Rel = 1, Exec = 2, Dyn = 3, Core = 4 };

struct Ident {
    char magic[4];
    Class class;
    Data data;
    u8 version;
    u8 osAbi;
    u8 abiVersion;
    padding[7];
};

struct Header32 {
    ObjectType type;
    u16 machine;
    u32 version;
    u32 entry;
    u32 phoff;
    u32 shoff;
    u32 flags;
    u16 ehsize;
    u16 phentsize;
    u16 phnum;
    u16 shentsize;
    u16 shnum;
    u16 shstrndx;
};

struct Header64 {
    ObjectType type;
    u16 machine;
    u32 version;
    u64 entry;
    u64 phoff;
    u64 shoff;
    u32 flags;
    u16 ehsize;
    u16 phentsize;
    u16 phnum;
    u16 shentsize;
    u16 shnum;
    u16 shstrndx;
};

struct Elf {
    Ident ident;
    if (ident.class == Class::Elf32) {
        Header32 header;
    } else if (ident.class == Class::Elf64) {
        Header64 header;
    }
};

Elf elf @ 0x00;
