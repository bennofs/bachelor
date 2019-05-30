use zstd;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use structopt::StructOpt;
use std::path::{PathBuf, Path};
use indicatif::{ProgressBar, ProgressStyle};
use flate2::write::GzEncoder;

#[derive(Debug, StructOpt)]
struct Args {
    /// path to the zstd compressed wikidata dump file
    dump_path: PathBuf,

    #[structopt(short="s",long,default_value="10000")]
    /// number of samples to use for training the dictionary
    sample_count: usize,

    #[structopt(short="d",long,default_value="10000")]
    /// maximum size of the trained dictionary in bytes
    dict_size: usize,


}

fn open_dump(dump_path: &Path) -> (impl BufRead, ProgressBar) {
    let file = File::open(dump_path).expect("wikidata dump file open");
    let file_size = file.metadata().unwrap().len();
    let bar = ProgressBar::new(file_size);

    let style = ProgressStyle::default_bar().template("{elapsed_precise} {percent:2} {bar:40} {bytes:>7}/{total_bytes:>7} ETA {eta_precise}");
    bar.set_style(style);

    let decoder = zstd::Decoder::new(bar.wrap_read(file)).expect("zstd decoder init");
    (BufReader::new(decoder), bar)
}

fn main() {
    let args = Args::from_args();
    // let (reader, bar) = open_dump(&args.dump_path);
    // bar.println("collect samples for dict");
    // let samples: Vec<_> = reader.lines().map(|e| e.unwrap()).take(args.sample_count).collect();
    // bar.finish();

    // eprintln!("building dictionary");
    // let dict_vec = zstd::dict::from_samples(&samples, args.dict_size).unwrap();
    // let dict = zstd::dict::EncoderDictionary::new(&dict_vec, 3);

    let (reader, bar) = open_dump(&args.dump_path);
    bar.println("compress linewise with gzip");

    let stdout_stream = std::io::stdout();
    let stdout_lock = stdout_stream.lock();
    let mut writer = std::io::BufWriter::new(stdout_lock);
    for wikidata_object in reader.split('\n' as u8) {
        let wikidata_object = wikidata_object.unwrap();
        // let mut stream = zstd::stream::Encoder::with_prepared_dictionary(&mut stdout_lock, &dict).unwrap();
        let mut stream = GzEncoder::new(&mut writer, flate2::Compression::fast());
        stream.write_all(&wikidata_object).unwrap();
        stream.finish().unwrap();
    }
    bar.finish();
}
