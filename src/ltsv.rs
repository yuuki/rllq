use std::io::{self, BufRead, BufReader, Stdin, stdin};
use std::fs::File;
use std::path::Path;
use std::collections::HashMap;

use error::*;

pub type Record = HashMap<String, String>;
pub type FieldGroupCount = HashMap<String, i32>;

pub enum LineReader {
    Stdin(Stdin),
    FileIn(BufReader<File>),
}

impl LineReader {
    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        match *self {
            LineReader::Stdin(ref mut r) => r.read_line(buf),
            LineReader::FileIn(ref mut r) => r.read_line(buf),
        }
    }
}

pub fn open_file(name: &str) -> Result<LineReader, Error> {
    match name {
        "-" => Ok(LineReader::Stdin(stdin())),
        _ => {
            let f = File::open(&Path::new(name))?;
            Ok(LineReader::FileIn(BufReader::new(f)))
        }
    }
}

pub fn parse_head(input: &mut LineReader) -> Result<Record, Error> {
    let found: String;
    loop {
        let mut line = String::new();
        match input.read_line(&mut line).map_err(Error::Io) {
            Ok(0) | Ok(1) => continue,
            Ok(_) => {
                found = line;
                break;
            }
            Err(err) => return Err(err),
        }
    }

    let mut record = Record::new();
    for field in found.split('\t').collect::<Vec<&str>>().into_iter() {
        let v = field.splitn(2, ':').collect::<Vec<&str>>();
        match v.len() {
            0 | 1 => {
                return Err(ParseError { msg: format!("invalid ltsv field: {}", field) })
                    .map_err(Error::Parse);
            }
            2 => record.insert(v[0].to_string(), v[1].to_string()),
            _ => {
                return Err(ParseError { msg: format!("unreachable error: {}", field) })
                    .map_err(Error::Parse);
            }
        };
    }

    Ok(record)
}

pub fn each_record<F>(reader: &mut LineReader, f: F) -> Result<(), Error>
    where F: Fn(&Record)
{
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Err(err) => return Err(err).map_err(Error::Io),
            Ok(0) => break, // EOF
            Ok(_) => {
                line.pop(); // remove '\n'
                if line.len() == 0 {
                    continue;
                }

                let mut record = Record::new();
                for item in line.split('\t').collect::<Vec<&str>>().into_iter() {
                    let v = item.splitn(2, ':').collect::<Vec<&str>>();
                    match v.len() {
                        0 | 1 => {
                            return Err(ParseError { msg: format!("invalid ltsv item: {}", item) })
                                .map_err(Error::Parse);
                        }
                        2 => record.insert(v[0].to_string(), v[1].to_string()),
                        _ => {
                            return Err(ParseError { msg: format!("unreachable error: {}", item) })
                                .map_err(Error::Parse);
                        }
                    };
                }

                f(&record);
            }
        }
    }
    Ok(())
}

pub fn group_by(reader: &mut LineReader, label: &String) -> Result<FieldGroupCount, Error> {
    let mut group = FieldGroupCount::new();
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Err(err) => return Err(err).map_err(Error::Io),
            Ok(0) => break, // EOF
            Ok(_) => {
                line.pop(); // remove '\n'
                if line.len() == 0 {
                    continue;
                }

                for item in line.split('\t').collect::<Vec<&str>>().into_iter() {
                    let v = item.splitn(2, ':').collect::<Vec<&str>>();
                    match v.len() {
                        0 | 1 => {
                            return Err(ParseError { msg: format!("invalid ltsv item: {}", item) })
                                .map_err(Error::Parse);
                        }
                        2 => {
                            if label != &v[0] {
                                continue;
                            }
                            let count = group.entry(v[1].to_string()).or_insert(0);
                            *count += 1;
                        }
                        _ => {
                            return Err(ParseError { msg: format!("unreachable error: {}", item) })
                                .map_err(Error::Parse);
                        }
                    }
                }
            }
        }
    }
    Ok(group)
}

pub fn order_by(reader: &mut LineReader, label: &String) -> Result<Vec<String>, Error> {
    let mut lines = Vec::new();
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Err(err) => return Err(err).map_err(Error::Io),
            Ok(0) => break, // EOF
            Ok(_) => lines.push(line),
        }
    }
    lines.sort_by(|a, b| {
        let av = match line2record(a) {
            None => "".to_string(),
            Some(record) => {
                match record.get(label) {
                    Some(v) => v.to_string(),
                    None => "".to_string(),
                }
            }
        };
        let bv = match line2record(b) {
            None => "".to_string(),
            Some(record) => {
                match record.get(label) {
                    Some(v) => v.to_string(),
                    None => "".to_string(),
                }
            }
        };
        av.cmp(&bv)
    });

    Ok(lines)
}

fn line2record(line: &String) -> Option<Record> {
    let mut record = Record::new();
    for field in line.split('\t').collect::<Vec<_>>().into_iter() {
        let v = field.splitn(2, ':').collect::<Vec<_>>();
        match v.len() {
            2 => record.insert(v[0].to_string(), v[1].to_string()),
            _ => return None,
        };
    }
    Some(record)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_parse_head() {}
}
