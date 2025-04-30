use std::{fs, io::{BufRead, BufReader, Bytes, Read, Write}};
use borsh::{BorshDeserialize};

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, BorshDeserialize)]
pub struct BitmapFileHeader {
    pub bf_type: u16,      // Deve ser 'BM' (0x4D42)
    pub bf_size: u32,      // Tamanho total do arquivo
    pub bf_reserved1: u16, // Reservado (0)
    pub bf_reserved2: u16, // Reservado (0)
    pub bf_off_bits: u32,  // Offset até o início dos dados da imagem
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, BorshDeserialize)]
pub struct BitmapInfoHeader {
    pub bi_size: u32,          // Tamanho do header (40 bytes)
    pub bi_width: i32,         // Largura da imagem
    pub bi_height: i32,        // Altura da imagem
    pub bi_planes: u16,        // Número de planos (1)
    pub bi_bit_count: u16,     // Bits por pixel (24 para RGB)
    pub bi_compression: u32,   // Tipo de compressão (0 = BI_RGB)
    pub bi_size_image: u32,    // Tamanho da imagem
    pub bi_x_pels_per_meter: i32, // Pixels por metro horizontal
    pub bi_y_pels_per_meter: i32, // Pixels por metro vertical
    pub bi_clr_used: u32,      // Número de cores usadas
    pub bi_clr_important: u32, // Número de cores importantes
}


fn main() {
    let mut buffer: Bytes<_>;
    let mut file = fs::File::open("./lena_512.bmp").expect("Failed to open file");
    buffer = file.bytes();
    let bytes = buffer.collect::<Result<Vec<u8>, std::io::Error>>();
    let mut bytes = bytes.expect("Failed to read bytes");
    
    // let header = BitmapFileHeader::deserialize(&mut &bytes[..14]).expect("Failed to deserialize header");
    let info_header = BitmapInfoHeader::deserialize(&mut &bytes[..54]).expect("Failed to deserialize info header");
    let data = &bytes[..54];
    dbg!(info_header);
    let (red, green, blue) = bytes[data.len()..].chunks(3).map(|chunk| {
        let r = chunk[0];
        let g = chunk[1];
        let b = chunk[2];
        (r, g, b)
    }).collect::<(Vec<_>, Vec<_>, Vec<_>)>();
    
    let mut all_red = fs::File::create_new("all_red.bmp").expect("Failed to create file");
    all_red.write_all(&data).expect("Failed to write to file");
    all_red.write_all(&red).expect("Failed to write to file");
    all_red.write_all(&green).expect("Failed to write to file");
    all_red.write_all(&blue).expect("Failed to write to file");
    println!("Read line: {:?}", red);
}
