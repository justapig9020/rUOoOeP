# ruOoOep

## Introduction

**ruOoOep** is an out-of-order processor emulator written in Rust. It can directly execute assembly code and shows the cycle by cycle result on the TUI. Also, **ruOoOep** provides an interface to easily add new assembly instructions. As a proof of concept, ruOoOep currently supports basic arithmetic and memory access instructions.

## Features

- Implements out-of-order execution based on Tomasulo's algorithm.
- Displays each cycle's execution results through a TUI, including register renaming in the register file, instructions in the reservation station, and instructions currently being executed.
- Provides an interface for easily adding new instructions.

## Installation and Usage

### Requirements

- git
- cargo

### Example

1. Clone the repository:
   ```bash
   git clone https://github.com/justapig9020/rUOoOeP
   ```
2. Run the emulator:
   ```bash
   cargo run
   ```

## Technical Details

This project was presented at COSCUP 2022. For more detailed information, please refer to: [COSCUP 2022 Presentation](https://coscup.org/2022/zh-TW/session/LWCM3T) (Chinese).

## Contribution Guidelines

Currently, the project lacks:

1. Branch instructions.
2. GUI.

Contributions to these features are welcome! Feel free to open an issue or submit a pull request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
