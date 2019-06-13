use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs;
use std::str;
use term_size;
use chrono::{DateTime, FixedOffset};
use chrono::format::ParseError;

pub fn run(config: Config) {
    let contents = fs::read_to_string(config.filename)
        .expect("Something went wrong reading the file");

    let trackpoints = find_lat_lon(&contents);
    let min_max = find_min_max(trackpoints);

    println!("Max elevation:\n{}", min_max.max_elevation);
    println!("Min elevation:\n{}", min_max.min_elevation);
    println!("With height:\n{}", config.terminal_height);
}

fn find_lat_lon(contents: &String) -> Vec<TrackPoint> {
    let mut reader = Reader::from_str(contents);
    reader.trim_text(true);

    // let mut txt = Vec::new();
    let mut buf = Vec::new();
    let mut state = XmlState::START;
    let mut trackpoints:Vec<TrackPoint> = Vec::new();

    let mut lon:f32 = 0.0;
    let mut lat:f32 = 0.0;
    let mut elevation:f32 = 0.0;
    let mut datetime = String::from("");

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name() {
                    b"trkpt" => {
                        state = XmlState::TRACKPT;
                        let attributes = e.attributes().map(|a| a.unwrap().value).collect::<Vec<_>>();
                        lat = extract_float_from_attribute(&attributes[0]);
                        lon = extract_float_from_attribute(&attributes[1]);
                        // lat = str::from_utf8(&attributes[0]).unwrap();
                        // lat = &lat[1..lat.len() -1 ]
                        // let lon = str::from_utf8(&attributes[0]).unwrap();
                        // println!("{:?} {:?}", lat, lon);
                        // println!("attributes values: {:?}",
                        //                 e.attributes().map(|a| a.unwrap().value).collect::<Vec<_>>())
                    },
                    b"ele" => {
                        state = XmlState::ELEVATION;
                    },
                    b"time" => {
                        // println!("state {:?}", state);
                        match state { 
                            XmlState::START => continue,
                            _ => state = XmlState::TIME
                        };
                    },
                    _ => (),
                }
            },
            Ok(Event::End(ref e)) => {
                match e.name() {
                    b"trkpt" => {
                        state = XmlState::TRACKPT;
                        let trackpoint = TrackPoint::new(elevation, DateTime::parse_from_rfc3339(&datetime).unwrap(), lat, lon);
                        trackpoints.push(trackpoint);
                    },
                    _ => (),
                }
            },
            Ok(Event::Text(e)) => {

                match state {
                    XmlState::ELEVATION => {
                        // println!("elevation {}", e.unescape_and_decode(&reader).unwrap()
                        elevation = e.unescape_and_decode(&reader).unwrap().parse::<f32>().expect("Could no read float");

                    },
                    XmlState::TIME => {
                        // println!("time {}", e.unescape_and_decode(&reader).unwrap())
                        datetime = e.unescape_and_decode(&reader).unwrap();
                        // println!("time: {:?}", datetime);
                    },
                    _ => ()
                }

                // println!("{}", e.unescape_and_decode(&reader).unwrap());
            },
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (), // There are several other `Event`s we do not consider here
        }
        // println!("{}", str::from_utf8(&buf).unwrap());
        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    trackpoints
}

fn extract_float_from_attribute(attribute:&[u8]) -> f32 {
    let attribute = str::from_utf8(attribute).unwrap();
    return attribute.parse().expect("No valid number");
}

fn find_min_max(trackpoints:Vec<TrackPoint>) -> MinMax {

    let mut min_elevation = 1000000.0;
    let mut max_elevation = 0.0;
    let mut min_lat = 0.0;
    let mut max_lat = 0.0;
    let mut min_lon = 0.0;
    let mut max_lon = 0.0;

    for trackpoint in &trackpoints {
        min_elevation = test_min_value(trackpoint.elevation, min_elevation);
        max_elevation = test_max_value(trackpoint.elevation, max_elevation);
        min_lat = test_min_value(trackpoint.lat, min_lat);
        max_lat = test_max_value(trackpoint.lat, max_lat);
        min_lon = test_min_value(trackpoint.lon, min_lon);
        max_lon = test_max_value(trackpoint.lon, max_lon);
    }

    MinMax {
        min_elevation,
        max_elevation,
        min_lat,
        max_lat,
        min_lon,
        max_lon
    }
}

fn test_min_value(test_value:f32, min_value:f32) -> f32 {
    if test_value < min_value {
        return test_value
    }

    min_value
}

fn test_max_value(test_value:f32, max_value:f32) -> f32 {
    if test_value > max_value {
        return test_value
    }

    max_value
}

#[derive(Debug)]
enum XmlState {
    START,
    TRACKPT,
    ELEVATION,
    TIME
}

#[derive(Debug)]
struct TrackPoint {
    elevation: f32,
    datetime: DateTime<FixedOffset>,
    lat: f32,
    lon: f32
}

impl TrackPoint {
    fn new (elevation: f32, datetime: DateTime<FixedOffset>, lat: f32, lon: f32) -> TrackPoint {
        TrackPoint {
            // elevation: elevation.parse::<f32>().expect("Float expected"),
            elevation,
            datetime,
            lat,
            lon
        }
    }
}


pub struct Config {
    filename: String,
    terminal_width: usize,
    terminal_height: usize
}

impl Config {
    pub fn new(args: &[String], ) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments");
        };

        let (terminal_width, terminal_height) = match term_size::dimensions() {
            Some((w, h)) => (w, h),
            None => return Err("Could not read terminal size")
        };

        let filename = args[1].clone();

        Ok(Config { filename, terminal_height, terminal_width })
    }
}

struct MinMax {
    min_elevation: f32,
    max_elevation: f32,
    min_lat: f32,
    max_lat: f32,
    min_lon: f32,
    max_lon: f32
}