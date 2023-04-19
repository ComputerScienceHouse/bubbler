use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use std::env;
use std::fmt::Display;
use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub enum SlotConfig {
    OWFS(String),
    GPIO {
        vend: LineHandle,
        stocked: LineHandle,
    },
    NewdrinkSmall {
        vend: LineHandle,
        stock: LineHandle,
        cam: LineHandle,
    },
}

impl Display for SlotConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OWFS(id) => write!(f, "{}", id),
            Self::GPIO { vend, stocked } => {
                write!(f, "{}.{}", vend.line().offset(), stocked.line().offset())
            }
            Self::NewdrinkSmall { vend, stock, cam } => write!(
                f,
                "{}.{}.{}",
                vend.line().offset(),
                stock.line().offset(),
                cam.line().offset()
            ),
        }
    }
}

#[allow(dead_code)]
pub struct Latch {
    delete_thread: JoinHandle<()>,
    sender: Sender<Instant>,
}

impl Latch {
    fn new(pin: LineHandle) -> Self {
        let (sender, receiver) = channel::<Instant>();
        let delete_thread = thread::spawn(move || {
            loop {
                let instant = receiver.recv().unwrap();
                let now = Instant::now();
                if now > instant {
                    continue;
                }
                pin.set_value(1).unwrap();
                thread::sleep(instant.duration_since(now));
                while let Ok(instant) = receiver.try_recv() {
                    let now = Instant::now();
                    if now > instant {
                        continue;
                    }
                    // Let this run finish first
                    thread::sleep(instant.duration_since(now));
                }
                pin.set_value(0).unwrap();
            }
        });
        Latch {
            delete_thread,
            sender,
        }
    }
    pub fn open(&self) {
        // No way the motor will spin > 1 minute
        self.sender
            .send(Instant::now() + Duration::from_secs(60))
            .unwrap();
    }
}

pub struct ConfigData {
    pub temperature_id: String,
    pub slots: Vec<SlotConfig>,
    pub latch: Option<Latch>,
    pub drop_delay: u64,
    pub poll_delay: u64,
}

impl ConfigData {
    pub fn new() -> ConfigData {
        let mut slots: Vec<SlotConfig> = Vec::new();
        if let Ok(addresses) = env::var("BUB_SLOT_ADDRESSES") {
            let slot_addresses = addresses.split(',');
            for slot in slot_addresses {
                slots.push(SlotConfig::OWFS(slot.to_string()));
            }
        } else if let Ok(vend) = env::var("BUB_NEW_VEND_PINS") {
            let vends = vend.split(",");
            let stock = env::var("BUB_NEW_STOCK_PINS").unwrap();
            let stock = stock.split(",");
            let cam = env::var("BUB_NEW_CAM_PINS").unwrap();
            let cam = cam.split(",");
            let mut chip = Chip::new("/dev/gpiochip0").unwrap();
            for ((vend, stock), cam) in vends.zip(stock).zip(cam) {
                let vend = chip
                    .get_line(vend.parse::<u32>().unwrap())
                    .unwrap()
                    .request(LineRequestFlags::OUTPUT, 0, "bubbler-vend")
                    .unwrap();
                let stock = chip
                    .get_line(stock.parse::<u32>().unwrap())
                    .unwrap()
                    .request(LineRequestFlags::INPUT, 0, "bubbler-stocked")
                    .unwrap();
                let cam = chip
                    .get_line(cam.parse::<u32>().unwrap())
                    .unwrap()
                    .request(LineRequestFlags::INPUT, 0, "bubbler-cam")
                    .unwrap();
                slots.push(SlotConfig::NewdrinkSmall { vend, stock, cam });
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
            latch: env::var("BUB_LATCH_PIN")
                .map(|pin| pin.parse::<u32>().unwrap())
                .map(|pin| {
                    Chip::new("/dev/gpiochip0")
                        .unwrap()
                        .get_line(pin)
                        .unwrap()
                        .request(LineRequestFlags::OUTPUT, 0, "bubbler-latch")
                        .unwrap()
                })
                .map(Latch::new)
                .ok(),
            drop_delay: env::var("BUB_DROP_DELAY").unwrap().parse::<u64>().unwrap(),
            poll_delay: env::var("BUB_POLL_DELAY").unwrap().parse::<u64>().unwrap(),
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
