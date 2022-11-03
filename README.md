# Bubbler 💭

Drink machine implementation in Rust for the meme

## Usage

We've historically used Raspberry Pis. These instructions are tested against `2021-05-07-raspios-buster-armhf-lite.img` using what I can only assume is a Pi 3.

God help us.

### GPIO Hardware description

Bubbler expects every peripheral to be active-high:
* When the machine is stocked, current should be allowed to flow through the switch.
* When the machine is dropping, the vend pins are energized.

Something worth noting is that some Raspberry Pi pins start pulled high.
[Here's a list](https://www.raspberrypi.org/app/uploads/2012/02/BCM2835-ARM-Peripherals.pdf#page=102).

For Big Drink, we uses pins 9-15 for the vend motors, 17-23 for the stocked switches, and pin 16 for the latch relay (optional).

The latch pin gets pulled high for 1 minute after every drop.
It's intended to prevent motors from burning out if they get jammed.

In Big Drink, we use a relay, which allows us to cut power to all the motors when they shouldn't be running.
Note that this isn't a cutoff for the vend pins (that wouldn't be very useful), but rather the power source for the motors.

Boneless supplied [a wire diagram](https://slack-files.com/T04S6SNC4-F0496MU66MB-52a1601e54).

### Initial setup

*Note: These steps only matter for 1-wire machines*

1. Run `raspi-config` and enable I2C and One-Wire interfaces... Make sure to change the `root` and `pi` users' passwords!
1. Install `owfs`: `apt install owfs`
1. Modify `/etc/owfs.conf` and add the 1-wire HAT: `server: i2c = /dev/i2c-1:0`
1. Also make sure `mountpoint = /mnt/w1` in `owfs.conf`. Create `/mnt/w1` if it does not exist.
1. Reboot!

### Bubbler install

Install rust using [rustup](https://rustup.rs):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Grab a copy of bubbler:

```bash
git clone https://github.com/ComputerScienceHouse/bubbler
```

Build bubbler:

#### Compilation

```bash
cargo build --release
```

#### Cross compilation

Bubbler supports `cross`. To cross-compile for ARM, simply run

```bash
cargo install cross
cross build --target=armv7-unknown-linux-gnueabihf --release
```

Copy `.env.example` to `.env`. Make sure to modify it for your machine.

Drop this systemd unit into `/etc/systemd/system/bubbler.service`:

```ini
[Unit]
Description=Bubbler
After=network.target
Requires=owfs.service # Only needed on 1-wire systems. Remove otherwise.

[Service]
Restart=always
Type=simple
WorkingDirectory=/root/bubbler
EnvironmentFile=/root/bubbler/.env
ExecStart=/root/bubbler/target/release/bubbler


[Install]
WantedBy=multi-user.target
```

### Nginx setup

We use nginx to provide SSL termination with certbot.

First, install nginx and certbot:

```bash
apt install nginx certbot python3-certbot-nginx
```

Disable the default config:

```bash
rm /etc/nginx/sites-enabled/default
```

Make a new config for bubbler at `/etc/nginx/sites-available/bubbler`:

```nginx
server {
    listen [::]:80;
    listen 80;

    server_name littledrink.csh.rit.edu;

    location / {
        if ($http_x_auth_token != "DRINK MACHINE TOKEN GOES HERE") {
            return 403;
        }
        proxy_pass http://127.0.0.1:8080;
    }
}
```

Enable the config: `ln -s /etc/nginx/sites-{available,enabled}/bubbler`

Fetch a certificate by running `certbot`. Make sure to enable redirects.
