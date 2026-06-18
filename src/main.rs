use bif;
use glob::glob;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
enum Cli {
    Decode {
        #[structopt(help = "(e.g. index.bif)")]
        bif_file: PathBuf,

        #[structopt(help = "directory images will be saved to")]
        output: PathBuf,
    },
    Encode {
        #[structopt(help = "directory containing images to be indexed")]
        images: PathBuf,

        #[structopt(help = "(e.g. index.bif)")]
        bif_file: PathBuf,

        #[structopt(
            long = "ti",
            default_value = "1",
            help = "Timestamp interval between images (multiplied by the framewise separation value to determine timestamp values in milliseconds)"
        )]
        timestamp_interval: u32,

        #[structopt(
            long = "fs",
            default_value = "1000",
            help = "Timestamp multiplier (in milliseconds)"
        )]
        framewise_separation: u32,
    },
}

fn main() {
    let args = Cli::from_args();

    match args {
        Cli::Decode { bif_file, output } => {
            let b = bif::decode(&bif_file);
            println!("BIF Version: {}", b.version);
            println!("Number of images: {}", b.total_images);
            println!("Framewise Separation: {}ms", b.framewise_separation);
            println!("Generating images...");
            bif::extract_images(b, &output);
            println!("Finished.");
        }
        Cli::Encode {
            images,
            bif_file,
            timestamp_interval,
            framewise_separation,
        } => {
            assert!(images.is_dir(), "images must be a directory");
            let base_path = images.to_str().expect("couldn't coerce path to str");
            let mut images = Vec::<PathBuf>::new();
            for entry in glob(&format!("{base_path}/*")).expect("failed to glob") {
                let path = entry.expect("no file");
                match path.extension().and_then(|e| e.to_str()) {
                    Some("jpg") | Some("jpeg") | Some("webp") => {
                        images.push(path);
                    }
                    _ => {}
                }
            }
            images.sort();
            let b = bif::encode(images, bif_file, timestamp_interval, framewise_separation);
            println!("BIF Version: {}", b.version);
            println!("Number of images: {}", b.total_images);
            println!("Timestamp Interval: {}", timestamp_interval);
            println!("Framewise Separation: {}ms", b.framewise_separation);
            println!("Finished.");
        }
    }
}
