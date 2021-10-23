use csv::Writer;
use kml::types::Geometry::Point;
use kml::{Kml, KmlReader};
use serde::Serialize;
use std::error::Error;
use std::path::Path;

/* home (rather close to home, anyway!) */
const HOME_LATITUDE: f64 = 51.097848;
const HOME_LONGITUDE: f64 = -0.243409;

#[derive(Serialize, Debug)]
struct Walk {
    name: String,
    description: String,
    length: f64,
    latitude: f64,
    longitude: f64,
}

impl Walk {
    fn new(name: String, description: String, length: f64, latitude: f64, longitude: f64) -> Walk {
        Walk {
            name,
            description,
            length,
            latitude,
            longitude,
        }
    }
}

fn parse_fancy_free_walks_map(element: Kml) -> Vec<Walk> {
    let mut walks: Vec<Walk> = Vec::new();

    // https://docs.rs/kml/0.0.6/kml/enum.Kml.html#variant.KmlDocument
    match element {
        /* parse parent elements with children first */
        Kml::KmlDocument(doc) => {
            for child in doc.elements {
                walks.append(&mut parse_fancy_free_walks_map(child));
            }
        }
        Kml::Document { attrs: _, elements } => {
            for child in elements {
                walks.append(&mut parse_fancy_free_walks_map(child));
            }
        }
        Kml::Folder { attrs: _, elements } => {
            for child in elements {
                walks.append(&mut parse_fancy_free_walks_map(child));
            }
        }

        /* ignored elements */
        Kml::Element(_) => {}
        Kml::Style(_) => {}
        Kml::StyleMap(_) => {}

        /* decode placemarks! */
        Kml::Placemark(placemark) => {
            let name: String;
            let mut description: Option<String> = Option::from(String::from(""));
            let mut length: Option<f64> = Option::from(0.0);
            let mut latitude: Option<f64> = Option::from(0.0);
            let mut longitude: Option<f64> = Option::from(0.0);

            // TODO pub_walk: check if includes the word pub (ignorecase)
            // TODO regex /www\.fancyfreewalks\.org.*$/gm to get the URL
            // TODO replace "\'"
            name = placemark.name.unwrap();

            match placemark.description {
                Some(walk_description) => {
                    description = Some(walk_description);

                    // TODO decode walk length from description
                    //  - regex groups /\d+[¼½¾]*/gm to get miles
                    //  - convert the unicode fractional to number
                    //  - find the highest number matched
                    length = Some(0.0);
                }
                _ => {}
            }

            match placemark.geometry {
                Some(geometry) => match geometry {
                    Point(point) => {
                        latitude = Some(point.coord.y);
                        longitude = Some(point.coord.x);

                        // TODO calculate distance out how far away home is from each walk in miles
                    }
                    _ => {}
                },
                _ => {}
            }

            /* add walk into array */
            let walk = Walk::new(
                name,
                description.unwrap(),
                length.unwrap(),
                latitude.unwrap(),
                longitude.unwrap(),
            );
            walks.push(walk);
        }

        /* ignore other elements */
        _ => {}
    };

    walks
}

fn main() -> Result<(), Box<dyn Error>> {
    let kmz_path = Path::new("FancyFreeWalks Summary South East.kmz");
    let mut kmz_reader = KmlReader::<_, f64>::from_kmz_path(kmz_path).unwrap();
    let kmz_data = kmz_reader.read().unwrap();

    // parse the walks from the kmz file
    let walks = parse_fancy_free_walks_map(kmz_data);

    println!("{:#?}", walks);
    // TODO sort first by distance, then length

    // export to csv
    let mut csv = Writer::from_path("out.csv")?;
    for walk in &walks {
        csv.serialize(walk)?;
    }
    csv.flush()?;

    Ok(())
}
