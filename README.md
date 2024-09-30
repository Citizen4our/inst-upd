# InstUpd

InstUpd - project for **Inst**ant **Upd**ates on demand that allows you to send a camera shot and create a stream from your Raspberry Pi Zero 2 W via a bot.

## Features

- Capture photos remotely via Telegram bot
- Start and stop video streaming on demand
- Access video stream through a web page
- Remote access via ngrok tunneling

## Prerequisites

- Raspberry Pi Zero 2 W with OS installed and connected to WiFi
- USB webcam compatible with Raspberry Pi (you may need to use a Micro-B Male to A Female OTG Cable)
- Telegram API key (for bot integration) - create a new bot using BotFather on Telegram
- ngrok API key and static domain (for remote access) - sign up on the [ngrok website](https://ngrok.com/) and create a static domain (1 for free)

## Pre-installation

1. Install Rust on your PC. Follow the instructions on the [official website](https://www.rust-lang.org/tools/install).
2. Install RoboPLC framework on your PC. Follow the instructions on the [official website](https://info.bma.ai/en/actual/roboplc/quickstart_hello.html#creating-a-new-rust-project).
3. Add target to your Rust toolchain:
    ```bash
    rustup target add aarch64-unknown-linux-gnu
    ```

For macOS:
Install `cross`. Documentation [here](https://github.com/cross-rs/cross):

```bash
cargo install cross --git https://github.com/cross-rs/cross
```

## Installation
1. Install RoboPLC Manager on your Raspberry Pi Zero 2 W. Follow the instructions on the [official website](https://info.bma.ai/en/actual/roboplc/config.html#roboplc-manager).
2. Update the env file, filled by [.env.example](.env.example) file on Raspberry Pi - path `/etc/roboplc/program.env`.
3. Set a static IP for Raspberry Pi Zero 2 W on your router.

#### Optional:
1. Isolate CPUs for the RoboPLC Manager process. Follow these [instructions](https://yosh.ke.mu/raspberry_pi_isolating_cores_in_linux_kernel).
2. Increase swap size on Raspberry Pi Zero 2 W.

## Usage

1. To run the project on Raspberry Pi Zero 2 W, you need to build it on your PC and flash it to Raspberry Pi Zero 2 W.
   (Specify IP address in local network in [file](robo.toml))

```bash
robo flash -r -f
```

2. Interact with your Telegram bot using the following commands:
    - `/help` — List available commands
    - `/photo` — Get a photo from the camera
    - `/getvideo` — Get a URL with video stream
    - `/stopvideo` — Stop video stream

## Architecture

The project consists of several key components:

1. `camera.rs`: Handles camera operations and frame capture.
2. `telegram_bot.rs`: Implements the Telegram bot functionality.
3. `ws_server.rs`: Manages the WebSocket server for video streaming.
4. `core.rs`: Defines core data structures and configurations.

## Security

- Only authorized users (defined in `TELEGRAM_ALLOWED_USER_IDS` var) can interact with the bot.
- The admin user (defined by `TELEGRAM_ADMIN_USER_ID`) receives notifications about bot activities.
- ngrok is used for secure tunneling, allowing remote access to the video stream.

## Troubleshooting

- If the camera is not detected, ensure it's properly connected and compatible with Raspberry Pi Zero 2 W.
- If ngrok fails to start, check your authentication token and internet connection. Note that data transfer limits on the Free account are 1GB per month by default.
- For other issues, check the application logs (set `RUST_LOG=debug` for more detailed logging).

## TODO

The following features are planned for future development:

- Fine-tune the WebSocket server
- Transition to a real-time operating system
- Integrate with Home Assistant for smart home functionality

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
