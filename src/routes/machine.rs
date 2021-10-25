use std::fs;

use super::config::ConfigData;
use std::thread;
use std::time::Duration;

pub fn get_temperature(config: ConfigData) -> f32 {
    let temperature_id = 
        config.temperature_id;
    if temperature_id.is_empty() {
        return 0.0;
    }
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

pub fn run_motor(slot_id: String, state: bool) -> Result<DropState, DropError> {
    let num_state = match state {
        true  => "1",
        false => "0",
    };

    let motor_okay = match slot_id.len() > 4 {
        true  => fs::write(format!("/mnt/w1/{}/PIO", slot_id), num_state),
        false => fs::write(format!("/sys/class/gpio/gpio{}/value", slot_id), num_state),
    };
    return match motor_okay {
        Err(err) => {
            println!("Error actuating motor: {:?}", err);
            return Err(DropError::MotorFailed);
        },
        Ok(_) => Ok(DropState::Success),
    };
}

pub fn drop(config: ConfigData, slot: usize) -> Result<DropState, DropError> {
    if slot >= config.slot_ids.len() {
        return Err(DropError::BadSlot);
    }

    let slot_id = &config.slot_ids[slot - 1];
    println!("Dropping {}!", slot);

    if let Err(err) = run_motor(slot_id.clone(), true) {
        return Err(err);
    }
    thread::sleep(Duration::from_millis(config.drop_delay));
    if let Err(err) = run_motor(slot_id.clone(), false) {
        return Err(err);
    }
    thread::sleep(Duration::from_secs(2));

    return Ok(DropState::Success);
}
