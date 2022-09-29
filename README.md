gpxstat
=======

A tool for summarizing stats of GPX tracks, for bike rides & hikes.

# Usage

```
gpxstat gpxstat [OPTIONS] <INPUT_PATHS>...
```

Arguments:
* `<INPUT_PATHS>...`
  * Path to a GPX file to process

Options:
* `-e`, `--min-elevation-gain <MIN_ELEVATION_GAIN>`
  * Minimum change in elevation (in meters) for a point to contribute to Elevation Gain
  * [default: 10]

* `-d`, `--min-distance <MIN_DISTANCE>`
  * Minimum change in distance (in meters) for a point to contribute to Total Distance
  * [default: 1]

* `-t`, `--standstill-time <STANDSTILL_TIME>`
  * Minimum time (in seconds) without change in position (per --min_distance) before points do not contribute to Moving Time
  * [default: 10]

* `--join-segments`
  * Join all segments in a track as one continuous segment instead of processing them separately

* `--join-tracks`
  * Join all tracks / files together as one continuous track. Implies --join-segments

* `--filter-zero-ele`
  * Filter out points with an elevation of exactly zero.
  * Some software emits GPX points with <ele>0</ele> when it doesn't have a good fix, and you will want to discard these to avoid incorrect elevation data.

* `--filter-ele-below <FILTER_ELE_BELOW>`
  * Filter out points with an elevation below this many meters.
  * Some software emits GPX points with nonsensical low elevations whan it doesn't have a good fix, and you will want to discard these to avoid incorrect elevation data.

* `-h`, `--help`
  * Print help information (use `-h` for a summary)

* `-V`, `--version`
  * Print version information

# Example Output

```
$ gpxstat ~/gpx/2019-06-17_10-39_Mon_Mason_Lake_Mt_Defiance.gpx
input: [
    "/home/wfraser/gpx/2019-06-17_10-39_Mon_Mason_Lake_Mt_Defiance.gpx"
]
parameters:
  min elevation gain: 10.0 m
  min distance: 1.0 m
  min moving speed: 0.1 m/s
file "/home/wfraser/dropbox/gpx/2019-06-17_10-39_Mon_Mason_Lake_Mt_Defiance":
  name: 2019-06-17_10-39_Mon
  creator: OsmAnd 3.3.8
  tracks: 1
  segments: 2
---
track 1: 2019-06-17_10-39_Mon
  segment 1:
    points: 1043
    starting elevation: 2100.8 ft
    ending elevation: 3717.1 ft
    min elevation: 2056.0 ft
    max elevation: 3717.1 ft
    elevation gain: 1627.4 ft
    total distance: 2.7 mi
    total time: 1:37
    moving time: 1:23
  segment 2:
    points: 3484
    starting elevation: 3751.8 ft
    ending elevation: 2079.7 ft
    min elevation: 2077.5 ft
    max elevation: 5516.2 ft
    elevation gain: 2230.1 ft
    total distance: 8.2 mi
    total time: 5:29
    moving time: 4:05
```

# Explanation of Options

Measuring distance of a track is tricky. As described by the
[Coastline Paradox](https://en.wikipedia.org/wiki/Coastline_paradox), the more points you use in
your measurement, the longer the measured distance gets, and it can get quite absurdly long. GPS
data also has slight errors, which contribute to this. For example, consider a person standing
completely still, but recording GPS data. Their position will wander around, within the error range
of the GPS signal. These points obviously shouldn't contribute to the distance.

Therefore, gpxstat doesn't count distance until you're a certain distance away from the last
measured point. This is configured with the `--min-distance` parameter, which defaults to 1 meter.

Elevation Gain has the same problem, but worse, because GPS elevation data is quite poor. Therefore,
gpxstat has a separate parameter for not counting vertical changes until they reach some distance
from the previous point, configurable with the `--min-elevation-gain` parameter, which defaults to
10 meters.

Note that min elevation gain and min distance work together: min distance filtering is done first,
and then min elevation gain. So if you ascend a vertical cliff, this gain won't be counted until you
also move some distance horizontally from the starting point.

Finally, computing Moving Time is tricky because as aforesaid, even a standing still person will see
their GPS position meander around a bit. So gpxstat has a parameter configurable with
`--standstill-time`, where if your position does not move the minimum distance in the specified
amount of time, you're considered standing still, and moving time is not updated. The default is 10
seconds.

# Units

All calculations and parameters are in metric, because that's what GPX uses, but output is converted
to feet and miles, because I live in the United States and that's what units all my maps are in.
Maybe I'll make this configurable, but for me it is not a priority.
