use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use chrono::offset::TimeZone;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use strong_xml::XmlRead;
use structopt::StructOpt;

mod gpx;
mod units;

use crate::units::{Meters, Feet, Miles};

#[derive(Debug, StructOpt)]
struct Args {
    /// Minimum change in elevation (in meters) for a point to contribute to Elevation Gain.
    #[structopt(short = "e", long, parse(try_from_str), default_value = "5")]
    min_elevation_gain: Meters,

    /// Minimum change in distance (in meters) for a point to contribute to Total Distance.
    #[structopt(short = "d", long, parse(try_from_str), default_value = "1")]
    min_distance: Meters,

    /// Minimum time (in seconds) without change in position (per --min_distance) before points do
    /// not contribute to Moving Time.
    #[structopt(short = "t", long, parse(try_from_str = duration_secs), default_value = "10")]
    standstill_time: Duration,

    /// Path to a GPX file to process.
    #[structopt(parse(from_os_str))]
    input_path: PathBuf,
}

fn duration_secs(s: &str) -> Result<Duration> {
    Ok(Duration::seconds(s.parse()?))
}

#[derive(Debug, Clone, Copy)]
struct Point {
    lat: f64,
    lon: f64,
    ele: Option<Meters>,
    time: DateTime<Utc>,
}

impl Point {
    pub fn new(gpx: &gpx::Point<'_>) -> Result<Self> {
        Ok(Self {
            lat: gpx.latitude.parse().context("invalid latitude")?,
            lon: gpx.longitude.parse().context("invalid longitude")?,
            ele: gpx.elevation
                .as_deref()
                .map(Meters::from_str)
                .transpose()
                .context("invalid altitude")?,
            time: Utc.datetime_from_str(&gpx.time, "%+")
                .or_else(|e| {
                    // HACK: try the time with 'Z' appended, for bad GPX files missing timezone
                    // info.
                    Utc.datetime_from_str(&(gpx.time.to_owned() + "Z"), "%+")
                        .map_err(|_| e) // restore original error if this fails
                })
                .context("invalid date/time")?,
        })
    }
}

fn main() -> Result<()> {
    let args = Args::from_args();

    let input = fs::read_to_string(&args.input_path)
        .context("failed to read GPX file to string")?;

    let gpx = gpx::Gpx::from_str(&input)
        .context("failed to parse GPX")?;

    let file_name = gpx.metadata.as_ref().and_then(|m| m.name.as_deref());
            
    println!("input: {:?}", args.input_path);
    println!("parameters:");
    println!("  min elevation gain: {}", args.min_elevation_gain);
    println!("  min distance: {}", args.min_distance);

    let min_moving_speed = args.min_distance.0
        / args.standstill_time.num_milliseconds() as f64 * 1000.;
    println!("  min moving speed = {} m/s", min_moving_speed);

    for (tnum, track) in gpx.tracks.into_iter().enumerate() {
        let name = track.name
            .as_deref()
            .or(file_name)
            .unwrap_or("<unnamed>");

        println!("track {}: {}", tnum + 1, name);

        for (snum, seg) in track.segments.into_iter().enumerate() {
            println!("  segment {}:", snum + 1);

            let mut ele_start = Meters(std::f64::NAN);
            let mut ele_min = Meters(std::f64::MAX);
            let mut ele_max = Meters(std::f64::MIN);
            let mut ele_end = Meters(std::f64::NAN);
            let mut ele_gain = Meters(0.);
            let mut ele_last: Option<Meters> = None;

            let mut dist_total = Meters(0.);
            let mut dist_last: Option<Point> = None;

            let time_start: DateTime<Utc>;
            let mut time_end: DateTime<Utc>;
            let mut time_moving = Duration::seconds(0);

            let points = seg.points.iter().map(Point::new);

            if let Some(point) = points.clone().next().transpose()? {
                time_start = point.time;
                time_end = point.time;
            } else {
                println!("    no points");
                continue;
            }

            for point in points {
                let point = point?;
                if let Some(e) = point.ele {
                    if ele_start.0.is_nan() {
                        ele_start = e;
                    }
                    if ele_min.0 > e.0 {
                        ele_min = e;
                    }
                    if ele_max.0 < e.0 {
                        ele_max = e;
                    }
                    ele_end = e;

                    if let Some(Meters(last)) = ele_last {
                        if (e.0 - last).abs() >= args.min_elevation_gain.0 {
                            if e.0 > last {
                                ele_gain.0 += e.0 - last;
                            }
                            ele_last = Some(e);
                        }
                    } else {
                        ele_last = Some(e);
                    }
                }

                if let Some(last) = dist_last {
                    let (dist, time, speed) = dist_time_speed(&last, &point);
                    if dist.0 >= args.min_distance.0 {
                        dist_total.0 += dist.0;
                        if speed >= min_moving_speed {
                            time_moving = time_moving + time;
                        }
                        dist_last = Some(point);
                    }
                } else {
                    dist_last = Some(point);
                }

                time_end = point.time;
            }

            println!("    starting elevation: {}", Feet(ele_start));
            println!("    ending elevation: {}", Feet(ele_end));
            println!("    min elevation: {}", Feet(ele_min));
            println!("    max elevation: {}", Feet(ele_max));
            println!("    elevation gain: {}", Feet(ele_gain));
            println!("    total distance: {}", Miles(dist_total));
            println!("    total time: {}", fmt_duration(time_end - time_start));
            println!("    moving time: {}", fmt_duration(time_moving));
        }
    }

    Ok(())
}

fn fmt_duration(d: chrono::Duration) -> String {
    let hours = d.num_hours();
    let from_hours = chrono::Duration::hours(hours);
    let mins = (d - from_hours).num_minutes();
    format!("{}:{:02}", hours, mins)
}

fn distance(a: &Point, b: &Point) -> Meters {
    // This is simple great-circle distance.
    // Yes, the Earth is not a sphere, but computing an ellipsoid distance is a pain, and besides,
    // distances are normally given without regard to land elevation, which is a bigger factor than
    // not using an ellipsoid. So this is fine.
    const P: f64 = std::f64::consts::PI / 180.;
    const R: f64 = 6371000.; // approximate earth radius in meters
    let a = 0.5 - ((a.lat - b.lat) * P).cos() / 2.
        + (
            (a.lat * P).cos()
            * (b.lat * P).cos()
            * (1. - ((a.lon - b.lon) * P).cos())
            / 2.
        );
    Meters(2. * R * a.sqrt().asin())
}

fn dist_time_speed(a: &Point, b: &Point) -> (Meters, Duration, f64) {
    let dist = distance(a, b);
    let time = if a.time > b.time { a.time - b.time } else { b.time - a.time };
    let speed = dist.0 / time.num_milliseconds().abs() as f64 * 1000.;
    (dist, time, speed)
}
