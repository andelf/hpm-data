use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Chip {
    pub name: String,
    pub family: String,
    pub sub_family: String,
    // pub device_id: u32,
    #[serde(default)]
    pub keywords: Vec<String>,
    pub packages: Vec<chip::Package>,
    pub memory: Vec<chip::Memory>,
    // pub docs: Vec<chip::Doc>,
    pub cores: Vec<chip::Core>,
    // pub _raw: HashMap<String, String>,
}

pub mod chip {
    use std::collections::BTreeMap;

    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
    pub struct Package {
        pub name: String,
        pub package: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
    pub struct Memory {
        pub name: String,
        pub kind: memory::Kind,
        pub address: u32,
        #[serde(deserialize_with = "crate::parse_size_with_surfix")]
        pub size: u32,
    }

    pub mod memory {
        use serde::{Deserialize, Serialize};

        #[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(rename_all = "lowercase")]
        pub enum Kind {
            Flash,
            Ram,
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
    pub struct Doc {
        pub r#type: String,
        pub title: String,
        pub name: String,
        pub url: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
    pub struct Core {
        pub name: String,
        #[serde(default)]
        pub peripherals: Vec<core::Peripheral>,
        #[serde(default)]
        pub interrupts: Vec<core::Interrupt>,
        #[serde(default)]
        pub dma_channels: Vec<core::DmaChannels>,

        // Register block index helper
        #[serde(default)]
        pub resources: Vec<core::Resource>,
        #[serde(default)]
        pub clocks: Vec<core::Clock>,
        #[serde(default)]
        pub pins: Vec<core::IoPin>,

        // include fields, for common peripherals
        #[serde(skip_serializing_if = "Option::is_none")]
        pub include_interrupts: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub include_dmamux: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub include_peripherals: Option<Vec<String>>,

        // gen fields, auto gen from properties
        #[serde(skip_serializing_if = "Option::is_none")]
        pub gen_dma_channels: Option<BTreeMap<String, usize>>,
    }

    pub mod core {
        use serde::{Deserialize, Serialize};

        #[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct Peripheral {
            pub name: String,
            pub address: u32,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub registers: Option<peripheral::Registers>,

            #[serde(skip_serializing_if = "Option::is_none")]
            pub sysctl: Option<peripheral::Sysctl>,

            #[serde(default, skip_serializing_if = "Vec::is_empty")]
            pub pins: Vec<peripheral::Pin>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub interrupts: Option<Vec<peripheral::Interrupt>>, // TODO: This should just be a Vec
            #[serde(default, skip_serializing_if = "Vec::is_empty")]
            pub dma_channels: Vec<peripheral::DmaChannel>,
        }

        pub mod peripheral {
            use serde::{Deserialize, Serialize};

            #[derive(
                Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize,
            )]
            pub struct Registers {
                pub kind: String,
                pub version: String,
                pub block: String,
            }

            #[derive(
                Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize,
            )]
            pub struct Sysctl {
                /// GROUPx[0], or GROUPx[1]  ...
                pub group_link: usize,
                pub group_bit_offset: u8,
                pub resource_clock_top: Option<usize>,
                pub resource: usize,
                pub clock_node: Option<usize>,
            }

