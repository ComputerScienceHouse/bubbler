use std::env;
use std::fs;

#[derive(Clone)]
pub struct DrinkSlot {
    pub drop: u64,
    pub check: u64,
}

#[derive(Clone)]
pub struct ConfigData {
    pub temperature_id: String,
    pub slot_ids: Vec<DrinkSlot>,
    pub drop_delay: u64,
}

impl ConfigData {
    pub fn initialize_slots(self: ConfigData) -> std::io::Result<()> {
        for slot in self.slot_ids {
                fs::write(
                    format!(
                        "/sys/class/gpio/gpio{}/direction",
                        slot.drop.to_string()
                    ),
                    "high"
                )?;
                fs::write(
                    format!(
                        "/sys/class/gpio/gpio{}/active_low",
                        slot.drop.to_string()
                    ),
                    "1"
                )?;
        }
        return Ok(());
    }
    pub fn new() -> ConfigData {
        let addresses = env::var("BUB_SLOT_ADDRESSES").unwrap();
        let slots = addresses.split(",");
        let mut string_slots: Vec<DrinkSlot> = Vec::new();
        for slot in slots {
            let mut splits = slot.split(":");
            string_slots.push(DrinkSlot {
                drop: splits.next().unwrap().parse::<u64>().unwrap(),
                check: splits.next().unwrap().parse::<u64>().unwrap()
            });
        }
        return ConfigData {
            temperature_id: env::var("BUB_TEMP_ADDRESS").unwrap(),
            slot_ids: string_slots,
            drop_delay: env::var("BUB_DROP_DELAY").unwrap().parse::<u64>().unwrap()
        };
    }
}

pub struct AppData {
    pub config: ConfigData
}
