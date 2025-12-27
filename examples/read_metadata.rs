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

    println!("\n=== Attributes ===");
    let attrs = nd2.attributes()?;
    println!("{:#?}", attrs);

    println!("\n=== Text Info ===");
    let text_info = nd2.text_info()?;
    println!("{:#?}", text_info);

    println!("\n=== Experiment Loops ===");
    let experiment = nd2.experiment()?;
    println!("{:#?}", experiment);

    println!("\n=== Available Chunks ===");
    let chunks = nd2.chunk_names();
    for chunk in chunks {
        println!("  - {}", chunk);
    }

    Ok(())
}