            #[derive(
                Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize,
            )]
            pub struct Pin {
                pub pin: pin::Pin,
                pub signal: String,
                #[serde(skip_serializing_if = "Option::is_none")]
                pub alt: Option<u8>,
            }

            pub mod pin {
                use serde::{Deserialize, Serialize};

                #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
                pub struct Pin {
                    pub port: char,
                    pub num: u8,
                }

                impl Pin {
                    pub fn parse(pin: &str) -> Option<Self> {
                        let mut chars = pin.chars();
                        let p = chars.next()?;
                        if p != 'P' {
                            return None;
                        }
                        let port = chars.next()?;
                        let num = chars.as_str().parse().ok()?;

                        Some(Self { port, num })
                    }
                }

                impl std::fmt::Display for Pin {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "P{}{:02}", self.port, self.num)
                    }
                }

                impl Serialize for Pin {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: serde::Serializer,
                    {
                        serializer.serialize_str(&format!("{self}"))
                    }
                }

                struct PinVisitor;

                impl<'de> serde::de::Visitor<'de> for PinVisitor {
                    type Value = Pin;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("pin")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok(Pin::parse(v).unwrap())
                    }
                }

                impl<'de> Deserialize<'de> for Pin {
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where
                        D: serde::Deserializer<'de>,
                    {
                        deserializer.deserialize_str(PinVisitor)
                    }
                }
            }

            #[derive(
                Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize,
            )]
            pub struct Interrupt {
                pub signal: String,
                pub interrupt: String,
            }

            #[derive(
                Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize,
            )]
            pub struct DmaChannel {
                pub signal: String,
                //  #[serde(skip_serializing_if = "Option::is_none")]
                //  pub dma: Option<String>,
                //  #[serde(skip_serializing_if = "Option::is_none")]
                //  pub channel: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none")]
                pub dmamux: Option<String>,
                pub request: u8,
            }
        }

        #[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct Interrupt {
            pub name: String,
            pub number: u8,
        }

        #[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct DmaChannels {
            pub name: String,
            pub dma: String,
            // DMAMUX output channel
            pub channel: u8,
            pub dmamux_channel: u8,
        }

        #[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct Resource {
            pub name: String,
            // SYSCTL
            pub index: usize,
        }

        #[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct Clock {
            pub name: String,
            // SYSCTL.CLOCK
            pub index: usize,
        }

        #[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct IoPin {
            pub name: String,
            // IOC pad index
            pub index: usize,
        }
    }
}

fn parse_size_with_surfix<'de, D>(deserializer: D) -> std::result::Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    if s.starts_with("0x") || s.starts_with("0X") {
        Ok(u32::from_str_radix(&s[2..], 16).expect(&format!("error while parsering {:?}", s)))
    } else if s.ends_with("K") {
        Ok(1024
            * u32::from_str_radix(&s[..s.len() - 1], 10)
                .expect(&format!("error while parsering {:?}", s)))
    } else if s.ends_with("KiB") {
        Ok(1024
            * u32::from_str_radix(&s[..s.len() - 3], 10)
                .expect(&format!("error while parsering {:?}", s)))
    } else if s.ends_with("KB") {
        Ok(1024
            * u32::from_str_radix(&s[..s.len() - 2], 10)
                .expect(&format!("error while parsering {:?}", s)))
    } else if s.ends_with("M") {
        Ok(1024
            * 1024
            * u32::from_str_radix(&s[..s.len() - 1], 10)
                .expect(&format!("error while parsering {:?}", s)))
    } else if s.ends_with("MiB") {
        Ok(1024
            * 1024
            * u32::from_str_radix(&s[..s.len() - 3], 10)
                .expect(&format!("error while parsering {:?}", s)))
    } else if s.ends_with("MB") {
        Ok(1024
            * 1024
            * u32::from_str_radix(&s[..s.len() - 2], 10)
                .expect(&format!("error while parsering {:?}", s)))
    } else {
        // parse pure digits here
        Ok(s.parse().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::path::Path;
    use std::{fs, str};

    use super::*;

    fn normalize_line(line: &str) -> Cow<'_, str> {
        // The python script saves with 4 spaces instead of 2
        let line = line.trim_start();

        // The python script escapes unicode
        let mut line = Cow::Borrowed(line);
        for symbol in [("\\u00ae", "\u{00ae}"), ("\\u2122", "\u{2122}")] {
            if line.contains(symbol.0) {
                line = Cow::Owned(line.replace(symbol.0, symbol.1));
            }
        }

        line
    }

    fn normalize(file: &[u8]) -> impl Iterator<Item = Cow<'_, str>> + '_ {
        str::from_utf8(file).unwrap().lines().map(normalize_line)
    }

    fn check_file(path: impl AsRef<Path>) {
        println!("Checking {:?}", path.as_ref());
        let original = fs::read(path).unwrap();
        let parsed: Chip = serde_yaml::from_slice(&original).unwrap();
        let reencoded = serde_yaml::to_string(&parsed).unwrap();
        //   println!("{:?}", parsed);
        // itertools::assert_equal(normalize(&original), normalize(&reencoded))
    }

    const CHIPS_DIR: &str = "data/chips/";

    #[test]
    fn test_one() {
        let path = Path::new(CHIPS_DIR).join("HPM5361.yaml");
        check_file(path);
    }
}
