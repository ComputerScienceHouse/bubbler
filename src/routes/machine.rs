use serde::Serialize;

use super::config::{ConfigData, SlotConfig, SlotConfig::*};
use std::fmt::Debug;
use std::fs::{self};
use std::thread;
use std::time::Duration;

pub fn get_temperature(config: &ConfigData) -> f32 {
    let temperature_id = &config.temperature_id;
    if temperature_id.is_empty() {
        return 0.0;
    }
    let path = format!("/mnt/w1/{}/temperature12", temperature_id);
    let temperature = fs::read_to_string(path.clone());

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

fn is_stocked(slot: &SlotConfig) -> bool {
    match slot {
        GPIO { stocked, .. } => stocked.get_value().unwrap() == 1,
        NewdrinkSmall { stock, .. } => stock.get_value().unwrap() == 1,
        OWFS(id) => fs::File::open(format!("/mnt/w1/{}/id", id)).is_ok(),
    }
}

// TODO: Why the heck is the API like this?
pub fn get_slots_old(config: &ConfigData) -> Vec<String> {
    let mut slots: Vec<String> = Vec::new();
    for slot in &config.slots {
        slots.push(match is_stocked(slot) {
            false => format!("Slot {} ({}) is empty", slots.len() + 1, slot),
            true => format!("Slot {} ({}) is stocked", slots.len() + 1, slot),
        })
    }
    slots
}

#[derive(Serialize)]
pub struct SlotStatus {
    pub id: String,
    pub number: i32,
    pub stocked: bool,
}
pub fn get_slots(config: &ConfigData) -> Vec<SlotStatus> {
    config
        .slots
        .iter()
        .enumerate()
        .map(|(number, slot)| SlotStatus {
            id: format!("{}", slot),
            number: number as i32,
            stocked: is_stocked(slot),
        })
        .collect()
}

#[derive(Debug)]
pub enum DropState {
    Success,
}

#[derive(Debug)]
pub enum DropError {
    MotorFailed,
    BadSlot,
}

pub fn run_motor(slot: &SlotConfig, state: bool) -> Result<DropState, DropError> {
    let num_state = match state {
        true => 1,
        false => 0,
    };
    let motor_okay = match slot {
        OWFS(slot_id) => fs::write(format!("/mnt/w1/{}/PIO", slot_id), num_state.to_string())
            .map_err(|err| format!("{:?}", err)),
        GPIO { vend, .. } => vend
            .set_value(num_state)
            .map_err(|err| format!("{:?}", err)),
        NewdrinkSmall { vend, .. } => vend
            .set_value(num_state)
            .map_err(|err| format!("{:?}", err)),
    };
    match motor_okay {
        Err(err) => {
            println!("Error actuating motor: {}", err);
            Err(DropError::MotorFailed)
        }
        Ok(_) => Ok(DropState::Success),
    }
}

pub fn drop(config: &ConfigData, slot: usize) -> Result<DropState, DropError> {
    if slot > config.slots.len() || slot == 0 {
        eprintln!("We were asked to drop an invalid slot {}: BadSlot!", slot);
        return Err(DropError::BadSlot);
    }

    let slot_config = &config.slots[slot - 1];
    println!("Dropping {}!", slot_config);

    let mut result = Ok(DropState::Success);
    if let Some(latch) = config.latch.as_ref() {
        latch.open();
    }
    if let Err(err) = run_motor(slot_config, true) {
        eprintln!("Problem dropping {} ({})! {:?}", slot, slot_config, err);
        result = Err(err);
    } else {
        println!("Sleeping for {}ms after dropping", config.drop_delay);
        thread::sleep(Duration::from_millis(config.drop_delay));
    }

    if let NewdrinkSmall { cam, .. } = slot_config {
        println!("Waiting for cam {:?} to stop motor", cam.line());
        while cam.get_value().unwrap() != 1 {
            thread::sleep(Duration::from_millis(config.drop_delay));
        }
    }

    println!("Shutting off motor for slot {} ({})", slot, slot_config);
    if let Err(err) = run_motor(slot_config, false) {
        eprintln!(
            "Couldn't turn off motor for slot {} ({})! {:?}",
            slot, slot_config, err
        );
        result = Err(err);
    }

    match slot_config {
        OWFS(_) => {
            println!("Drop completed. Allowing another drop time to stop motors again.");
            thread::sleep(Duration::from_millis(config.drop_delay));

            println!("Shutting off motor again to ensure it's safe");
            if let Err(err) = run_motor(slot_config, false) {
                eprintln!(
                    "Couldn't turn off motor [again] for slot {} ({})! {:?}",
                    slot, slot_config, err
                );
                return Err(err);
            }
        }
        GPIO { .. } => {
            println!("Drop completed (GPIO drop, we trust the kernel)");
        }
        NewdrinkSmall { .. } => {
            println!("Drop completed, should have worked :)")
        }
    };

    println!("Drop transaction finished with {:?}", result);

    result
}
