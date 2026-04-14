use nd2_rs::{Nd2File, Result};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path-to-nd2-file>", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];
    let mut nd2 = Nd2File::open(path)?;

    println!("=== ND2 File Information ===");
    println!("Version: {:?}", nd2.version());

    let summary = nd2.summary()?;
    println!("Logical frames: {}", summary.logical_frame_count);
    println!("Sizes: {:?}", summary.sizes);
    println!("Channels: {:?}", summary.channels);

    let first_plane = nd2.read_frame_2d(0, 0, 0, 0)?;
    println!("First plane pixels: {}", first_plane.len());

    Ok(())
}
