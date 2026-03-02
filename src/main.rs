use clap::{Parser, Subcommand};
use nd2_rs::{Nd2File, Result};
use serde::Serialize;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use tiff::encoder::{colortype::Gray16, TiffEncoder};

#[derive(Parser)]
#[command(name = "nd2-rs")]
#[command(version, about = "Read Nikon ND2 microscopy files")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print file metadata as JSON
    Info {
        /// Path to the ND2 file
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// Extract one frame and save as 16-bit TIFF
    Frame {
        /// Path to the ND2 file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output TIFF path
        #[arg(value_name = "OUTPUT")]
        output: PathBuf,

        /// Read frame by sequence index. If not provided, use (--p, --t, --c, --z)
        #[arg(short = 's', long)]
        sequence: Option<usize>,

        /// Position index (for --p/--t/--c/--z mode)
        #[arg(long, default_value_t = 0)]
        p: usize,
        /// Time index (for --p/--t/--c/--z mode)
        #[arg(long, default_value_t = 0)]
        t: usize,
        /// Channel index
        #[arg(long, default_value_t = 0)]
        c: usize,
        /// Z-stack index (for --p/--t/--c/--z mode)
        #[arg(long, default_value_t = 0)]
        z: usize,
    },
}

#[derive(Serialize)]
struct InfoOutput {
    positions: usize,
    frames: usize,
    channels: usize,
    height: usize,
    width: usize,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info { file } => {
            let mut nd2 = Nd2File::open(&file)?;
            let sizes = nd2.sizes()?;
            let output = InfoOutput {
                positions: *sizes.get("P").unwrap_or(&1),
                frames: *sizes.get("T").unwrap_or(&1),
                channels: *sizes.get("C").unwrap_or(&1),
                height: *sizes.get("Y").unwrap_or(&0),
                width: *sizes.get("X").unwrap_or(&0),
            };
            println!("{}", serde_json::to_string_pretty(&output).expect("JSON"));
        }
        Commands::Frame {
            file,
            output,
            sequence,
            p,
            t,
            c,
            z,
        } => {
            let mut nd2 = Nd2File::open(&file)?;
            let sizes = nd2.sizes()?;
            let height = *sizes.get("Y").ok_or_else(|| {
                nd2_rs::Nd2Error::InvalidFormat("Missing image height".to_string())
            })?;
            let width = *sizes.get("X").ok_or_else(|| {
                nd2_rs::Nd2Error::InvalidFormat("Missing image width".to_string())
            })?;

            let (pixels, source) = if let Some(sequence_index) = sequence {
                let frame = nd2.read_frame(sequence_index)?;
                let n_chan = *sizes.get("C").ok_or_else(|| {
                    nd2_rs::Nd2Error::InvalidFormat("Missing channel count".to_string())
                })?;
                let frame_pixels = height * width;
                if c >= n_chan {
                    return Err(nd2_rs::Nd2Error::InvalidFormat(format!(
                        "channel index {} out of range for {} channels",
                        c, n_chan
                    )));
                }
                let start = c * frame_pixels;
                let end = (c + 1) * frame_pixels;
                (
                    frame[start..end].to_vec(),
                    format!("sequence {sequence_index}, channel {c}"),
                )
            } else {
                (
                    nd2.read_frame_2d(p, t, c, z)?,
                    format!("p={p}, t={t}, c={c}, z={z}"),
                )
            };

            write_u16_tiff(&output, width as u32, height as u32, &pixels)?;
            println!(
                "wrote {} ({}x{}) from {}",
                output.display(),
                width,
                height,
                source
            );
        }
    }

    Ok(())
}

fn write_u16_tiff(output: &Path, width: u32, height: u32, pixels: &[u16]) -> Result<()> {
    let file = File::create(output)?;
    let mut encoder = TiffEncoder::new(file).map_err(|e| {
        nd2_rs::Nd2Error::InvalidFormat(format!("Failed to create TIFF encoder: {e}"))
    })?;

    let expected = (width as usize)
        .checked_mul(height as usize)
        .ok_or_else(|| nd2_rs::Nd2Error::InvalidFormat("Image dimensions overflow".to_string()))?;
    if pixels.len() != expected {
        return Err(nd2_rs::Nd2Error::InvalidFormat(format!(
            "Pixel count {} does not match image dimensions {}x{}",
            pixels.len(),
            width,
            height
        )));
    }

    encoder
        .write_image::<Gray16>(width, height, pixels)
        .map_err(|e| nd2_rs::Nd2Error::InvalidFormat(format!("Failed to write TIFF: {e}")))?;

    Ok(())
}
