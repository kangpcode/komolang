// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Penulis berkas objek biner Mach-O 64-bit mandiri untuk macOS (x86_64 dan Apple Silicon ARM64).

/// Arsitektur target untuk berkas Mach-O.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArsitekturMachO {
    X86_64,
    ARM64,
}

/// Menghasilkan bita berkas objek Mach-O 64-bit (MH_OBJECT).
pub fn buat_objek_macho64(
    arsitektur: ArsitekturMachO,
    kode_text: &[u8],
    data_cstring: &[u8],
) -> Vec<u8> {
    let mut berkas = Vec::new();

    // 1. mach_header_64 (32 bytes)
    let magic: u32 = 0xfeedfacf; // MH_MAGIC_64
    let (cputype, cpusubtype): (u32, u32) = match arsitektur {
        ArsitekturMachO::X86_64 => (0x01000007, 0x80000003), // CPU_TYPE_X86_64, CPU_SUBTYPE_X86_64_ALL
        ArsitekturMachO::ARM64 => (0x0100000c, 0x00000000),  // CPU_TYPE_ARM64, CPU_SUBTYPE_ARM64_ALL
    };

    let filetype: u32 = 1; // MH_OBJECT
    let ncmds: u32 = 1; // 1 segment command (LC_SEGMENT_64)

    // Ukuran LC_SEGMENT_64:
    // segment_command_64: 72 bytes + 2 * section_64 (80 bytes each) = 232 bytes
    let sizeofcmds: u32 = 72 + 80 * 2;
    let flags: u32 = 0x00002000; // MH_SUBSECTIONS_VIA_SYMBOLS
    let reserved: u32 = 0;

    berkas.extend_from_slice(&magic.to_le_bytes());
    berkas.extend_from_slice(&cputype.to_le_bytes());
    berkas.extend_from_slice(&cpusubtype.to_le_bytes());
    berkas.extend_from_slice(&filetype.to_le_bytes());
    berkas.extend_from_slice(&ncmds.to_le_bytes());
    berkas.extend_from_slice(&sizeofcmds.to_le_bytes());
    berkas.extend_from_slice(&flags.to_le_bytes());
    berkas.extend_from_slice(&reserved.to_le_bytes());

    assert_eq!(berkas.len(), 32);

    // 2. LC_SEGMENT_64 (Command 0x19)
    let offset_header_dan_cmds = 32 + sizeofcmds as usize;
    let offset_text = (offset_header_dan_cmds + 15) & !15;
    let size_text = kode_text.len();

    let offset_cstring = (offset_text + size_text + 15) & !15;
    let size_cstring = data_cstring.len();

    let vmsize = (offset_cstring + size_cstring) as u64;

    // segment_command_64 (72 bytes)
    berkas.extend_from_slice(&0x19u32.to_le_bytes()); // cmd = LC_SEGMENT_64
    berkas.extend_from_slice(&sizeofcmds.to_le_bytes()); // cmdsize
    berkas.extend_from_slice(&[0u8; 16]); // segname (empty for MH_OBJECT)
    berkas.extend_from_slice(&0u64.to_le_bytes()); // vmaddr
    berkas.extend_from_slice(&vmsize.to_le_bytes()); // vmsize
    berkas.extend_from_slice(&0u64.to_le_bytes()); // fileoff
    berkas.extend_from_slice(&vmsize.to_le_bytes()); // filesize
    berkas.extend_from_slice(&7u32.to_le_bytes()); // maxprot = rwx (7)
    berkas.extend_from_slice(&7u32.to_le_bytes()); // initprot = rwx (7)
    berkas.extend_from_slice(&2u32.to_le_bytes()); // nsects = 2 (__text, __cstring)
    berkas.extend_from_slice(&0u32.to_le_bytes()); // flags

    // section_64: __TEXT,__text (80 bytes)
    tulis_section_macho64(
        &mut berkas,
        b"__text\0\0\0\0\0\0\0\0\0\0",
        b"__TEXT\0\0\0\0\0\0\0\0\0\0",
        offset_text as u64,
        size_text as u64,
        offset_text as u32,
        4, // align 2^4 = 16
        0x80000400, // S_REGULAR | S_ATTR_PURE_INSTRUCTIONS | S_ATTR_SOME_INSTRUCTIONS
    );

    // section_64: __TEXT,__cstring (80 bytes)
    tulis_section_macho64(
        &mut berkas,
        b"__cstring\0\0\0\0\0\0\0",
        b"__TEXT\0\0\0\0\0\0\0\0\0\0",
        offset_cstring as u64,
        size_cstring as u64,
        offset_cstring as u32,
        0, // align 2^0 = 1
        0x00000002, // S_CSTRING_LITERALS
    );

    // 3. Tulis payload __text dan __cstring
    while berkas.len() < offset_text {
        berkas.push(0);
    }
    berkas.extend_from_slice(kode_text);

    while berkas.len() < offset_cstring {
        berkas.push(0);
    }
    berkas.extend_from_slice(data_cstring);

    berkas
}

fn tulis_section_macho64(
    out: &mut Vec<u8>,
    sectname: &[u8; 16],
    segname: &[u8; 16],
    addr: u64,
    size: u64,
    offset: u32,
    align: u32,
    flags: u32,
) {
    out.extend_from_slice(sectname);
    out.extend_from_slice(segname);
    out.extend_from_slice(&addr.to_le_bytes());
    out.extend_from_slice(&size.to_le_bytes());
    out.extend_from_slice(&offset.to_le_bytes());
    out.extend_from_slice(&align.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // reloff
    out.extend_from_slice(&0u32.to_le_bytes()); // nreloc
    out.extend_from_slice(&flags.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // reserved1
    out.extend_from_slice(&0u32.to_le_bytes()); // reserved2
    out.extend_from_slice(&0u32.to_le_bytes()); // reserved3
}
