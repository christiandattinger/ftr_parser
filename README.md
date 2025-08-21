# FTR Parser
A Rust parser for FTR files, a datatype for Transation-Level Modeling simulations.
For more information on FTR see https://github.com/Minres/LWTR4SC

## Usage
FTR Parser provides two main methods `parse_ftr(file_name: PathBuf)` and `read_from_bytes(bytes: Vec<u8>)`, that return the FTR data as part of a single wrapper data structure, which can be used to access the individual transaction streams.

## License
ftr_parser is licensed under the [EUPL-1.2 license](LICENSE-EUPL-1.2.txt).
