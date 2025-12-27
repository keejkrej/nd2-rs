use clap::{Parser, Subcommand};
use nd2_rs::{Nd2File, Result};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "nd2-rs")]
#[command(version, about = "Read Nikon ND2 microscopy files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Display file information and metadata
    Info {
        /// Path to the ND2 file
        #[arg(short, long, value_name = "FILE")]
        input: PathBuf,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// List all chunks in the file
    Chunks {
        /// Path to the ND2 file
        #[arg(short, long, value_name = "FILE")]
        input: PathBuf,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info { input, json } => {
            let mut nd2 = Nd2File::open(&input)?;
            print_info(&mut nd2, json)?;
        }
        Commands::Chunks { input, json } => {
            let nd2 = Nd2File::open(&input)?;
            print_chunks(&nd2, json)?;
        }
    }

    Ok(())
}

fn print_chunks(nd2: &Nd2File, json: bool) -> Result<()> {
    let chunks = nd2.chunk_names();

    if json {
        let output = serde_json::json!({
            "chunks": chunks,
            "count": chunks.len(),
        });
        println!("{}", serde_json::to_string_pretty(&output).expect("JSON serialization failed"));
    } else {
        println!("Chunks in file ({} total):", chunks.len());
        for chunk in chunks {
            println!("  - {}", chunk);
        }
    }

    Ok(())
}

fn print_info(nd2: &mut Nd2File, json: bool) -> Result<()> {
    let version = nd2.version();
    let attributes = nd2.attributes()?.clone();
    let text_info = nd2.text_info()?.clone();
    let experiment = nd2.experiment()?.clone();

    if json {
        let output = serde_json::json!({
            "version": {
                "major": version.0,
                "minor": version.1,
            },
            "attributes": attributes,
            "text_info": text_info,
            "experiment": experiment,
        });
        println!("{}", serde_json::to_string_pretty(&output).expect("JSON serialization failed"));
    } else {
        print_info_human(version, &attributes, &text_info, &experiment);
    }

    Ok(())
}

fn print_info_human(
    version: (u32, u32),
    attributes: &nd2_rs::Attributes,
    text_info: &nd2_rs::TextInfo,
    experiment: &[nd2_rs::ExpLoop],
) {
    println!("=== ND2 File Information ===\n");

    // Version
    println!("Format Version: {}.{}", version.0, version.1);
    println!();

    // Attributes
    println!("=== Image Attributes ===");
    if let Some(width) = attributes.width_px {
        println!("Dimensions: {} x {} px", width, attributes.height_px);
    } else {
        println!("Height: {} px", attributes.height_px);
    }
    println!("Channels: {}", attributes.component_count);
    println!("Frames: {}", attributes.sequence_count);
    println!("Bit Depth: {} bits (significant: {})",
        attributes.bits_per_component_in_memory,
        attributes.bits_per_component_significant
    );
    println!("Pixel Type: {:?}", attributes.pixel_data_type);

    if let Some(compression) = &attributes.compression_type {
        println!("Compression: {:?}", compression);
    }

    if let Some(tile_w) = attributes.tile_width_px {
        if let Some(tile_h) = attributes.tile_height_px {
            println!("Tile Size: {} x {} px", tile_w, tile_h);
        }
    }
    println!();

    // Text Info
    if text_info.description.is_some()
        || text_info.author.is_some()
        || text_info.date.is_some()
    {
        println!("=== Text Information ===");

        if let Some(desc) = &text_info.description {
            println!("Description: {}", desc);
        }
        if let Some(author) = &text_info.author {
            println!("Author: {}", author);
        }
        if let Some(date) = &text_info.date {
            println!("Date: {}", date);
        }
        if let Some(app_version) = &text_info.app_version {
            println!("App Version: {}", app_version);
        }
        println!();
    }

    // Experiment
    if !experiment.is_empty() {
        println!("=== Experiment ===");
        for (i, exp_loop) in experiment.iter().enumerate() {
            match exp_loop {
                nd2_rs::ExpLoop::TimeLoop(tl) => {
                    println!("  [{}] Time Loop: {} frames (level {})",
                        i, tl.count, tl.nesting_level);
                    println!("      Period: {:.2} ms, Duration: {:.2} ms",
                        tl.parameters.period_ms, tl.parameters.duration_ms);
                }
                nd2_rs::ExpLoop::ZStackLoop(zl) => {
                    println!("  [{}] Z-Stack Loop: {} slices (level {})",
                        i, zl.count, zl.nesting_level);
                    println!("      Step: {:.3} μm, Home: {}",
                        zl.parameters.step_um, zl.parameters.home_index);
                }
                nd2_rs::ExpLoop::XYPosLoop(xyl) => {
                    println!("  [{}] XY Position Loop: {} positions (level {})",
                        i, xyl.count, xyl.nesting_level);
                    println!("      Points: {}", xyl.parameters.points.len());
                }
                nd2_rs::ExpLoop::NETimeLoop(nel) => {
                    println!("  [{}] NE Time Loop: {} frames (level {})",
                        i, nel.count, nel.nesting_level);
                    println!("      Periods: {}", nel.parameters.periods.len());
                }
                nd2_rs::ExpLoop::CustomLoop(cl) => {
                    println!("  [{}] Custom Loop: {} iterations (level {})",
                        i, cl.count, cl.nesting_level);
                }
            }
        }
        println!();
    }
}
