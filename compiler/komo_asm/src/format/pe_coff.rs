// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Penulis berkas objek biner PE/COFF mandiri untuk Windows (x86_64 dan ARM64).

/// Arsitektur target untuk berkas COFF.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArsitekturCoff {
    X86_64,
    ARM64,
}

/// Menghasilkan bita berkas objek PE/COFF (.obj) untuk Windows.
pub fn buat_objek_pe_coff(
    arsitektur: ArsitekturCoff,
    kode_text: &[u8],
    data_rdata: &[u8],
) -> Vec<u8> {
    let mut berkas = Vec::new();

    // 1. IMAGE_FILE_HEADER (20 bytes)
    let machine: u16 = match arsitektur {
        ArsitekturCoff::X86_64 => 0x8664, // IMAGE_FILE_MACHINE_AMD64
        ArsitekturCoff::ARM64 => 0xaa64,  // IMAGE_FILE_MACHINE_ARM64
    };

    let number_of_sections: u16 = 2; // .text dan .rdata
    let time_date_stamp: u32 = 0;
    let pointer_to_symbol_table: u32 = 0;
    let number_of_symbols: u32 = 0;
    let size_of_optional_header: u16 = 0;
    let characteristics: u16 = 0;

    berkas.extend_from_slice(&machine.to_le_bytes());
    berkas.extend_from_slice(&number_of_sections.to_le_bytes());
    berkas.extend_from_slice(&time_date_stamp.to_le_bytes());
    berkas.extend_from_slice(&pointer_to_symbol_table.to_le_bytes());
    berkas.extend_from_slice(&number_of_symbols.to_le_bytes());
    berkas.extend_from_slice(&size_of_optional_header.to_le_bytes());
    berkas.extend_from_slice(&characteristics.to_le_bytes());

    assert_eq!(berkas.len(), 20);

    // 2. Section Headers (40 bytes per section)
    let offset_headers = 20 + 40 * 2;
    let offset_text = (offset_headers + 15) & !15;
    let size_text = kode_text.len();

    let offset_rdata = (offset_text + size_text + 15) & !15;
    let size_rdata = data_rdata.len();

    // Section 1: .text (IMAGE_SCN_CNT_CODE | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ)
    tulis_section_header_coff(
        &mut berkas,
        b".text\0\0\0",
        size_text as u32,
        offset_text as u32,
        0x60500020, // Code, execute, read, align 16
    );

    // Section 2: .rdata (IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ)
    tulis_section_header_coff(
        &mut berkas,
        b".rdata\0\0",
        size_rdata as u32,
        offset_rdata as u32,
        0x40400040, // Initialized data, read, align 16
    );

    // 3. Tulis payloads .text dan .rdata
    while berkas.len() < offset_text {
        berkas.push(0);
    }
    berkas.extend_from_slice(kode_text);

    while berkas.len() < offset_rdata {
        berkas.push(0);
    }
    berkas.extend_from_slice(data_rdata);

    berkas
}

fn tulis_section_header_coff(
    out: &mut Vec<u8>,
    name: &[u8; 8],
    size: u32,
    offset: u32,
    characteristics: u32,
) {
    out.extend_from_slice(name);
    out.extend_from_slice(&size.to_le_bytes()); // VirtualSize
    out.extend_from_slice(&0u32.to_le_bytes()); // VirtualAddress
    out.extend_from_slice(&size.to_le_bytes()); // SizeOfRawData
    out.extend_from_slice(&offset.to_le_bytes()); // PointerToRawData
    out.extend_from_slice(&0u32.to_le_bytes()); // PointerToRelocations
    out.extend_from_slice(&0u32.to_le_bytes()); // PointerToLinenumbers
    out.extend_from_slice(&0u16.to_le_bytes()); // NumberOfRelocations
    out.extend_from_slice(&0u16.to_le_bytes()); // NumberOfLinenumbers
    out.extend_from_slice(&characteristics.to_le_bytes());
}
