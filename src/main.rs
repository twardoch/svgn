// this_file: src/main.rs

//! SVGN CLI - Command-line interface for the SVGN SVG optimizer

use anyhow::{Context, Result};
use clap::{Arg, Command};
use std::fs;
use svgn::{optimize, Config};
use base64::prelude::*;

fn main() -> Result<()> {
    let matches = Command::new("svgn")
        .version("0.1.0")
        .author("Adam Twardoch <adam@twardoch.com>")
        .about("A native Rust port of SVGO (SVG Optimizer)")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("Input SVG file")
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file (defaults to input file)")
                .required(false),
        )
        .arg(
            Arg::new("pretty")
                .short('p')
                .long("pretty")
                .help("Pretty-print the output SVG")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("multipass")
                .long("multipass")
                .help("Run optimizations multiple times")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("datauri")
                .long("datauri")
                .value_name("TYPE")
                .help("Output as data URI (base64, enc, unenc)")
                .required(false),
        )
        .get_matches();

    let input_path = matches.get_one::<String>("input").unwrap();
    let output_path = matches
        .get_one::<String>("output")
        .map(|s| s.as_str())
        .unwrap_or(input_path);

    // Read input file
    let input_svg = fs::read_to_string(input_path)
        .with_context(|| format!("Failed to read input file: {}", input_path))?;

    // Create configuration
    let config = Config {
        path: Some(input_path.clone()),
        plugins: vec![svgn::PluginConfig::Name("preset-default".to_string())],
        multipass: matches.get_flag("multipass"),
        js2svg: svgn::Js2SvgOptions {
            pretty: matches.get_flag("pretty"),
            indent: 2,
        },
        datauri: matches.get_one::<String>("datauri").cloned(),
    };

    // Optimize the SVG
    let result = optimize(&input_svg, Some(config))
        .with_context(|| "Failed to optimize SVG")?;

    if let Some(error) = result.error {
        eprintln!("Error during optimization: {}", error);
        std::process::exit(1);
    }

    // Handle data URI output
    let output_data = if let Some(datauri_type) = matches.get_one::<String>("datauri") {
        match datauri_type.as_str() {
            "base64" => {
                let encoded = base64::prelude::BASE64_STANDARD.encode(&result.data);
                format!("data:image/svg+xml;base64,{}", encoded)
            }
            "enc" => {
                let encoded = urlencoding::encode(&result.data);
                format!("data:image/svg+xml;charset=utf-8,{}", encoded)
            }
            "unenc" => {
                format!("data:image/svg+xml;charset=utf-8,{}", result.data)
            }
            _ => {
                eprintln!("Invalid datauri type. Use: base64, enc, or unenc");
                std::process::exit(1);
            }
        }
    } else {
        result.data
    };

    // Write output file
    fs::write(output_path, output_data)
        .with_context(|| format!("Failed to write output file: {}", output_path))?;

    // Print optimization info
    println!("Optimization complete!");
    println!("Original size: {} bytes", result.info.original_size);
    println!("Optimized size: {} bytes", result.info.optimized_size);
    println!("Compression ratio: {:.2}%", result.info.compression_ratio * 100.0);

    Ok(())
}