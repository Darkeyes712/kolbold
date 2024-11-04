# Kolbold

**Kolbold** is a Rust library designed to measure time and memory complexity for code execution. It provides tools for gathering system metrics such as CPU and memory usage, making it ideal for performance analysis in both single-threaded and multi-threaded environments.

## Features

- **Time Complexity Measurement**: Collects CPU usage and execution time in synchronous and asynchronous contexts.
- **Memory Complexity Measurement**: Measures memory usage during code execution.
- **Thread-Specific Metrics**: Captures detailed metrics for individual threads in multi-threaded scenarios.
- **Easy Integration**: Simple API for adding performance measurement to your Rust code.

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
  - [Time Complexity](#time-complexity)
  - [Memory Complexity](#memory-complexity)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)
- [Contact](#contact)

## Installation

To use `kolbold`, add the following line to your `Cargo.toml`:

```toml
[dependencies]
kolbold = "1.0.6"
```

Then, include it in your code:

```rust
use kolbold::time::time_measurement::TimeComplexity;
use kolbold::memory::memory_measurement::MemoryComplexity;
```

## Usage

### Time Complexity

Measure time complexity for a synchronous, single-threaded function:

```rust
use kolbold::time::time_measurement::{TimeComplexity, TimeMeasurement};
use anyhow::Result;

fn main() -> Result<()> {
    let result = TimeMeasurement::measure_single_thread_sync(|| {
        // Your code to be measured
        let sum: u64 = (0..1_000_000).sum();
        println!("Sum: {}", sum);
    })?;

    println!("Measurement Data: {:?}", result);
    Ok(())
}
```

Measure time complexity for an asynchronous, single-threaded function:

```rust
use kolbold::time::time_measurement::{TimeComplexity, TimeMeasurement};
use anyhow::Result;
use tokio::main;

#[main]
async fn async_main() -> Result<()> {
    let result = TimeMeasurement::measure_single_thread_async(|| {
        // Your async code to be measured
        let product: u64 = (1..100).product();
        println!("Product: {}", product);
    }).await?;

    println!("Measurement Data: {:?}", result);
    Ok(())
}
```

### Memory Complexity

Measure memory usage for a synchronous function:

```rust
use kolbold::memory::memory_measurement::{MemoryComplexity, MemoryMeasurement};
use anyhow::Result;

fn main() -> Result<()> {
    let result = MemoryMeasurement::measure_single_thread_sync(|| {
        // Code to measure memory usage
        let vec: Vec<u64> = (0..1_000_000).collect();
        println!("Vector length: {}", vec.len());
    })?;

    println!("Memory Measurement Data: {:?}", result);
    Ok(())
}
```

## Documentation

For detailed API documentation and examples, refer to the [full documentation](https://docs.rs/kolbold).

## Contributing

Contributions are welcome! If you want to contribute to the project, please check out the [Contributing guidelines](https://github.com/Darkeyes712/kolbold/blob/master/CONTRIBUTING.md) for more information.

## License

This project is licensed under the Apache 2.0 License License. See the [Apache 2.0 License](https://github.com/Darkeyes712/kolbold/blob/master/LICENSE) file for details.

## Contact

For any inquiries or further information, feel free to reach out:

- **Author**: [Nikola Kolev](https://github.com/Darkeyes712)
- **Email**: nikolakolev712@gmail.com
