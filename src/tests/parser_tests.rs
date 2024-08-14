#[cfg(test)]
mod test {
    use std::fs;
    use std::fs::File;
    use std::path::PathBuf;
    use crate::parse::parse_ftr;

    #[test]
    fn uncomp_parsing() {

        let path = PathBuf::from("./example_files/my_db.ftr");

        let mut ftr = parse_ftr(path).unwrap();

        println!("Timescale: {:?}", ftr.time_scale);
        println!("Max timestamp: {:?}", ftr.max_timestamp);

        println!("Dictionary: ");
        for entry in &ftr.str_dict {
            println!("key: {}, value: {}", entry.0, entry.1);
        }
        println!();

        println!("Streams: ");
        for stream in &ftr.tx_streams {
            println!("{:?}", stream);
        }
        println!();

        ftr.load_stream_into_memory(1);
        ftr.load_stream_into_memory(2);
        ftr.load_stream_into_memory(3);

        println!("Generators: ");
        for gen in &ftr.tx_generators {
            println!("{:?}", gen);
        }

        println!();

        println!("Streams: ");
        for stream in &ftr.tx_streams {
            println!("{:?}", stream);
        }
    }

    #[test]
    fn comp_parsing() {
        let path = PathBuf::from("example_files/my_db_c.ftr");

        let mut ftr = parse_ftr(path).unwrap();

        println!("Timescale: {:?}", ftr.time_scale);
        println!("Max timestamp: {:?}", ftr.max_timestamp);

        println!("Dictionary: ");
        for entry in &ftr.str_dict {
            println!("key: {}, value: {}", entry.0, entry.1);
        }
        println!();

        println!("Streams: ");
        for stream in &ftr.tx_streams {
            println!("{:?}", stream);
        }
        println!();

        ftr.load_stream_into_memory(1);
        ftr.load_stream_into_memory(2);
        ftr.load_stream_into_memory(3);

        println!("Generators: ");
        for gen in &ftr.tx_generators {
            println!("{:?}", gen);
        }
    }

    #[test]
    fn from_bytes() {
        let bytes = fs::read("./example_files/my_db.ftr").unwrap();

        let ftr = crate::parse::read_from_bytes(bytes).unwrap();

        println!("Timescale: {:?}", ftr.time_scale);
        println!("Max timestamp: {:?}", ftr.max_timestamp);

        println!("Dictionary: ");
        for entry in &ftr.str_dict {
            println!("key: {}, value: {}", entry.0, entry.1);
        }
        println!();

        println!("Streams: ");
        for stream in &ftr.tx_streams {
            println!("{:?}", stream);
        }
        println!();

        println!("Generators: ");
        for gen in &ftr.tx_generators {
            println!("{:?}", gen);
        }

        println!();
    }

}
