use std::env;

#[derive(Clone)]
pub struct ConfigData {
    pub temperature_id: String,
    pub slot_ids: Vec<String>,
    pub drop_delay: u64,
}

pub struct AppData {
    pub config: ConfigData
}

pub fn init() -> ConfigData {
    println!("Init!");
    let addresses = env::var("BUB_SLOT_ADDRESSES").unwrap();
    let slots = addresses.split(",");
    let mut string_slots: Vec<String> = Vec::new();
    for slot in slots {
        println!("Got slot!!");
        string_slots.push(slot.to_string());
    }
    return ConfigData {
        temperature_id: env::var("BUB_TEMP_ADDRESS").unwrap(),
        slot_ids: string_slots,
        drop_delay: env::var("BUB_DROP_DELAY").unwrap().parse::<u64>().unwrap()
    };
}
