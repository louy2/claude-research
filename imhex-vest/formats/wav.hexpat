#pragma description RIFF/WAVE header and format chunk
#pragma MIME audio/wav
#pragma endian little

struct RiffHeader {
    char riff[4];
    u32 size;
    char wave[4];
};

enum Format : u16 {
    PCM = 1,
    IEEEFloat = 3,
    ALaw = 6,
    MuLaw = 7,
    Extensible = 0xFFFE
};

struct FmtChunk {
    char id[4];
    u32 size;
    Format format;
    u16 channels;
    u32 sampleRate;
    u32 byteRate;
    u16 blockAlign;
    u16 bitsPerSample;
};

struct Wav {
    RiffHeader riff;
    FmtChunk fmt;
};

Wav wav @ 0x00;
