//! Alpha 21164 SROM decoder

use std::{
    io::Write,
    process::{Command, Stdio},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os();
    let _ = args.next();
    let infilename = args.next().expect("Need input binary filename");
    let outbin_filename = args.next().expect("Need output binary filename");
    let outasm_filename = args.next().expect("Need output assembly filename");
    println!("Reading {}", infilename.to_string_lossy());
    let mut data = std::fs::read(infilename).expect("Failed to load file");
    println!("Read {} bytes", data.len());

    let mut outfile = std::fs::File::create(&outbin_filename)?;
    outfile.set_len(0)?;

    let remainder = data.len() % 25;
    if remainder != 0 {
        eprintln!("I want a multiple of 25 bytes for 21164 SROM");
    }

    let mut lines = Vec::new();

    while data.len() >= 25 {
        let remainder = data.split_off(25);
        let mut line = Vec::new();
        for word_bytes in data.chunks_exact(4) {
            let word_bytes: [u8; 4] = word_bytes.try_into().unwrap();
            let word: u32 = u32::from_le_bytes(word_bytes);
            line.push(word);
        }
        data = remainder;
        lines.push(line);
    }

    for line in lines {
        for word in line.iter() {
            print!("0x{:08x} ", word);
        }
        print!(" -- ");
        let decoded = process_line(&line);
        for word in decoded.iter() {
            print!("0x{:08x} ", word);
            outfile.write_all(&word.to_le_bytes())?;
        }
        println!();
    }

    let mut cmd = Command::new("alpha-linux-gnu-objdump");
    cmd.args(["-b", "binary", "-m", "alpha", "-D"]);
    cmd.arg(&outbin_filename);
    cmd.stdout(Stdio::piped());
    let child = cmd.spawn()?;
    let output = child.wait_with_output()?;
    let asm = std::str::from_utf8(&output.stdout)?;
    let mut outfile_asm = std::fs::File::create(&outasm_filename)?;
    for line in asm.lines() {
        if let Some(x) = line.find("\u{0009}pal1") {
            let instruction = &line[x..];
            println!("Decoding {:?}", instruction);
            let decoded = decode(instruction);
            writeln!(outfile_asm, "{} ; {}", line, decoded)?;
        } else {
            writeln!(outfile_asm, "{}", line)?;
        };
    }
    println!("Wrote {}", outasm_filename.to_string_lossy());
    Ok(())
}

fn decode(instruction: &str) -> String {
    let mut i = instruction.split_ascii_whitespace();
    let instruction = i.next().unwrap();
    let arg = i.next().unwrap();
    let arg = if let Some(suffix) = arg.strip_prefix("0x") {
        u32::from_str_radix(suffix, 16).unwrap()
    } else {
        u32::from_str_radix(arg, 10).unwrap()
    };
    match instruction {
        "pal19" => {
            let ipr = ipr_decode(arg as u16);
            let reg = REG_MAP[((arg >> 16) & 0x1F) as usize];
            format!("HW_MFPR: read {ipr} to {reg}")
        }
        "pal1b" => {
            format!("HW_LD")
        }
        "pal1d" => {
            let ipr = ipr_decode(arg as u16);
            let reg = REG_MAP[((arg >> 16) & 0x1F) as usize];
            format!("HW_MTPR: write {reg} to {ipr}")
        }
        "pal1e" => {
            format!("HW_REI")
        }
        "pal1f" => {
            format!("HW_ST")
        }
        _ => {
            panic!("Decoding wrong instruction {:?}", instruction);
        }
    }    
}

static REG_MAP: [&str; 32] = [
    "v0", "t0", "t1", "t2", "t3", "t4", "t5", "t6", "t7", "s0", "s1", "s2", "s3", "s4", "s5", "s6",
    "a0", "a1", "a2", "a3", "a4", "a5", "t8", "t9", "t10", "t11", "ra", "pv", "at", "gp", "sp",
    "zero",
];

