### Trading cli 

### Requirements
- Cargo 
- Rust 


### Usage

1. build with ``` cargo build ```
2. To run program, either use ``` Cargo run ``` or directly call the excutable in target/debug or target/release folder
3. testing, run  ``` Cargo test ``` 


### Todos
- extended to use Clap with the option of using different assets and exchanges 
- Build a tui around the cli with Table support for rendering the order table
- Fix arbitrage detection
- Try using different exchanges and datasets

### Current Problems
- Data sets given have a negative spread (min ask is greater than max bid ), which means we'd be trading against 