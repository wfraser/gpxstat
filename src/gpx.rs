use std::borrow::Cow;
use strong_xml::XmlRead;

#[derive(Debug, XmlRead)]
#[xml(tag = "gpx")]
pub struct Gpx<'a> {
    #[xml(child = "metadata")]
    pub metadata: Option<Metadata<'a>>,
    
    //#[xml(child = "wpt")],
    //pub waypoints: Vec<Waypoint<'a>>,

    //#[xml(child = "rte")],
    //pub routes: Vec<Route<'a>>,

    #[xml(child = "trk")]
    pub tracks: Vec<Track<'a>>,

    //#[xml(child = "extensions")],
    //pub extensions: Option<Extensions<'a>>,

    #[xml(attr = "version")]
    pub version: Cow<'a, str>,

    #[xml(attr = "creator")]
    pub creator: Cow<'a, str>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "metadata")]
pub struct Metadata<'a> {
    #[xml(flatten_text = "name")]
    pub name: Option<Cow<'a, str>>,

    //#[xml(flatten_text = "desc")]
    //pub description: Option<Cow<'a, str>>,

    // author
    // copyright
    // link
    
    //#[xml(flatten_text = "time")]
    //pub time: Option<Cow<'a, str>>,

    //#[xml(flatten_text = "keywords")]
    //pub keywords: Option<Cow<'a, str>>,

    // bounds
    // extensions
}

#[derive(Debug, XmlRead)]
#[xml(tag = "trk")]
pub struct Track<'a> {
    #[xml(flatten_text = "name")]
    pub name: Option<Cow<'a, str>>,

    //#[xml(flatten_text = "cmt")]
    //pub comment: Option<Cow<'a, str>>,

    //#[xml(flatten_text = "desc")]
    //pub description: Option<Cow<'a, str>>,

    //#[xml(flatten_text = "src")]
    //pub source: Option<Cow<'a, str>>,

    // link

    //#[xml(flatten_text = "number")]
    //pub number: Option<u64>,

    //#[xml(flatten_text = "type")]
    //pub track_type: Option<Cow<'a, str>>,

    // extensions

    #[xml(child = "trkseg")]
    pub segments: Vec<Segment<'a>>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "trkseg")]
pub struct Segment<'a> {
    #[xml(child = "trkpt")]
    pub points: Vec<Point<'a>>,

    // extensions
}

/*
#[derive(Debug, XmlRead)]
#[xml(tag = "copyright")]
pub struct Copyright<'a> {
    #[xml(flatten_text = "year")]
    pub year: Option<u16>,

    #[xml(flatten_text = "license")]
    pub license_uri: Option<Cow<'a, str>>,

    #[xml(attr = "author")]
    pub author: Cow<'a, str>,
}
*/

// link
// email
// person

#[derive(Debug, XmlRead)]
#[xml(tag = "trkpt")]
pub struct Point<'a> {
    #[xml(flatten_text = "ele")]
    pub elevation: Option<Cow<'a, str>>,

    // GPX 1.1 spec says this is optional:
    //      <xsd:element name="time" type="xsd:dateTime" minOccurs="0">
    // but really, it's not; it's always present because the data is meaningless without it.
    #[xml(flatten_text = "time")]
    pub time: Cow<'a, str>,

    #[xml(attr = "lat")]
    pub latitude: Cow<'a, str>,

    #[xml(attr = "lon")]
    pub longitude: Cow<'a, str>,
}

// bounds
