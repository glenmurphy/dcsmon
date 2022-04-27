## dcsmon
![screenshot](./screenshot.png)

Command-line server browser for Digital Combat Simulator

## Installation

Download the latest dcsmon.exe executable from the [releases](https://github.com/glenmurphy/dcsmon/releases) page

## Usage
Uses your DCS username and password

    dcsmon.exe -u username -p password

Filter on server names using the -f command line

    dcsmon.exe -u username -p password -f australia

## Develop

Requires [Rust](https://www.rust-lang.org/tools/install)

    git clone https://github.com/glenmurphy/dcsmon.git
    cd dcsmon
    cargo build