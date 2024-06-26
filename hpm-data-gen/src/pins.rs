//! parse sysctl registers from sdk_code

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub fn add_ioc_pins_from_sdk<P: AsRef<Path>>(
    data_dir: P,
    chip: &mut hpm_data_serde::Chip,
) -> anyhow::Result<()> {
    let sdk_path = std::env::var("HPM_SDK_BASE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| data_dir.as_ref().parent().unwrap().join("hpm_sdk"));

    let chip_name = &chip.name;

    let header_file = match chip_name {
        n if n.starts_with("HPM53") => sdk_path.join("soc/HPM5300/ip/hpm_ioc_regs.h"),
        n if n.starts_with("HPM62") => sdk_path.join("soc/HPM6200/ip/hpm_ioc_regs.h"),
        n if n.starts_with("HPM63") => sdk_path.join("soc/HPM6300/ip/hpm_ioc_regs.h"),
        n if n.starts_with("HPM67") || n.starts_with("HPM64") => {
            sdk_path.join("soc/HPM6700/ip/hpm_ioc_regs.h")
        }
        n if n.starts_with("HPM68") => sdk_path.join("soc/HPM6800/ip/hpm_ioc_regs.h"),
        n if n.starts_with("HPM6E") => sdk_path.join("soc/HPM6E00/ip/hpm_ioc_regs.h"),
        _ => anyhow::bail!("Unknown chip: {}", chip_name),
    };

    let content = std::fs::read_to_string(&header_file)
        .expect(format!("Failed to read file: {:?}", &header_file).as_str());

    // #define IOC_PAD_PA00 (0UL)
    let ioc_pin_pattern =
        regex::Regex::new(r"#define\s+IOC_PAD_(\w+)\s+\((\d+)UL\)").expect("Invalid regex");
    let pins: HashMap<String, u32> = ioc_pin_pattern
        .captures_iter(&content)
        .map(|cap| {
            (
                cap.get(1).unwrap().as_str().to_string(),
                cap.get(2).unwrap().as_str().parse().unwrap(),
            )
        })
        .collect();

    let mut pins: Vec<_> = pins
        .iter()
        .map(|(name, idx)| hpm_data_serde::chip::core::IoPin {
            name: name.clone(),
            index: *idx as _,
        })
        .collect();

    // fix wrong pins
    if chip_name.starts_with("HPM53") {
        pins = pins
            .into_iter()
            .filter(|p| !p.name.starts_with("PX"))
            .collect();
    }

    pins.sort_by_key(|p| p.index);

    for core in &mut chip.cores {
        core.pins = pins.clone();
    }

    Ok(())
}
