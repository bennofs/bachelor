use zstd;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use structopt::StructOpt;
use std::path::{PathBuf, Path};
use indicatif::{ProgressBar, ProgressStyle, HumanBytes};
use std::hash::Hash;
use std::collections::{HashMap};
use xml::reader::{XmlEvent, ParserConfig};
use std::convert::TryInto;
use zstd::block::{Compressor, Decompressor};

#[derive(Debug, StructOpt)]
struct Args {
    /// file specifying the order of revision by item,date pairs
    date_order_path: PathBuf,

    /// file with incremental data
    incr_path: Vec<PathBuf>,

    #[structopt(short="s",long,default_value="100")]
    /// number of samples to use for training the dictionary
    sample_count: usize,

    #[structopt(short="d",long,default_value="1000")]
    /// maximum size of the trained dictionary in bytes
    dict_size: usize,

    #[structopt(short="c",long,default_value="3")]
    /// level for compression of working data
    compress_level: i32,

    #[structopt(short="m",long,default_value="1000")]
    /// maximum number of megabytes to use before bailing out
    mem_limit: usize,
}

fn open_zstd(path: &Path) -> impl BufRead {
    let file = File::open(path).expect("wikidata dump file open");
    let decoder = zstd::Decoder::new(file).expect("zstd decoder init");
    BufReader::new(decoder)
}

fn iter_bar(count: usize) -> ProgressBar {
    let bar = ProgressBar::new(count.try_into().unwrap());
    let style = ProgressStyle::default_bar().template("{elapsed_precise} {percent:3} {bar:40} {pos:>6} / {len:<6} ETA {eta_precise} {msg}");
    bar.set_style(style);

    bar
}

fn open_dump(dump_path: &Path) -> (impl BufRead, ProgressBar) {
    let file = File::open(dump_path).expect("wikidata dump file open");
    let file_size = file.metadata().unwrap().len();
    let bar = ProgressBar::new(file_size);

    let style = ProgressStyle::default_bar().template("{elapsed_precise} {percent:3} {bar:40} {bytes:<7}/{total_bytes:>7} ETA {eta_precise} {msg}");
    bar.set_style(style);

    let decoder = zstd::Decoder::new(bar.wrap_read(file)).expect("zstd decoder init");
    (BufReader::new(decoder), bar)
}

fn read_file<A: Hash + Eq, B, F: FnMut(&str, &str) -> Option<(A,B)>>(path: &Path, mut mapper: F) -> HashMap<A, B> {
    let (reader, bar) = open_dump(path);
    bar.println(format!("reading input {}", path.to_string_lossy()));
    let mut out = HashMap::new();
    for line in reader.lines() {
        let line = line.unwrap();
        let parts: Vec<_> = line.split(",").collect();
        if let Some((key, val)) = mapper(parts[0], parts[1]) {
            out.insert(key, val);
        }
    }
    bar.finish();
    out
}

fn entity_to_int(entity: &str) -> u64 {
    let code = entity.chars().nth(0).unwrap();
    let v = if entity.starts_with("Property:") {
        entity[10..].parse::<u64>().unwrap()
    } else {
        entity[1..].parse::<u64>().unwrap() 
    };
    v * 4 + match code {
        'Q' => 0,
        'P' => 1,
        'L' => 2,
        _ => panic!("invalid entity type {} {}", code, entity)
    }
}

fn get_rev_text<T: Read>(reader: T) -> impl Iterator<Item=(String, String, String)> {
    let parser = ParserConfig::new()
        .whitespace_to_characters(true)
        .trim_whitespace(false)
        .create_reader(reader);

    let mut in_text = false;
    let mut in_revision = false;
    let mut in_date = false;
    let mut rev_date = None;
    let mut rev_title = None;
    let mut in_title = false;
    let mut in_ns = false;
    let mut ns = 0;
    let mut depth = 0;

    parser.into_iter().filter_map(move |event| {
        match event.unwrap() {
            XmlEvent::StartElement { ref name, .. } => {
                in_text = name.local_name == "text";
                in_title = depth == 2 && name.local_name == "title";
                in_ns = depth == 2 && name.local_name == "ns";
                in_date = depth == 3 && in_revision && name.local_name == "timestamp";
                if name.local_name == "revision" {
                    in_revision = true;
                }
                depth += 1;
                None
            }
            XmlEvent::EndElement { name, .. } => {
                in_text = false;
                in_date = false;
                in_title = false;
                in_ns = false;
                if name.local_name == "revision" {
                    in_revision = false;
                }
                depth -= 1;
                None
            }
            XmlEvent::Characters(data) => {
                if in_date {
                    rev_date = Some(data);
                } else if in_title {
                    rev_title = Some(data);
                } else if in_ns {
                    ns = data.parse().unwrap();
                } else if in_text && (ns == 120 || ns == 0) {
                    if let (Some(title), Some(date)) = (rev_title.clone(), rev_date.clone()) {
                        return Some((title, date, data));
                    }
                }
                None
            }
            _ => {
                None
            }
        }
    })
}

fn main() {
    let args = Args::from_args();

    // read all files
    let mut idx = 0;
    let item_date = read_file(&args.date_order_path, |item, date| {
        let i = idx;
        idx += 1;
        Some((entity_to_int(item), (date.to_string(), i)))
    });

    // train dictionary
    let reader = open_zstd(&args.incr_path[0]);
    let bar = iter_bar(args.sample_count);
    bar.println("collect samples for dict");
    let samples: Vec<_> = bar.wrap_iter(get_rev_text(reader).map(|(_title, _date, data)| data).take(args.sample_count)).collect();
    bar.finish();

    eprintln!("building dictionary");
    let dict_vec = zstd::dict::from_samples(&samples, args.dict_size).unwrap();

    
    // buffer for read revisions
    let mut buffer: Vec<Vec<u8>> = vec![Vec::new(); idx ];
    let mut compressor = Compressor::with_dict(dict_vec.clone());
    let mut total_size_comp: usize = 0;
    let mut comp_count: usize = 0;

    for incr in args.incr_path.iter() {
        let (reader, bar) = open_dump(incr);
        bar.println(format!("sort data (max index: {}) {}", idx, incr.to_string_lossy()));
        let count = get_rev_text(reader).filter(|(title, date, text)| {
            if let Some((wanted_date, idx)) = item_date.get(&entity_to_int(title)) {
                if wanted_date != date {
                    return false;
                }

                let mut compressed = compressor.compress(text.as_bytes(), args.compress_level).unwrap();
                compressed.shrink_to_fit();
                total_size_comp += compressed.capacity();
                comp_count += 1;
                let avg: f64 = (total_size_comp as f64) / (comp_count as f64);
                let ratio: f64 = (compressed.len() as f64) / (text.as_bytes().len() as f64);
                bar.set_message(&format!("mem {} avg {:.2} ratio {:.3}", HumanBytes(total_size_comp as u64), avg, ratio));
                buffer[*idx] = compressed;
                true
            } else {
                false
            }
        }).count();
        bar.set_message(&format!("{} items", count));
        bar.finish();
    }

    let mut decompressor = Decompressor::with_dict(dict_vec);
    let stream = std::io::stdout();
    let lock = stream.lock();
    let mut writer = std::io::BufWriter::new(lock);
    for data in buffer {
        if data.len() == 0 {
            continue;
        }
        let doc = decompressor.decompress(&data, 100 << 20).unwrap();
        writer.write_all(&doc).unwrap();
        writer.write_all("\n".as_bytes()).unwrap();
    }
}
