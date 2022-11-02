use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use std::env;
use std::fmt::Display;
use std::sync::Mutex;

pub enum SlotConfig {
    OWFS(String),
    GPIO {
        vend: LineHandle,
        stocked: LineHandle,
    },
}

impl Display for SlotConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OWFS(id) => write!(f, "{}", id),
            Self::GPIO { vend, stocked } => {
                write!(f, "{}.{}", vend.line().offset(), stocked.line().offset())
            }
        }
    }
}

pub struct ConfigData {
    pub temperature_id: String,
    pub slots: Vec<SlotConfig>,
    pub drop_delay: u64,
}

impl ConfigData {
    pub fn new() -> ConfigData {
        let mut slots: Vec<SlotConfig> = Vec::new();
        if let Ok(addresses) = env::var("BUB_SLOT_ADDRESSES") {
            let slot_addresses = addresses.split(',');
            for slot in slot_addresses {
                slots.push(SlotConfig::OWFS(slot.to_string()));
            }
        } else {
            let vend = env::var("BUB_VEND_PINS").unwrap();
            let vend = vend.split(',');
            let stocked = env::var("BUB_STOCKED_PINS").unwrap();
            let stocked = stocked.split(',');
            let mut chip = Chip::new("/dev/gpiochip0").unwrap();
            for (vend, stocked) in vend.zip(stocked) {
                let vend = chip
                    .get_line(vend.parse::<u32>().unwrap())
                    .unwrap()
                    .request(LineRequestFlags::OUTPUT, 0, "bubbler-vend")
                    .unwrap();
                let stocked = chip
                    .get_line(stocked.parse::<u32>().unwrap())
                    .unwrap()
                    .request(LineRequestFlags::INPUT, 0, "bubbler-stocked")
                    .unwrap();
                slots.push(SlotConfig::GPIO { vend, stocked });
            }
        }
        ConfigData {
            temperature_id: env::var("BUB_TEMP_ADDRESS").unwrap(),
            slots,
            drop_delay: env::var("BUB_DROP_DELAY").unwrap().parse::<u64>().unwrap(),
        }
    }
}

impl Default for ConfigData {
    fn default() -> ConfigData {
        ConfigData::new()
    }
}

pub struct AppData {
    pub config: Mutex<ConfigData>,
}
