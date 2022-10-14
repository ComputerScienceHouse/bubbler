use serde::Serialize;

use super::config::ConfigData;
use std::thread;
use std::fs::{self};
use std::fmt::Debug;
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
    );

    match temperature {
        Ok(temperature) => match temperature.trim_end().parse::<f32>() {
            Ok(temperature) => temperature,
            Err(err) => {
                eprintln!("Temperature sensor {} errored out: {:?}", path, err);
                0.0
            }
        },
        Err(_) => {
            eprintln!("Temperature sensor {} doesn't exist!", path);
            0.0
        }
    }
}

#[derive(Serialize)]
pub struct SlotStatus {
    pub id: String,
    pub number: i32,
    pub stocked: bool,
}
pub fn get_slots(config: ConfigData) -> Vec<SlotStatus> {
    config.slot_ids.iter().enumerate().map(|(number, slot)| {
        let st = fs::read_to_string(format!("/sys/class/gpio/gpio{}/value", &slot.check));
        SlotStatus {
            id: slot.drop.to_string().clone(),
            number: number as i32,
            stocked: match st {
                Ok(val) => val.starts_with("1"),
                Err(_) => false
            }
        }
    }).collect()
}

#[derive(Debug)]
pub enum DropState {
    Success
}

#[derive(Debug)]
pub enum DropError {
    MotorFailed,
    BadSlot
}

pub fn run_motor(slot_id: &u64, state: bool) -> Result<DropState, DropError> {
    let num_state = match state {
        true  => "1",
        false => "0",
    };
    let motor_okay = fs::write(format!("/sys/class/gpio/gpio{}/value", slot_id), num_state);

    return match motor_okay {
        Err(err) => {
            println!("Error actuating motor: {:?}", err);
            return Err(DropError::MotorFailed);
        },
        Ok(_) => Ok(DropState::Success),
    };
}

pub fn drop(config: ConfigData, slot: usize) -> Result<DropState, DropError> {
    if slot > config.slot_ids.len() || slot <= 0 {
        eprintln!("We were asked to drop an invalid slot {}: BadSlot!", slot);
        return Err(DropError::BadSlot);
    }

    let slot_id = &config.slot_ids[slot - 1];
    println!("Dropping {}!", slot);

    let mut result = Ok(DropState::Success);
    if let Err(err) = run_motor(&slot_id.drop, true) {
        eprintln!("Problem dropping {} ({})! {:?}", slot, slot_id.drop, err);
        result = Err(err);
    } else {
        println!("Sleeping for {}ms after dropping", config.drop_delay);
        thread::sleep(Duration::from_millis(config.drop_delay));
    }

    println!("Shutting off motor for slot {} ({})", slot, slot_id.drop);
    if let Err(err) = run_motor(&slot_id.drop, false) {
        eprintln!("Couldn't turn off motor for slot {} ({})! {:?}", slot, slot_id.drop, err);
        result = Err(err);
    }

    println!("Drop completed. Allowing another drop time to stop motors again.");
    thread::sleep(Duration::from_millis(config.drop_delay));

    println!("Shutting off motor again to ensure it's safe");
    if let Err(err) = run_motor(&slot_id.drop, false) {
        eprintln!("Couldn't turn off motor [again] for slot {} ({})! {:?}", slot, slot_id.drop, err);
        return Err(err);
    }

    println!("Drop transaction finished with {:?}", result);

    result
}
