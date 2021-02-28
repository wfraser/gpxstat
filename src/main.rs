use anyhow::{bail, Context, Result};
use chrono::{DateTime, Duration, Utc};
use chrono::offset::TimeZone;
use std::collections::BTreeMap;
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
    #[structopt(short = "e", long, parse(try_from_str), default_value = "10")]
    min_elevation_gain: Meters,

    /// Minimum change in distance (in meters) for a point to contribute to Total Distance.
    #[structopt(short = "d", long, parse(try_from_str), default_value = "1")]
    min_distance: Meters,

    /// Minimum time (in seconds) without change in position (per --min_distance) before points do
    /// not contribute to Moving Time.
    #[structopt(short = "t", long, parse(try_from_str = duration_secs), default_value = "10")]
    standstill_time: Duration,

    /// Join all segments in a track as one continuous segment instead of processing them
    /// separately.
    #[structopt(long)]
    join_segments: bool,

    /// Join all tracks / files together as one continuous track. Implies --join-segments.
    #[structopt(long)]
    join_tracks: bool,

    /// Path to a GPX file to process.
    #[structopt(parse(from_os_str), required(true))]
    input_paths: Vec<PathBuf>,
}

fn duration_secs(s: &str) -> Result<Duration> {
    Ok(Duration::seconds(s.parse()?))
}

#[derive(Debug)]
struct Track {
    name: String,
    segments: Vec<Segment>,
}

#[derive(Debug)]
struct Segment {
    points: Vec<Point>,
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
            time: DateTime::parse_from_str(&gpx.time, "%+")
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|e| {
                    // HACK: try the time with 'Z' appended, for bad GPX files missing timezone
                    // info.
                    Utc.datetime_from_str(&(gpx.time.to_owned() + "Z"), "%+")
                        .map_err(|_| e) // restore original error if this fails
                })
                .with_context(|| format!("invalid date/time {:?}", gpx.time))?,
        })
    }
}

fn main() -> Result<()> {
    let args = Args::from_args();

    println!("input: {:#?}", args.input_paths);
    println!("parameters:");
    println!("  min elevation gain: {}", args.min_elevation_gain);
    println!("  min distance: {}", args.min_distance);

    let min_moving_speed = args.min_distance.0
        / args.standstill_time.num_milliseconds() as f64 * 1000.;
    println!("  min moving speed: {} m/s", min_moving_speed);

    let mut tracks = Vec::<Track>::with_capacity(args.input_paths.len());
    for path in args.input_paths {
        let input = fs::read_to_string(&path)
            .with_context(|| format!("failed to read GPX file to string: {:?}", path))?;

        let gpx = gpx::Gpx::from_str(&input)
            .with_context(|| format!("failed to parse GPX file {:?}", path))?;

        let file_name = gpx.metadata.as_ref().and_then(|m| m.name.as_deref());

        println!("file {:?}:", path);
        println!("  name: {}", file_name.unwrap_or("<unnamed>"));
        println!("  creator: {}", gpx.creator);
        println!("  tracks: {}", gpx.tracks.len());
        println!("  segments: {}", gpx.tracks.iter().map(|t| t.segments.len()).sum::<usize>());

        for gpx_track in gpx.tracks.into_iter() {
            let track = if args.join_tracks {
                match tracks.get_mut(0) {
                    Some(t) => t,
                    None => {
                        tracks.push(Track {
                            name: file_name.unwrap_or("<unnamed>").to_owned(),
                            segments: vec![],
                        });
                        tracks.last_mut().unwrap()
                    }
                }
            } else {
                tracks.push(Track {
                    name: gpx_track.name
                        .as_deref()
                        .or(file_name)
                        .unwrap_or("<unnamed>")
                        .to_owned(),
                    segments: Vec::with_capacity(gpx_track.segments.len()),
                });
                tracks.last_mut().unwrap()
            };

            for gpx_seg in gpx_track.segments {
                let segment = if args.join_tracks || args.join_segments {
                    match track.segments.get_mut(0) {
                        Some(s) => s,
                        None => {
                            track.segments.push(Segment {
                                points: Vec::with_capacity(gpx_seg.points.len()),
                            });
                            track.segments.last_mut().unwrap()
                        }
                    }
                } else {
                    track.segments.push(Segment {
                        points: Vec::with_capacity(gpx_seg.points.len()),
                    });
                    track.segments.last_mut().unwrap()
                };

                for gpx_point in gpx_seg.points {
                    segment.points.push(
                        Point::new(&gpx_point)?
                    );
                }
            }
        }
    }

    println!("---");

    for (tnum, track) in tracks.into_iter().enumerate() {
        println!("track {}: {}", tnum + 1, track.name);

        if args.join_segments {
            println!("  (all segments joined)");
        }

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

            if let Some(point) = seg.points.get(0) {
                time_start = point.time;
                time_end = point.time;
            } else {
                println!("    no points");
                continue;
            }

            println!("    points: {}", seg.points.len());

            let mut last_time = None;
            let mut time_deltas = vec![];

            for point in seg.points {
                time_end = point.time;

                if let Some(t) = last_time {
                    if point.time < t {
                        bail!("time went backwards? {:?} -> {:?}", t, point.time);
                    }
                    let delta: chrono::Duration = point.time - t;
                    let stddelta = delta.to_std()
                        .with_context(|| format!("duration out of range: {:?}", delta))?;
                    time_deltas.push(stddelta);
                }
                last_time = Some(point.time);

                // Distance smoothing.
                let mut use_point = true;
                if let Some(last) = dist_last {
                    let (dist, time, speed) = dist_time_speed(&last, &point);
                    if dist.0 >= args.min_distance.0 {
                        dist_total.0 += dist.0;
                        if speed >= min_moving_speed {
                            time_moving = time_moving + time;
                        }
                        dist_last = Some(point);
                    } else {
                        use_point = false;
                    }
                } else {
                    dist_last = Some(point);
                }

                // Elevation smoothing.
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
                        if use_point && (e.0 - last).abs() >= args.min_elevation_gain.0 {
                            if e.0 > last {
                                ele_gain.0 += e.0 - last;
                            }
                            ele_last = Some(e);
                        }
                    } else {
                        ele_last = Some(e);
                    }
                }
            }

            time_deltas.sort();
            let mean = time_deltas.iter()
                .sum::<std::time::Duration>() / time_deltas.len() as u32;
            let median = time_deltas[time_deltas.len() / 2];
            let mut freq = BTreeMap::new();
            for d in &time_deltas {
                *freq.entry(d).or_insert(0) += 1;
            }
            let mode = freq.iter().max_by(|(_, count1), (_, count2)| count1.cmp(count2)).unwrap().0;
            println!("    point time deltas:");
            println!("        mean:   {:?}", mean);
            println!("        median: {:?}", median);
            println!("        mode:   {:?}", mode);

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
