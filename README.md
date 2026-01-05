# 📻 Galena SDR

A modern Software Defined Radio (SDR) application, built with Rust and [Iced](https://iced.rs/). 

> ⚠️ **Note:** This project is under active development!

## ✨ Features

- 🎛️ **RTL-SDR** - Connect to your RTL-SDR dongle for live radio reception
- 📁 **WAV File Playback** - Analyze pre-recorded IQ samples from WAV files
- 📊 **Real-time Waterfall Display** - Visualize the spectrum over time
- 📻 **Multiple Demodulation Modes**
  - FM (Frequency Modulation)
  - Raw/AM (Amplitude Modulation)
  - More coming!

## 🔧 Prerequisites

### macOS

This application requires **libusb** (used by `rusb` → `rtl-sdr-rs` for USB communication with SDR hardware).

Install with Homebrew:

```bash
brew install libusb
```

### Linux

```bash
# Debian/Ubuntu
sudo apt install libusb-1.0-0-dev

# Arch
sudo pacman -S libusb
```

### Windows

libusb should be automatically handled by the build system. If you encounter issues, install the [Zadig](https://zadig.akeo.ie/) driver for your RTL-SDR device.

## 🚀 Building & Running

### Quick Start

```bash
# Clone the repository
git clone https://github.com/Msa360/galena.git
cd galena

# Run the application
cargo run --release
```

### Development

```bash
cargo run
```

## 🛠️ Tech Stack

- **[Iced](https://iced.rs/)** - Cross-platform GUI framework
- **[rtl-sdr-rs](https://crates.io/crates/rtl-sdr-rs)** - RTL-SDR device interface
- **[RustFFT](https://crates.io/crates/rustfft)** - Fast Fourier Transform
- **[Rodio](https://crates.io/crates/rodio)** - Audio playback
- **[Tokio](https://tokio.rs/)** - Async runtime


## 🤝 Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.
