use std::{
    fs::{create_dir_all, File},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

const VERSION: u32 = 0;
const MAGIC: [u8; 8] = [0x89, 0x42, 0x49, 0x46, 0x0d, 0x0a, 0x1a, 0x0a];

fn is_bif(mut f: &File) -> bool {
    let mut buf = [0; MAGIC.len()];
    f.read_exact(&mut buf).expect("failed to read bytes");
    buf == MAGIC
}

fn read_u32(mut f: &File) -> u32 {
    let mut buf = [0; 4];
    f.read_exact(&mut buf).expect("failed to read bytes");
    u32::from_le_bytes(buf)
}

pub struct Image {
    pub name: String,
    pub timestamp: u32,
    pub offset: u32,
    pub size: usize,
}

pub struct Bif {
    pub path: PathBuf,
    pub version: u32,
    pub total_images: u32,
    pub framewise_separation: u32,
    pub images: Vec<Image>,
}

pub fn extract_image(mut f: &File, image: Image, output: &PathBuf) {
    f.seek(SeekFrom::Start(image.offset as u64))
        .expect("failed to seek");
    create_dir_all(output).expect("failed to create output directory");
    let mut buf = vec![0; image.size];
    f.read_exact(&mut buf).expect("failed to read bytes");
    let mut output_file = output.clone();
    output_file.push(format!("{}.jpg", image.name));
    let mut image = File::create(output_file).expect("failed to create image");
    image
        .write_all(&buf)
        .expect("failed to write image contents");
}

pub fn extract_images(bif: Bif, output: &PathBuf) {
    let f = File::open(bif.path).expect("file not found");
    for image in bif.images {
        extract_image(&f, image, output);
    }
}

pub fn decode(path: &PathBuf) -> Bif {
    let mut f = File::open(path).expect("file not found");
    assert!(is_bif(&f), "File is not valid BIF");

    let version = read_u32(&f);
    assert_eq!(version, VERSION, "Can only parse version 0 formats");

    let total_images = read_u32(&f);
    let framewise_separation = match read_u32(&f) {
        0 => 1000,
        x => x,
    };

    f.seek(SeekFrom::Start(0x40)).expect("failed to seek");
    let mut prev_timestamp = 0;
    let mut prev_offset = 0;
    let mut images: Vec<Image> = Vec::new();
    loop {
        let timestamp = read_u32(&f);
        let offset = read_u32(&f);
        if prev_offset != 0 {
            let name = format!(
                "frame_{:020}",
                prev_timestamp as u64 * framewise_separation as u64
            );
            images.push(Image {
                name,
                timestamp: prev_timestamp,
                offset: prev_offset,
                size: (offset - prev_offset) as usize,
            });
        }
        prev_timestamp = timestamp;
        prev_offset = offset;
        if timestamp == 0xFFFFFFFF {
            break;
        }
    }

    assert_eq!(
        total_images as usize,
        images.len(),
        "Did not read all images"
    );

    Bif {
        path: path.to_path_buf(),
        version,
        total_images,
        framewise_separation,
        images,
    }
}

pub fn encode(
    images: Vec<PathBuf>,
    bif_file: PathBuf,
    timestamp_interval: u32,
    framewise_separation: u32,
) -> Bif {
    let mut buffer = File::create(bif_file.as_path()).expect("could not make bif file");
    buffer.write_all(&MAGIC).expect("failed writing to buffer");
    buffer
        .write_all(&VERSION.to_le_bytes())
        .expect("failed writing to buffer");
    let total_images = images.len() as u32;
    buffer
        .write_all(&total_images.to_le_bytes())
        .expect("failed writing to buffer");
    buffer.seek(SeekFrom::Start(0x40)).expect("failed to seek");
    let mut prev_image_offset = 0x40 + 0x08 + (images.len() * 8) as u32;
    for idx in 0..images.len() {
        let timestamp = timestamp_interval * idx as u32;
        buffer
            .write_all(&timestamp.to_le_bytes())
            .expect("failed writing to buffer");
        let mut image = File::open(images[idx].to_path_buf()).expect("could not open image");
        let mut image_data = vec![];
        image
            .read_to_end(&mut image_data)
            .expect("could not read image");
        let image_size = image_data.len();
        buffer
            .write_all(&prev_image_offset.to_le_bytes())
            .expect("failed writing to buffer");
        let current_position = buffer
            .stream_position()
            .expect("could not get current position");
        buffer
            .seek(SeekFrom::Start(prev_image_offset as u64))
            .expect("failed to seek");
        buffer
            .write_all(&image_data)
            .expect("failed writing to buffer");
        prev_image_offset = prev_image_offset + image_size as u32;
        buffer
            .seek(SeekFrom::Start(current_position))
            .expect("failed to seek");
        if idx == images.len() - 1 {
            buffer
                .write_all(&(0xFFFFFFFF as u32).to_le_bytes())
                .expect("failed writing to buffer");
            buffer
                .write_all(&prev_image_offset.to_le_bytes())
                .expect("failed writing to buffer");
        }
    }

    Bif {
        path: bif_file.to_path_buf(),
        version: VERSION,
        total_images,
        framewise_separation,
        images: Vec::new(),
    }
}
