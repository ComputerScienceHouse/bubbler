use std::env;
use std::fs;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ConfigData {
    pub temperature_id: String,
    pub slot_ids: Vec<String>,
    pub drop_delay: u64,
}

impl ConfigData {
    pub fn initialize_slots(self: &ConfigData) -> std::io::Result<()> {
        for slot in self.clone().slot_ids {
            if slot.len() <= 4 {
                fs::write(
                    "/sys/class/gpio/export",
                    slot.to_string()
                )?;
                fs::write(
                    format!(
                        "/sys/class/gpio/gpio{}/direction",
                        slot.to_string()
                    ),
                    "high"
                )?;
                fs::write(
                    format!(
                        "/sys/class/gpio/gpio{}/active_low",
                        slot.to_string()
                    ),
                    "1"
                )?;
            }
        }
        return Ok(());
    }
    pub fn new() -> ConfigData {
        let addresses = env::var("BUB_SLOT_ADDRESSES").unwrap();
        let slots = addresses.split(",");
        let mut string_slots: Vec<String> = Vec::new();
        for slot in slots {
            string_slots.push(slot.to_string());
        }
        return ConfigData {
            temperature_id: env::var("BUB_TEMP_ADDRESS").unwrap(),
            slot_ids: string_slots,
            drop_delay: env::var("BUB_DROP_DELAY").unwrap().parse::<u64>().unwrap()
        };
    }
}

#[derive(Clone)]
pub struct AppData {
    pub config: ConfigData,
    pub drop_lock: Arc<Mutex<()>>,
}
