// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Penulis berkas objek biner ELF64 (Executable and Linkable Format) mandiri.
//! Menghasilkan berkas ELF relocatable (.o) standar untuk arsitektur x86_64 dan ARM64 di Linux.

/// Arsitektur mesin untuk berkas ELF.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArsitekturElf {
    X86_64,
    AArch64,
}

/// Definisi simbol yang diekspor dalam berkas ELF.
#[derive(Debug, Clone)]
pub struct SimbolElf {
    pub nama: String,
    pub offset_nilai: u64,
    pub ukuran: u64,
    pub section_idx: u16,
    pub publik: bool,
}

/// Menghasilkan bita berkas objek ELF64 (.o) dari data kode mesin dan simbol.
pub fn buat_objek_elf64(
    arsitektur: ArsitekturElf,
    kode_text: &[u8],
    data_rodata: &[u8],
    simbol: &[SimbolElf],
) -> Vec<u8> {
    let mut berkas = Vec::new();

    // 1. Siapkan string tabel nama bagian (.shstrtab)
    let mut shstrtab = Vec::new();
    shstrtab.push(0); // Index 0: string kosong
    let offset_sh_text = shstrtab.len() as u32;
    shstrtab.extend_from_slice(b".text\0");
    let offset_sh_rodata = shstrtab.len() as u32;
    shstrtab.extend_from_slice(b".rodata\0");
    let offset_sh_symtab = shstrtab.len() as u32;
    shstrtab.extend_from_slice(b".symtab\0");
    let offset_sh_strtab = shstrtab.len() as u32;
    shstrtab.extend_from_slice(b".strtab\0");
    let offset_sh_shstrtab = shstrtab.len() as u32;
    shstrtab.extend_from_slice(b".shstrtab\0");

    // 2. Siapkan string tabel simbol (.strtab) dan tabel simbol (.symtab)
    let mut strtab = Vec::new();
    strtab.push(0);

    let mut symtab_entries = Vec::new();
    // Simbol 0 wajib NULL symbol
    symtab_entries.extend_from_slice(&[0u8; 24]);

    for s in simbol {
        let name_offset = strtab.len() as u32;
        strtab.extend_from_slice(s.nama.as_bytes());
        strtab.push(0);

        // STB_GLOBAL = 1, STT_FUNC = 2 -> (1 << 4) | 2 = 0x12
        let st_info = if s.publik { 0x12 } else { 0x02 };
        let st_other = 0u8;
        let st_shndx = s.section_idx;
        let st_value = s.offset_nilai;
        let st_size = s.ukuran;

        let mut entry = Vec::with_capacity(24);
        entry.extend_from_slice(&name_offset.to_le_bytes());
        entry.push(st_info);
        entry.push(st_other);
        entry.extend_from_slice(&st_shndx.to_le_bytes());
        entry.extend_from_slice(&st_value.to_le_bytes());
        entry.extend_from_slice(&st_size.to_le_bytes());
        symtab_entries.extend_from_slice(&entry);
    }

    // 3. Hitung offset bagian dalam berkas biner
    // ELF Header: 64 bytes
    let offset_header = 64;
    let offset_text = offset_header;
    let size_text = kode_text.len();

    let offset_rodata = (offset_text + size_text + 7) & !7;
    let size_rodata = data_rodata.len();

    let offset_symtab = (offset_rodata + size_rodata + 7) & !7;
    let size_symtab = symtab_entries.len();

    let offset_strtab = (offset_symtab + size_symtab + 7) & !7;
    let size_strtab = strtab.len();

    let offset_shstrtab = (offset_strtab + size_strtab + 7) & !7;
    let size_shstrtab = shstrtab.len();

    let offset_section_headers = (offset_shstrtab + size_shstrtab + 7) & !7;
    let count_sections = 6; // NULL, .text, .rodata, .symtab, .strtab, .shstrtab

    // 4. Bangun ELF Header 64-bit (64 bytes)
    let e_machine: u16 = match arsitektur {
        ArsitekturElf::X86_64 => 62,   // EM_X86_64
        ArsitekturElf::AArch64 => 183, // EM_AARCH64
    };

    berkas.extend_from_slice(&[
        0x7f, b'E', b'L', b'F', // e_ident[EI_MAG0..3]
        2,                      // EI_CLASS = ELFCLASS64
        1,                      // EI_DATA = ELFDATA2LSB (little endian)
        1,                      // EI_VERSION = EV_CURRENT
        0,                      // EI_OSABI = ELFOSABI_NONE / System V
        0, 0, 0, 0, 0, 0, 0, 0, // padding
    ]);
    berkas.extend_from_slice(&1u16.to_le_bytes()); // e_type = ET_REL (1)
    berkas.extend_from_slice(&e_machine.to_le_bytes()); // e_machine
    berkas.extend_from_slice(&1u32.to_le_bytes()); // e_version
    berkas.extend_from_slice(&0u64.to_le_bytes()); // e_entry
    berkas.extend_from_slice(&0u64.to_le_bytes()); // e_phoff
    berkas.extend_from_slice(&(offset_section_headers as u64).to_le_bytes()); // e_shoff
    berkas.extend_from_slice(&0u32.to_le_bytes()); // e_flags
    berkas.extend_from_slice(&64u16.to_le_bytes()); // e_ehsize
    berkas.extend_from_slice(&0u16.to_le_bytes()); // e_phentsize
    berkas.extend_from_slice(&0u16.to_le_bytes()); // e_phnum
    berkas.extend_from_slice(&64u16.to_le_bytes()); // e_shentsize
    berkas.extend_from_slice(&(count_sections as u16).to_le_bytes()); // e_shnum
    berkas.extend_from_slice(&5u16.to_le_bytes()); // e_shstrndx (.shstrtab index 5)

    assert_eq!(berkas.len(), 64);

    // 5. Tulis muatan bagian (Section Payloads) dengan padding alignment
    // .text
    berkas.extend_from_slice(kode_text);

    // .rodata
    while berkas.len() < offset_rodata {
        berkas.push(0);
    }
    berkas.extend_from_slice(data_rodata);

    // .symtab
    while berkas.len() < offset_symtab {
        berkas.push(0);
    }
    berkas.extend_from_slice(&symtab_entries);

    // .strtab
    while berkas.len() < offset_strtab {
        berkas.push(0);
    }
    berkas.extend_from_slice(&strtab);

    // .shstrtab
    while berkas.len() < offset_shstrtab {
        berkas.push(0);
    }
    berkas.extend_from_slice(&shstrtab);

    // 6. Tulis Section Header Table (64 bytes per header)
    while berkas.len() < offset_section_headers {
        berkas.push(0);
    }

    // Section 0: NULL Section
    berkas.extend_from_slice(&[0u8; 64]);

    // Section 1: .text (PROGBITS, ALLOC + EXECINSTR)
    tulis_section_header_elf64(
        &mut berkas,
        offset_sh_text,
        1, // SHT_PROGBITS
        6, // SHF_ALLOC | SHF_EXECINSTR
        offset_text as u64,
        size_text as u64,
        0,
        0,
        16, // align 16
        0,
    );

    // Section 2: .rodata (PROGBITS, ALLOC)
    tulis_section_header_elf64(
        &mut berkas,
        offset_sh_rodata,
        1, // SHT_PROGBITS
        2, // SHF_ALLOC
        offset_rodata as u64,
        size_rodata as u64,
        0,
        0,
        8, // align 8
        0,
    );

    // Section 3: .symtab (SYMTAB)
    tulis_section_header_elf64(
        &mut berkas,
        offset_sh_symtab,
        2, // SHT_SYMTAB
        0,
        offset_symtab as u64,
        size_symtab as u64,
        4, // sh_link -> .strtab (index 4)
        1, // sh_info -> index first non-local sym
        8, // align 8
        24, // sh_entsize = 24
    );

    // Section 4: .strtab (STRTAB)
    tulis_section_header_elf64(
        &mut berkas,
        offset_sh_strtab,
        3, // SHT_STRTAB
        0,
        offset_strtab as u64,
        size_strtab as u64,
        0,
        0,
        1,
        0,
    );

    // Section 5: .shstrtab (STRTAB)
    tulis_section_header_elf64(
        &mut berkas,
        offset_sh_shstrtab,
        3, // SHT_STRTAB
        0,
        offset_shstrtab as u64,
        size_shstrtab as u64,
        0,
        0,
        1,
        0,
    );

    berkas
}

fn tulis_section_header_elf64(
    out: &mut Vec<u8>,
    sh_name: u32,
    sh_type: u32,
    sh_flags: u64,
    sh_offset: u64,
    sh_size: u64,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: u64,
    sh_entsize: u64,
) {
    out.extend_from_slice(&sh_name.to_le_bytes());
    out.extend_from_slice(&sh_type.to_le_bytes());
    out.extend_from_slice(&sh_flags.to_le_bytes());
    out.extend_from_slice(&0u64.to_le_bytes()); // sh_addr
    out.extend_from_slice(&sh_offset.to_le_bytes());
    out.extend_from_slice(&sh_size.to_le_bytes());
    out.extend_from_slice(&sh_link.to_le_bytes());
    out.extend_from_slice(&sh_info.to_le_bytes());
    out.extend_from_slice(&sh_addralign.to_le_bytes());
    out.extend_from_slice(&sh_entsize.to_le_bytes());
}
