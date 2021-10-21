use std::fs;

use super::config::ConfigData;
use std::thread;
use std::time::Duration;

pub fn get_temperature(config: ConfigData) -> f32 {
    let temperature_id = 
        config.temperature_id;
    let path = 
        format!("/mnt/w1/{}/temperature12", temperature_id);
    let temperature = fs::read_to_string(
        path.clone()
    ).expect(&format!("Temperature sensor {} doesn't exist!", path));

    return temperature.trim_end().parse::<f32>().unwrap();
}

// TODO: Why the heck is the API like this?
pub fn get_slots(config: ConfigData) -> Vec<String> {
    let mut slots: Vec<String> = Vec::new();
    for slot in config.slot_ids {
        slots.push(
            match fs::File::open(format!("/mnt/w1/{}/id", slot)) {
                Err(_) => format!("Slot {} ({}) is empty", slots.len() + 1, slot),
                Ok(_) => format!("Slot {} ({}) is stocked", slots.len() + 1, slot),
            }
        )
    }
    return slots;
}

pub enum DropState {
    Success
}

pub enum DropError {
    MotorFailed,
    BadSlot
}

pub fn drop(config: ConfigData, slot: usize) -> Result<DropState, DropError> {
    if slot >= config.slot_ids.len() {
        return Err(DropError::BadSlot);
    }

    println!("Dropping {}!", slot);

    match fs::write(format!("/mnt/w1/{}/PIO",
                            config.slot_ids[slot - 1]), "1") {
        Err(err) => {
            println!("Error actuating motor: {:?}", err);
            return Err(DropError::MotorFailed);
        },
        Ok(_) => {},
    }
    thread::sleep(Duration::from_millis(config.drop_delay));
    match fs::write(format!("/mnt/w1/{}/PIO",
                            config.slot_ids[slot - 1]), "0") {
        Err(err) => {
            println!("Error actuating motor: {:?}", err);
            return Err(DropError::MotorFailed);
        },
        Ok(_) => {},
    }
    thread::sleep(Duration::from_secs(2));

    return Ok(DropState::Success);
}
