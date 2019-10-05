use zstd;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use structopt::StructOpt;
use std::path::{PathBuf, Path};
use indicatif::{ProgressBar, ProgressStyle};
use std::hash::Hash;
use std::collections::{HashMap, HashSet};
use serde_json;

#[derive(Debug, StructOpt)]
struct Args {
    /// path to list of recent revisions
    rev_path: PathBuf,

    /// path to nt ids
    nt_ids_path: PathBuf,

    /// path to json ids
    json_ids_path: PathBuf,

    /// file with data for the items to update
    update_stream_path: PathBuf,
}

fn open_zstd(path: &Path) -> impl BufRead {
    let file = File::open(path).expect("wikidata dump file open");
    let decoder = zstd::Decoder::new(file).expect("zstd decoder init");
    BufReader::new(decoder)
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

fn read_file<A: Hash + Eq, B, F: FnMut(&str,&str) -> (A,B)>(path: &Path, mut mapper: F) -> HashMap<A, B> {
    let (reader, bar) = open_dump(path);
    bar.println(format!("reading input {}", path.to_string_lossy()));
    let mut out = HashMap::new();
    for line in reader.lines() {
        let line = line.unwrap();
        let parts: Vec<_> = line.split(",").collect();
        let (key, val) = mapper(parts[0], parts[1]);
        out.insert(key, val);
    }
    bar.finish();
    out
}

fn entity_to_int(entity: &str) -> u64 {
    let code = entity.chars().nth(0).unwrap();
    let v = entity[1..].parse::<u64>().unwrap();
    v * 4 + match code {
        'Q' => 0,
        'P' => 1,
        'L' => 2,
        _ => panic!("invalid entity type {} {}", code, entity)
    }
}

fn next_line<T: Iterator<Item=std::io::Result<String>>>(it: &mut T) -> Option<(String, String)> {
    let line = it.next()?.unwrap();
    let mut parts =line.split(',');
    Some((parts.next().unwrap().into(), parts.next().unwrap().into()))
}

fn next_update<T: Iterator<Item=std::io::Result<String>>>(it: &mut T) -> Option<(String, String)> {
    let line = it.next()?.unwrap();
    let value = serde_json::from_str::<serde_json::Value>(&line).expect("json parse ok");
    if value.get("redirect").is_some() {
        let entity_id = value.get("entity").expect("has entity").as_str().expect("entity string").into();
        let line = format!("{{\"type\":\"skip_json\",\"id\":\"{}\"}}", entity_id).to_string();
        Some((entity_id, line))
    } else {
        let id = value.get("id").expect("has id").as_str().expect("is_string").into();
        Some((id, line))
    }
}

fn main() {
    let args = Args::from_args();

    // read all files
    let mut date_revs = HashMap::new();
    read_file(&args.rev_path, |rev, date| {
        date_revs.entry(date.to_string()).or_insert(HashSet::new()).insert(rev.parse::<u64>().unwrap());
        ((), ())
    });
    let json_ids = read_file(&args.json_ids_path, |entity, _| {
        (entity_to_int(entity), ())
    });
    let nt_ids = read_file(&args.nt_ids_path, |entity, _| {
        (entity_to_int(entity), ())
    });

    let (json_reader, bar) = open_dump(&args.json_ids_path);
    let nt_reader = open_zstd(&args.nt_ids_path);
    let update_reader = open_zstd(&args.update_stream_path);
    bar.println("aligning the entity sequences");

    let mut lines_json = json_reader.lines();
    let mut lines_nt = nt_reader.lines();
    let mut lines_update = update_reader.lines();
    let mut json_line = next_line(&mut lines_json);
    let mut nt_line = next_line(&mut lines_nt);
    let mut update_line = next_update(&mut lines_update);

    let stdout_stream = std::io::stdout();
    let stdout_lock = stdout_stream.lock();
    let mut writer = std::io::BufWriter::new(stdout_lock);
    loop {
        let (json_id, json_rev) = match json_line {
            Some(x) => x,
            None => break
        };

        let (nt_id, nt_date) = match nt_line {
            Some(x) => x,
            None => break
        };

        // if ids don't match, decide to skip one of the entities
        if nt_id != json_id {
            if nt_ids.contains_key(&entity_to_int(&json_id)) {
                if json_ids.contains_key(&entity_to_int(&nt_id)) {
                    writeln!(writer, "{{\"type\":\"misorder\",\"nt\":\"{}\", \"json\":\"{}\"}}", nt_id, json_id).unwrap();
                }
                nt_line = next_line(&mut lines_nt);
                json_line = Some((json_id, json_rev));
                writeln!(writer, "{{\"type\":\"skip_nt\",\"id\":\"{}\"}}", nt_id).unwrap();
                continue;
            } else {
                nt_line = Some((nt_id, nt_date));
                json_line = next_line(&mut lines_json);
                writeln!(writer, "{{\"type\":\"skip_json\",\"id\":\"{}\"}}", json_id).unwrap();
                continue;
            }
        }

        // emit update if rev date is not equal
        if let Some(ref revs) = date_revs.get(&nt_date) {
            let need_update = !revs.contains(&json_rev.parse().unwrap());
            if need_update {
                match update_line {
                    Some(ref update) if update.0 == nt_id => {
                        writeln!(writer, "{}", update.1).unwrap();
                    }
                    _ => {
                        writeln!(writer,
                                 "{{\"type\":\"need_update\",\"id\":\"{}\",\"date\":\"{}\"}}",
                                 nt_id,
                                 nt_date).unwrap();
                    }
                }
            }

            update_line = update_line.and_then(|update| {
                if update.0 == nt_id {
                    next_update(&mut lines_update)
                } else {
                    Some(update)
                }
            });
        }

        json_line = next_line(&mut lines_json);
        nt_line = next_line(&mut lines_nt);

    }
}