fn ipr_decode(value: u16) -> &'static str {
    match value {
        // ISR R 100 E1
        0x100 => "ISR",
        // ITB_TAG W 101 E1
        0x101 => "ITB_TAG",
        // ITB_PTE R/W 102 E1
        0x102 => "ITB_PTE",
        // ITB_ASN R/W 103 E1
        0x103 => "ITB_ASN",
        // ITB_PTE_TEMP R 104 E1
        0x104 => "ITB_PTE_TEMP",
        // ITB_IA W 105 E1
        0x105 => "ITB_IA",
        // ITB_IAP W 106 E1
        0x106 => "ITB_IAP",
        // ITB_IS W 107 E1
        0x107 => "ITB_IS",
        // SIRR R/W 108 E1
        0x108 => "SIRR",
        // ASTRR R/W 109 E1
        0x109 => "ASTRR",
        // ASTER R/W 10A E1
        0x10A => "ASTER",
        // EXC_ADDR R/W 10B E1
        0x10B => "EXC_ADDR",
        // EXC_SUM R/W0C 10C E1
        0x10C => "EXC_SUM",
        // EXC_MASK R 10D E1
        0x10D => "EXC_MASK",
        // PAL_BASE R/W 10E E1
        0x10E => "PAL_BASE",
        // ICM R/W 10F E1
        0x10F => "ICM",
        // IPLR R/W 110 E1
        0x110 => "IPLR",
        // INTID R 111 E1
        0x111 => "INTID",
        // IFAULT_VA_FORM R 112 E1
        0x112 => "IFAULT_VA_FORM",
        // IVPTBR R/W 113 E1
        0x113 => "IVPTBR",
        // HWINT_CLR W 115 E1
        0x115 => "HWINT_CLR",
        // SL_XMIT W 116 E1
        0x116 => "SL_XMIT",
        // SL_RCV R 117 E1
        0x117 => "SL_RCV",
        // ICSR R/W 118 E1
        0x118 => "ICSR",
        // IC_FLUSH_CTL W 119 E1
        0x119 => "IC_FLUSH_CTL",
        // ICPERR_STAT R/W1C 11A E1
        0x11A => "ICPERR_STAT",
        // PMCTR R/W 11C E1
        0x11C => "PMCTR",
        // PALtemp0 R/W 140 E1
        0x140 => "PALtemp0",
        // PALtemp1 R/W 141 E1
        0x141 => "PALtemp1",
        // PALtemp2 R/W 142 E1
        0x142 => "PALtemp2",
        // PALtemp3 R/W 143 E1
        0x143 => "PALtemp3",
        // PALtemp4 R/W 144 E1
        0x144 => "PALtemp4",
        // PALtemp5 R/W 145 E1
        0x145 => "PALtemp5",
        // PALtemp6 R/W 146 E1
        0x146 => "PALtemp6",
        // PALtemp7 R/W 147 E1
        0x147 => "PALtemp7",
        // PALtemp8 R/W 148 E1
        0x148 => "PALtemp8",
        // PALtemp9 R/W 149 E1
        0x149 => "PALtemp9",
        // PALtemp10 R/W 14A E1
        0x14A => "PALtemp10",
        // PALtemp11 R/W 14B E1
        0x14B => "PALtemp11",
        // PALtemp12 R/W 14C E1
        0x14C => "PALtemp12",
        // PALtemp13 R/W 14D E1
        0x14D => "PALtemp13",
        // PALtemp14 R/W 14E E1
        0x14E => "PALtemp14",
        // PALtemp15 R/W 14F E1
        0x14F => "PALtemp15",
        // PALtemp16 R/W 150 E1
        0x150 => "PALtemp16",
        // PALtemp17 R/W 151 E1
        0x151 => "PALtemp17",
        // PALtemp18 R/W 152 E1
        0x152 => "PALtemp18",
        // PALtemp19 R/W 153 E1
        0x153 => "PALtemp19",
        // PALtemp20 R/W 154 E1
        0x154 => "PALtemp20",
        // PALtemp21 R/W 155 E1
        0x155 => "PALtemp21",
        // PALtemp22 R/W 156 E1
        0x156 => "PALtemp22",
        // PALtemp23 R/W 157 E1
        0x157 => "PALtemp23",
        // DTB_ASN W 200 E0
        0x200 => "DTB_ASN",
        // DTB_CM W 201 E0
        0x201 => "DTB_CM",
        // DTB_TAG W 202 E0
        0x202 => "DTB_TAG",
        // DTB_PTE R/W 203 E0
        0x203 => "DTB_PTE",
        // DTB_PTE_TEMP R 204 E0
        0x204 => "DTB_PTE_TEMP",
        // MM_STAT R 205 E0
        0x205 => "MM_STAT",
        // VA R 206 E0
        0x206 => "VA",
        // VA_FORM R 207 E0
        0x207 => "VA_FORM",
        // MVPTBR W 208 E0
        0x208 => "MVPTBR",
        // DTB_IAP W 209 E0
        0x209 => "DTB_IAP",
        // DTB_IA W 20A E0
        0x20A => "DTB_IA",
        // DTB_IS W 20B E0
        0x20B => "DTB_IS",
        // ALT_MODE W 20C E0
        0x20C => "ALT_MODE",
        // CC W 20D E0
        0x20D => "CC",
        // CC_CTL W 20E E0
        0x20E => "CC_CTL",
        // MCSR R/W 20F E0
        0x20F => "MCSR",
        // DC_FLUSH W 210 E0
        0x210 => "DC_FLUSH",
        // DC_PERR_STAT R/W1C 212 E0
        0x212 => "DC_PERR_STAT",
        // DC_TEST_CTL R/W 213 E0
        0x213 => "DC_TEST_CTL",
        // DC_TEST_TAG R/W 214 E0
        0x214 => "DC_TEST_TAG",
        // DC_TEST_TAG_TEMP R/W 215 E0
        0x215 => "DC_TEST_TAG_TEMP",
        // DC_MODE R/W 216 E0
        0x216 => "DC_MODE",
        // MAF_MODE R/W 217 E0
        0x217 => "MAF_MODE",
        _ => "UNKNOWN",
    }
}

