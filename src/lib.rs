use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs;

pub fn run(config: Config) {
    let contents = fs::read_to_string(config.filename)
        .expect("Something went wrong reading the file");

    find_lat_lon(&contents);

    // println!("With text:\n{}", contents);
}

pub fn find_lat_lon(contents: &String) -> () {
    let mut reader = Reader::from_str(contents);
    reader.trim_text(true);

    // let mut txt = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name() {
                    b"trkpt" => println!("attributes values: {:?}",
                                        e.attributes().map(|a| a.unwrap().value).collect::<Vec<_>>()),
                    b"ele" => println!("ele"),
                    _ => (),
                }
            },
            Ok(Event::Text(e)) => println!("{}", e.unescape_and_decode(&reader).unwrap()),
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (), // There are several other `Event`s we do not consider here
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

}


pub struct Config {
    filename: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments");
        }

        let filename = args[1].clone();

        Ok(Config { filename })
    }
}