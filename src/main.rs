use std::{fs, io::{Read, Write}};
use borsh::{BorshDeserialize};
use clap::{Arg, Command};

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, BorshDeserialize)]
pub struct BitmapFileHeader {
    pub bf_type: u16,      // Deve ser 'BM' (0x4D42)
    pub bf_size: u32,      // Tamanho total do arquivo
    pub bf_reserved1: u16, // Reservado (0)
    pub bf_reserved2: u16, // Reservado (0)
    pub bf_off_bits: u32,  // Deslocamento até o início dos dados da imagem
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, BorshDeserialize)]
pub struct BitmapInfoHeader {
    pub bi_size: u32,          // Tamanho do cabeçalho (40 bytes)
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
    // Documentação da CLI:
    // Este programa aplica dithering de Floyd-Steinberg a uma imagem BMP de 24 bits.
    // Uso:
    //   cargo run -- -i <caminho_para_imagem>
    // ou, após compilar:
    //   ./bmp_dither -i <caminho_para_imagem>
    // Opções:
    //   -i, --input <caminho>  Especifica o caminho para a imagem BMP de entrada (obrigatório).
    // Exemplo:
    //   ./bmp_dither -i ./lena_512.bmp
    // A saída é salva como 'dithered_output.bmp' no diretório atual, sobrescrevendo se já existir.
    // Use --help para mais informações: ./bmp_dither --help

    // Define a interface de linha de comando com clap
    let matches = Command::new("bmp_dither")
        .version("1.0")
        .about("Aplica dithering de Floyd-Steinberg a uma imagem BMP de 24 bits")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("CAMINHO")
                .help("Caminho para a imagem BMP de entrada")
                .required(true)
        )
        .get_matches();

    // Obtém o caminho de entrada da CLI
    let input_path = matches.get_one::<String>("input").expect("Caminho de entrada é obrigatório");

    // Lê o arquivo BMP de entrada
    let mut file = fs::File::open(input_path).expect("Falha ao abrir o arquivo de entrada");
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("Falha ao ler bytes");

    // Desserializa os cabeçalhos
    let file_header = BitmapFileHeader::deserialize(&mut &bytes[..14]).expect("Falha ao desserializar o cabeçalho do arquivo");
    let info_header = BitmapInfoHeader::deserialize(&mut &bytes[14..54]).expect("Falha ao desserializar o cabeçalho de informações");

    // Verifica se é um BMP de 24 bits
    if info_header.bi_bit_count != 24 || info_header.bi_compression != 0 {
        panic!("Apenas arquivos BMP de 24 bits sem compressão são suportados");
    }

    let width = info_header.bi_width as usize;
    let height = info_header.bi_height as usize;
    let padding = (4 - (width * 3) % 4) % 4; // Preenchimento de linha do BMP para múltiplo de 4 bytes
    let row_bytes = width * 3 + padding;

    // Extrai os dados dos pixels
    let data_start = file_header.bf_off_bits as usize;
    let mut pixels = vec![0u8; width * height * 3];
    for y in 0..height {
        let row_start = data_start + y * row_bytes;
        for x in 0..width {
            let pixel_idx = (y * width + x) * 3;
            let byte_idx = row_start + x * 3;
            pixels[pixel_idx] = bytes[byte_idx + 2]; // R
            pixels[pixel_idx + 1] = bytes[byte_idx + 1]; // G
            pixels[pixel_idx + 2] = bytes[byte_idx]; // B
        }
    }

    // Converte para escala de cinza e normaliza para [0, 1]
    let mut grayscale = vec![0.0f32; width * height];
    for i in 0..width * height {
        let r = pixels[i * 3] as f32 / 255.0;
        let g = pixels[i * 3 + 1] as f32 / 255.0;
        let b = pixels[i * 3 + 2] as f32 / 255.0;
        grayscale[i] = 0.299 * r + 0.587 * g + 0.114 * b; // Luminância
    }

    // Aplica dithering de Floyd-Steinberg
    let mut errors = vec![0.0f32; (width + 2) * (height + 2)];
    for y in 0..height {
        for x in 0..width {
            let idx = (y + 1) * (width + 2) + (x + 1);
            let old_pixel = grayscale[y * width + x] + errors[idx];
            let new_pixel = if old_pixel > 0.5 { 1.0 } else { 0.0 };
            grayscale[y * width + x] = new_pixel;
            let error = old_pixel - new_pixel;

            // Distribui o erro para os pixels vizinhos
            errors[idx + 1] += error * 7.0 / 16.0; // Direita
            errors[idx + width + 1] += error * 5.0 / 16.0; // Abaixo
            errors[idx + width] += error * 3.0 / 16.0; // Abaixo-esquerda
            errors[idx + width + 2] += error * 1.0 / 16.0; // Abaixo-direita
        }
    }

    // Converte de volta para RGB de 24 bits
    let mut dithered_pixels = vec![0u8; width * height * 3];
    for i in 0..width * height {
        let value = if grayscale[i] > 0.5 { 255 } else { 0 };
        dithered_pixels[i * 3] = value; // R
        dithered_pixels[i * 3 + 1] = value; // G
        dithered_pixels[i * 3 + 2] = value; // B
    }

    // Cria o arquivo BMP de saída (sobrescreve se existir)
    let mut output_file = fs::File::create("dithered_output.bmp").expect("Falha ao criar o arquivo de saída");
    output_file.write_all(&bytes[..data_start]).expect("Falha ao escrever os cabeçalhos");

    // Escreve os dados dos pixels ditherizados com preenchimento
    for y in 0..height {
        let row_start = y * width * 3;
        output_file.write_all(&dithered_pixels[row_start..row_start + width * 3]).expect("Falha ao escrever os dados dos pixels");
        output_file.write_all(&vec![0u8; padding]).expect("Falha ao escrever o preenchimento");
    }

    println!("Imagem ditherizada salva como dithered_output.bmp");
}