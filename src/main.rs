use csv::Writer;
use geoutils::Location;
use kml::types::Geometry::Point;
use kml::{Kml, KmlReader};
use regex::Regex;
use serde::Serialize;
use std::error::Error;
use std::path::Path;

/* home (rather close to home, anyway!) */
const HOME_LATITUDE: f64 = 51.097848;
const HOME_LONGITUDE: f64 = -0.243409;

#[derive(Serialize, Debug, PartialEq, PartialOrd)]
struct Walk {
    name: String,
    description: String,
    length: f64,
    latitude: f64,
    longitude: f64,
    distance: f64,
}

impl Walk {
    fn new(
        name: String,
        description: String,
        length: f64,
        latitude: f64,
        longitude: f64,
        distance: f64,
    ) -> Walk {
        Walk {
            name,
            description,
            length,
            latitude,
            longitude,
            distance,
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
            let mut length: f64 = 0.0;
            let mut latitude: f64 = 0.0;
            let mut longitude: f64 = 0.0;
            let mut distance_miles: f64 = 0.0;

            // TODO pub_walk: check if includes the word pub (ignorecase)
            // TODO regex /www\.fancyfreewalks\.org.*$/gm to get the URL
            // TODO replace "\'"
            name = placemark.name.unwrap();

            match placemark.description {
                Some(walk_description) => {
                    description = Some(walk_description);

                    /* Decode walk length from description
                     * - convert unicode fractionals to number
                     * - find the highest number matched
                     */
                    let re = Regex::new(r"(\d+[¼½¾]*)").unwrap();
                    let mut len: String;
                    for group in re.captures_iter(description.clone().unwrap().as_str()) {
                        len = String::from(&group[0])
                            .replace("¼", ".25")
                            .replace("½", ".50")
                            .replace("¾", ".75");
                        let tmp = len.parse::<f64>().unwrap();

                        /* keep the largest length */
                        if tmp > length {
                            length = tmp;
                        }
                    }
                }
                _ => {}
            }

            match placemark.geometry {
                Some(geometry) => match geometry {
                    Point(point) => {
                        latitude = point.coord.y;
                        longitude = point.coord.x;

                        /* calculate distance from home to the start of the walk in miles */
                        let home = Location::new(HOME_LATITUDE, HOME_LONGITUDE);
                        let walk_start = Location::new(latitude, longitude);
                        let distance = home.distance_to(&walk_start).unwrap();
                        distance_miles = (distance.meters() * 0.006213712).round() / 10.0;
                    }
                    _ => {}
                },
                _ => {}
            }

            /* add walk into array */
            let walk = Walk::new(
                name,
                description.unwrap(),
                length,
                latitude,
                longitude,
                distance_miles,
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
    let mut walks = parse_fancy_free_walks_map(kmz_data);

    // sort walks by distance
    walks.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

    // print walks
    println!("{:#?}", walks);

    // export to csv
    let mut csv = Writer::from_path("out.csv")?;
    for walk in &walks {
        csv.serialize(walk)?;
    }
    csv.flush()?;

    Ok(())
}
