use zstd;
use log::{info,debug,warn,error};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use clap::{App, Arg};

fn main() {
    let matches = App::new("Test compressing wikidata objects with zstd dictionary")
        .version("1.0")
        .author("Benno Fünfstück")
        .arg(Arg::with_name("DUMP")
             .help("Wikidata dump file (zstd compressed)")
             .required(true)
        )
        .arg(Arg::with_name("DICTSAMPLES")
             .short("-s")
             .default_value("100000")
             .takes_value(true)
             .help("number of samples to create the dictionary")
        )
        .arg(Arg::with_name("DICTSIZE")
             .short("-d")
             .default_value("100000")
             .takes_value(true)
             .help("max size of the dictionary")
        ) 
        .get_matches();

    let dump_file = File::open(matches.value_of("DUMP").expect("DUMP is required")).expect("DUMP file open ok");
    let dump_stream = BufReader::new(zstd::Decoder::new(dump_file).expect("zstd stream init"));
    let sample_count: usize = matches.value_of("DICTSAMPLES").expect("DICTSAMPLES has default").parse().unwrap();
    let dict_size: usize = matches.value_of("DICTSIZE").expect("DICTSIZE has default").parse().unwrap();

    eprintln!("collecting samples for dictionary");
    let samples: Vec<_> = dump_stream.lines().map(|e| e.unwrap()).take(sample_count).collect();

    eprintln!("building dictionary");
    let dict_vec = zstd::dict::from_samples(&samples, dict_size).unwrap();
    let dict = zstd::dict::EncoderDictionary::new(&dict_vec, 3);

    eprintln!("compressing linewise with dictionary");
    let dump_file = File::open(matches.value_of("DUMP").expect("DUMP is required")).expect("DUMP file open ok");
    let dump_stream = BufReader::new(zstd::Decoder::new(dump_file).expect("zstd stream init"));

    let mut stdout_stream = std::io::stdout();
    let mut stdout_lock = stdout_stream.lock();
    for wikidata_object in dump_stream.lines() {
        let wikidata_object = wikidata_object.unwrap();
        let mut stream = zstd::stream::Encoder::with_prepared_dictionary(&mut stdout_lock, &dict).unwrap();
        stream.write_all(wikidata_object.as_bytes()).unwrap();
        stream.finish().unwrap();
    }
}
