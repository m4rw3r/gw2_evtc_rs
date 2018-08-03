extern crate clap;
#[macro_use]
extern crate evtc;
extern crate fnv;
extern crate memmap;
extern crate regex;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate zip;

mod json;

use clap::App;
use clap::Arg;

use regex::Regex;

use std::borrow::Cow;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::error::Error;

use zip::ZipArchive;

const JS:  &[u8] = include_bytes!("../../frontend/dist/index.js");
const CSS: &[u8] = include_bytes!("../../frontend/dist/style.css");

fn main() {
    let matches = App::new("evtc_rs")
        .version("0.0.1")
        .author("Martin Wernstål <m4rw3r@gmail.com>")
        .about("Converts Guild Wars 2 evtc log files to JSON/HTML")
        .arg(Arg::with_name("INPUT")
            .help("The file to read (.evtc, .evtc.zip)")
            .required(true)
            .index(1))
        .arg(Arg::with_name("OUTPUT")
            .help("The file to output, defaults to filename without extension + .html")
            .index(2))
        .arg(Arg::with_name("json")
            .short("j")
            .help("If to output raw json instead of HTML"))
        .arg(Arg::with_name("pretty")
            .short("p")
            .help("If to pretty-print the JSON"))
        .get_matches();

    let is_json  = matches.occurrences_of("json") > 0;
    let name     = matches.value_of("INPUT").unwrap().to_string();
    let out_name = matches.value_of("OUTPUT")
        .map(|s| Cow::Owned(s.to_string()))
        .unwrap_or_else(|| Regex::new("\\.evtc(?:\\.zip)?$").unwrap().replace(&name, if is_json { ".json" } else { ".html" }))
        .into_owned();
    let file    = File::open(&name).expect("could not open file");
    let mut out = BufWriter::new(File::create(&out_name).expect("Coult not create file"));
    let pretty  = matches.occurrences_of("pretty") > 0;

    if name.ends_with(".zip") {
        use std::io::Read;

        let mut archive = ZipArchive::new(file).expect("Failed to unzip file");
        let mut file    = archive.by_index(0).expect("Failed to extract first file in archive");
        let mut buffer  = Vec::with_capacity(file.size() as usize);

        file.read_to_end(&mut buffer).expect("Failed to read first file in arcive");

        if is_json {
            json::parse_data(&buffer[..], name, pretty, out).unwrap();
        }
        else {
            wrap_html(&mut out, |out| json::parse_data(&buffer[..], name, pretty, out)).unwrap();
        }
    }
    else {
        let mmap = unsafe { memmap::Mmap::map(&file).expect("Failed to mmap() file") };

        if is_json {
            json::parse_data(&mmap[..], name, pretty, out).unwrap();
        }
        else {
            wrap_html(&mut out, |out| json::parse_data(&mmap[..], name, pretty, out)).unwrap();
        }
    }
}

fn wrap_html<'a, W: Write, E: Error + 'static, F: FnOnce(&mut W) -> Result<(), E>>(out: &'a mut W, f: F) -> Result<usize, Box<Error>> {
    out.write(&b"<html>
  <head>
    <meta charset=\"utf-8\" />
    <style>"[..])?;
    out.write(CSS)?;
    out.write(&b"</style>
  </head>
  <body>
    <div id=\"main\"></div>
    <script type=\"text/javascript\">"[..])?;
    out.write(JS)?;
    out.write(&b"

var evtc_data = "[..])?;

    f(out)?;

    out.write(&b";
evtc_rs(evtc_data, document.getElementById(\"main\"));
</script>
  </body>
</html>"[..]).map_err(From::from)
}