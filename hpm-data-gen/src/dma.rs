use regex::Regex;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// Parse dmamux data from header file (hpm_dmamux_src.h)
fn parse_dmamux_from_header(header_path: &Path) -> anyhow::Result<HashMap<String, usize>> {
    let content = std::fs::read_to_string(header_path)
        .map_err(|e| anyhow::anyhow!("Failed to read dmamux header {:?}: {}", header_path, e))?;

    // Match: #define HPM_DMA_SRC_UART0_RX (0x8UL) /* UART0 Receive */
    let pattern = Regex::new(r"#define\s+HPM_DMA_SRC_(\w+)\s+\((0x[0-9A-F]+)UL\)")
        .expect("Invalid dmamux regex");

    let mut dmamux = HashMap::new();

    for cap in pattern.captures_iter(&content) {
        let signal_name = cap.get(1).unwrap().as_str().to_string();
        let hex_value = cap.get(2).unwrap().as_str();
        let value = usize::from_str_radix(&hex_value[2..], 16) // Skip "0x" prefix
            .expect("Failed to parse hex value");

        dmamux.insert(signal_name, value);
    }

    if dmamux.is_empty() {
        anyhow::bail!("No DMAMUX definitions found in {:?}", header_path);
    }

    println!(
        "    Loaded {} dmamux entries from header: {:?}",
        dmamux.len(),
        header_path.file_name().unwrap()
    );

    Ok(dmamux)
}

/// Get dmamux header path for chip
/// Returns (Option<PathBuf>, Option<&'static str>) - (found_path, warning_message)
fn get_dmamux_header_path(chip_name: &str) -> (Option<PathBuf>, Option<String>) {
    let sdk_path = std::env::var("HPM_SDK_BASE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap().join("./hpm_sdk"));

    let header_path = match chip_name {
        n if n.starts_with("HPM5301") => sdk_path.join("soc/HPM5300/HPM5301/hpm_dmamux_src.h"),
        n if n.starts_with("HPM53") => sdk_path.join("soc/HPM5300/HPM5361/hpm_dmamux_src.h"),
        n if n.starts_with("HPM5E") => sdk_path.join("soc/HPM5E00/HPM5E31/hpm_dmamux_src.h"),
        n if n.starts_with("HPM62") => sdk_path.join("soc/HPM6200/HPM6280/hpm_dmamux_src.h"),
        n if n.starts_with("HPM63") => sdk_path.join("soc/HPM6300/HPM6360/hpm_dmamux_src.h"),
        n if n.starts_with("HPM67") || n.starts_with("HPM64") => {
            sdk_path.join("soc/HPM6700/HPM6750/hpm_dmamux_src.h")
        }
        n if n.starts_with("HPM68") => sdk_path.join("soc/HPM6800/HPM6880/hpm_dmamux_src.h"),
        n if n.starts_with("HPM6E") => sdk_path.join("soc/HPM6E00/HPM6E80/hpm_dmamux_src.h"),
        n if n.starts_with("HPM6P") => sdk_path.join("soc/HPM6P00/HPM6P81/hpm_dmamux_src.h"),
        _ => {
            return (
                None,
                Some(format!(
                    "Unknown chip family for DMAMUX: {}, skipping DMA channel generation",
                    chip_name
                )),
            )
        }
    };

    if header_path.exists() {
        (Some(header_path), None)
    } else {
        (
            None,
            Some(format!(
                "DMAMUX header not found: {:?}, skipping DMA channel generation",
                header_path
            )),
        )
    }
}

fn parse_signal(signal_name: &str, periph_name: &str) -> String {
    if signal_name.contains("_") {
        let suffix = signal_name.split("_").last().unwrap();

        if signal_name.starts_with("GPTMR") || signal_name.starts_with("NTMR") {
            format!("CH{}", suffix)
        } else {
            suffix.to_string()
        }
    } else if signal_name.starts_with("I2C") {
        "GLOBAL".to_string()
    } else {
        periph_name.to_string()
    }
}

pub fn handle_chip_dmamux_include<P: AsRef<Path>>(
    _path: P,
    chip: &mut hpm_data_serde::Chip,
) -> anyhow::Result<()> {
    // Load DMAMUX directly from SDK header file for all chips
    let (header_path, warning) = get_dmamux_header_path(&chip.name);

    let dmamux = if let Some(path) = header_path {
        parse_dmamux_from_header(&path)?
    } else {
        if let Some(msg) = warning {
            println!("    ⚠️  {}", msg);
        }
        return Ok(());
    };

    // Process the dmamux data for all cores
    for core in &mut chip.cores {
        // Clear deprecated include_dmamux field if present
        let _ = core.include_dmamux.take();

        for (signal_name, request_no) in &dmamux {
            for periph in core.peripherals.iter_mut() {
                let signal_periph_prefix =
                    signal_name.split('_').next().expect("empty signal_name");
                if periph.name == signal_periph_prefix {
                    let signal = parse_signal(signal_name, &periph.name);

                    periph
                        .dma_channels
                        .push(hpm_data_serde::chip::core::peripheral::DmaChannel {
                            signal: signal.clone(),
                            dmamux: Some("DMAMUX".to_string()),
                            request: *request_no as u8,
                        });
                }
            }
        }
    }

    Ok(())
}