static DFILLMAP: [usize; 128] = [
    /* data 0:127 -- fillmap[0:127]*/
    42, 44, 46, 48, 50, 52, 54, 56, /* 0:7 */
    58, 60, 62, 64, 66, 68, 70, 72, /* 8:15 */
    74, 76, 78, 80, 82, 84, 86, 88, /* 16:23 */
    90, 92, 94, 96, 98, 100, 102, 104, /* 24:31 */
    43, 45, 47, 49, 51, 53, 55, 57, /* 32:39 */
    59, 61, 63, 65, 67, 69, 71, 73, /* 40:47 */
    75, 77, 79, 81, 83, 85, 87, 89, /* 48:55 */
    91, 93, 95, 97, 99, 101, 103, 105, /* 56:63 */
    128, 130, 132, 134, 136, 138, 140, 142, /* 64:71 */
    144, 146, 148, 150, 152, 154, 156, 158, /* 72:79 */
    160, 162, 164, 166, 168, 170, 172, 174, /* 80:87 */
    176, 178, 180, 182, 184, 186, 188, 190, /* 88:95 */
    129, 131, 133, 135, 137, 139, 141, 143, /* 96:103 */
    145, 147, 149, 151, 153, 155, 157, 159, /* 104:111 */
    161, 163, 165, 167, 169, 171, 173, 175, /* 112:119 */
    177, 179, 181, 183, 185, 187, 189, 191, /* 120:127 */
];

fn process_line(line: &[u32]) -> [u32; 4] {
    if line.len() != 6 {
        panic!("Only want 6 words per line");
    }
    let mut output = [0u32; 4];
    for (out_idx, &in_idx) in DFILLMAP.iter().enumerate() {
        let in_word = in_idx >> 5;
        let in_offset = in_idx & 0x1F;
        let bit = (line[in_word] >> in_offset) & 1;
        if bit != 0 {
            let out_word = out_idx >> 5;
            let out_offset = out_idx & 0x1F;
            output[out_word] |= 1 << out_offset;
        }
    }
    output
}
