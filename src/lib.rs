#[macro_use] extern crate lazy_static;
extern crate regex;

use regex::Regex;

/// A single tile.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct Tile {
    zoom: u8,
    x: u32,
    y: u32,
}

impl Tile {
    /// Constucts a Tile with the following zoom, x and y values. Returns None if the x/y are
    /// invalid for that zoom level, or if the zoom is >= 100.
    pub fn new(zoom: u8, x: u32, y: u32) -> Option<Tile> {
        if zoom >= 100 {
            None
        } else if x < 2u32.pow(zoom as u32) && y < 2u32.pow(zoom as u32) {
            Some(Tile { zoom: zoom, x: x, y: y })
        } else {
            None
        }
    }

    /// Return the zoom of this tile
    pub fn zoom(&self) -> u8 { self.zoom }

    /// Return the X of this tile
    pub fn x(&self) -> u32 { self.x }

    /// Return the Y of this tile
    pub fn y(&self) -> u32 { self.y }

    /// Constucts a Tile with the following zoom, x and y values based on a TMS URL.
    /// Returns None if the TMS url is invalid, or those
    /// # Examples
    /// ```
    /// use slippy_map_tiles::Tile;
    /// let t = Tile::from_tms("/10/547/380.png");
    /// assert_eq!(t, Tile::new(10, 547, 380))
    /// ```
    pub fn from_tms(tms: &str) -> Option<Tile> {
        lazy_static! {
            static ref RE: Regex = Regex::new("/?(?P<zoom>[0-9]?[0-9])/(?P<x>[0-9]{1,10})/(?P<y>[0-9]{1,10})(\\.[a-zA-Z]{3,4})?$").unwrap();
        }

        let caps = RE.captures(tms);
        if caps.is_none() {
            return None;
        }
        let caps = caps.unwrap();

        let zoom = caps.name("zoom");
        let x = caps.name("x");
        let y = caps.name("y");
        if zoom.is_none() || x.is_none() || y.is_none() {
            return None;
        }
        let zoom = zoom.unwrap();
        let x = x.unwrap();
        let y = y.unwrap();

        let zoom = zoom.parse();
        let x = x.parse();
        let y = y.parse();
        if zoom.is_err() || x.is_err() || y.is_err() {
            return None;
        }
        let zoom: u8 = zoom.unwrap();
        let x: u32 = x.unwrap();
        let y: u32 = y.unwrap();

        Tile::new(zoom, x, y)
    }

    // TODO Add from_tc to parse the directory hiearchy so we can turn a filename in to a tile.
    // TODO Add from_ts to parse the directory hiearchy so we can turn a filename in to a tile.

    /// Returns the parent tile for this tile, i.e. the tile at the zoom-1 that this tile is
    /// inside. None if there is no parent, which is at zoom 0.
    pub fn parent(&self) -> Option<Tile> {
        match self.zoom {
            0 => {
                // zoom 0, no parent
                None
            },
            _ => {
                Tile::new(self.zoom-1, self.x/2, self.y/2)
            }
        }
    }

    /// Returns the subtiles (child) tiles for this tile. The 4 tiles at zoom+1 which cover this
    /// tile. Returns None if this is at the maximum permissable zoom level, and hence there are no
    /// subtiles.
    pub fn subtiles(&self) -> Option<[Tile; 4]> {
        match self.zoom {
            std::u8::MAX => {
                None
            },
            _ => {
                let z = self.zoom+1;
                let x = 2*self.x;
                let y = 2*self.y;
                Some([Tile{zoom:z, x:x, y:y}, Tile{zoom:z, x:x+1, y:y}, Tile{zoom:z, x:x, y:y+1}, Tile{zoom:z, x:x+1, y:y+1}])
            }
        }
    }

    /// Iterate on all child tiles of this tile
    pub fn all_subtiles_iter(&self) -> AllSubTilesIterator {
        AllSubTilesIterator::new_from_tile(&self)
    }

    /// Returns the LatLon for the centre of this tile.
    pub fn centre_point(&self) -> LatLon {
        tile_nw_lat_lon(self.zoom, (self.x as f32)+0.5, (self.y as f32)+0.5)
    }

    /// Returns the LatLon for the centre of this tile.
    pub fn center_point(&self) -> LatLon {
        self.centre_point()
    }

    /// Returns the LatLon of the top left, i.e. north west corner, of this tile.
    pub fn nw_corner(&self) -> LatLon {
        tile_nw_lat_lon(self.zoom, (self.x as f32), (self.y as f32))
    }

    /// Returns the LatLon of the top right, i.e. north east corner, of this tile.
    pub fn ne_corner(&self) -> LatLon {
        tile_nw_lat_lon(self.zoom, (self.x as f32)+1.0, (self.y as f32))
    }

    /// Returns the LatLon of the bottom left, i.e. south west corner, of this tile.
    pub fn sw_corner(&self) -> LatLon {
        tile_nw_lat_lon(self.zoom, (self.x as f32), (self.y as f32)+1.0)
    }

    /// Returns the LatLon of the bottom right, i.e. south east corner, of this tile.
    pub fn se_corner(&self) -> LatLon {
        tile_nw_lat_lon(self.zoom, (self.x as f32)+1.0, (self.y as f32)+1.0)
    }

    pub fn top(&self) -> f32 {
        self.nw_corner().lat
    }
    pub fn bottom(&self) -> f32 {
        self.sw_corner().lat
    }
    pub fn left(&self) -> f32 {
        self.nw_corner().lon
    }
    pub fn right(&self) -> f32 {
        self.se_corner().lon
    }

    /// Returns the TC (TileCache) path for storing this tile.
    pub fn tc_path<T: std::fmt::Display>(&self, ext: T) -> String {
        let tc = xy_to_tc(self.x, self.y);
        format!("{}/{}/{}/{}/{}/{}/{}.{}", self.zoom, tc[0], tc[1], tc[2], tc[3], tc[4], tc[5], ext)
    }

    /// Returns the MP (MapProxy) path for storing this tile.
    pub fn mp_path<T: std::fmt::Display>(&self, ext: T) -> String {
        let mp = xy_to_mp(self.x, self.y);
        format!("{}/{}/{}/{}/{}.{}", self.zoom, mp[0], mp[1], mp[2], mp[3], ext)
    }

    /// Returns the TS (TileStash safe) path for storing this tile.
    pub fn ts_path<T: std::fmt::Display>(&self, ext: T) -> String {
        let ts = xy_to_ts(self.x, self.y);
        format!("{}/{}/{}/{}/{}.{}", self.zoom, ts[0], ts[1], ts[2], ts[3], ext)
    }

    /// Returns the ZXY path for storing this tile.
    pub fn zxy_path<T: std::fmt::Display>(&self, ext: T) -> String {
        format!("{}/{}/{}.{}", self.zoom, self.x, self.y, ext)
    }

    /// Returns an iterator that yields all the tiles possible, starting from 0/0/0. Tiles are
    /// generated in a breath first manner, with all zoom 1 tiles before zoom 2 etc.
    pub fn all() -> AllTilesIterator {
        AllTilesIterator{ next_zoom: 0, next_x: 0, next_y: 0}
    }

    /// Returns an iterator that yields all the tiles from zoom 0 down to, and including, all the
    /// tiles at `max_zoom` zoom level.  Tiles are
    /// generated in a breath first manner, with all zoom 1 tiles before zoom 2 etc.
    pub fn all_to_zoom(max_zoom: u8) -> AllTilesToZoomIterator {
        AllTilesToZoomIterator{ max_zoom: max_zoom, next_zoom: 0, next_x: 0, next_y: 0}
    }

    /// The BBox for this tile.
    pub fn bbox(&self) -> BBox {
        let nw = self.nw_corner();
        let se = self.se_corner();

        BBox::new_from_points(&nw, &se)
    }

}

/// Iterates over all the tiles in the world.
pub struct AllTilesIterator {
    next_zoom: u8,
    next_x: u32,
    next_y: u32,
}

impl Iterator for AllTilesIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Tile> {
        let tile = Tile::new(self.next_zoom, self.next_x, self.next_y);
        let max_tile_no = 2u32.pow(self.next_zoom as u32) - 1;
        if self.next_y < max_tile_no {
            self.next_y += 1;
        } else if self.next_x < max_tile_no {
            self.next_x += 1;
            self.next_y = 0;
        } else  if self.next_zoom < std::u8::MAX {
            self.next_zoom += 1;
            self.next_x = 0;
            self.next_y = 0;
        }

        tile
    }
}

/// Iterates over all the tiles from 0/0/0 up to, and including, `max_zoom`.
pub struct AllTilesToZoomIterator {
    max_zoom: u8,
    next_zoom: u8,
    next_x: u32,
    next_y: u32,
}

fn remaining_in_this_zoom(next_zoom: u8, next_x: u32, next_y: u32) -> Option<usize> {
    if next_zoom == 0 && next_x == 0 && next_y == 0 {
        return Some(1);
    }

    let max_tile_no = 2u32.pow(next_zoom as u32);
    let remaining_in_column = max_tile_no - next_y;
    let remaining_in_column = remaining_in_column as usize;
    let remaining_rows = max_tile_no - next_x -1;
    let remaining_rows = remaining_rows as usize;

    let remaining_after_this_column = remaining_rows.checked_mul(max_tile_no as usize);
    if remaining_after_this_column.is_none() {
        return None;
    }
    let remaining_after_this_column = remaining_after_this_column.unwrap();


    remaining_in_column.checked_add(remaining_after_this_column)
}




impl Iterator for AllTilesToZoomIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Tile> {
        if self.next_zoom > self.max_zoom {
            return None;
        }
        let tile = Tile::new(self.next_zoom, self.next_x, self.next_y);
        let max_tile_no = 2u32.pow(self.next_zoom as u32) - 1;
        if self.next_y < max_tile_no {
            self.next_y += 1;
        } else if self.next_x < max_tile_no {
            self.next_x += 1;
            self.next_y = 0;
        } else  if self.next_zoom < std::u8::MAX {
            self.next_zoom += 1;
            self.next_x = 0;
            self.next_y = 0;
        }

        tile
    }


    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.next_zoom > self.max_zoom {
            return (0, Some(0));
        }

        let remaining_in_this_level = remaining_in_this_zoom(self.next_zoom, self.next_x, self.next_y);
        if remaining_in_this_level.is_none() {
            return (std::usize::MAX, None);
        }
        let remaining_in_this_level = remaining_in_this_level.unwrap();


        let mut total: usize = remaining_in_this_level as usize;
        for i in (self.next_zoom+1)..(self.max_zoom+1) {
            let tiles_this_zoom = num_tiles_in_zoom(i);
            if tiles_this_zoom.is_none() {
                return (std::usize::MAX, None);
            }

            let tiles_this_zoom = tiles_this_zoom.unwrap();

            let new_total = total.checked_add(tiles_this_zoom);
            if new_total.is_none() {
                return (std::usize::MAX, None);
            }
            total = new_total.unwrap();

        }

        // If we've got to here, we know how big it is
        (total, Some(total))
    }
}

pub struct AllSubTilesIterator {
    _tiles: Vec<Tile>,
}

impl AllSubTilesIterator {
    pub fn new_from_tile(base_tile: &Tile) -> Self {
        let new_tiles = match base_tile.subtiles() {
            None => Vec::new(),
            Some(t) => vec![t[0], t[1], t[2], t[3]],
        };
        AllSubTilesIterator{ _tiles: new_tiles }
    }

}

impl Iterator for AllSubTilesIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Tile> {
        if self._tiles.len() == 0 {
            return None;
        }
        let next = self._tiles.remove(0);
        if let Some(subtiles) = next.subtiles() {
            self._tiles.extend_from_slice(&subtiles);
        }
        Some(next)
    }
}


fn tile_nw_lat_lon(zoom: u8, x: f32, y: f32) -> LatLon {
    let n: f32 = 2f32.powi(zoom as i32);
    let lon_deg: f32 = (x as f32) / n * 360f32 - 180f32;
    let lat_rad: f32 = ((1f32 - 2f32 * (y as f32) / n) * std::f32::consts::PI).sinh().atan();
    let lat_deg: f32 = lat_rad * 180f32 * std::f32::consts::FRAC_1_PI;

    // FIXME figure out the unwrapping here....
    // Do we always know it's valid?
    LatLon::new(lat_deg, lon_deg).unwrap()
}

/// A single point in the world.
#[derive(PartialEq, Debug, Clone)]
pub struct LatLon {
    lat: f32,
    lon: f32,
}

impl LatLon {
    /// Constructs a LatLon from a given `lat` and `lon`. Returns `None` if the lat or lon is
    /// invalid, e.g. a lat of 100.
    pub fn new(lat: f32, lon: f32) -> Option<LatLon> {
        if lat <= 90f32 && lat >= -90f32 && lon <= 180f32 && lon >= -180f32 {
            Some(LatLon{ lat: lat, lon: lon })
        } else {
            None
        }
    }

    pub fn lat(&self) -> f32 { self.lat }
    pub fn lon(&self) -> f32 { self.lon }
}

/// A Bounding box
#[derive(PartialEq, Debug)]
pub struct BBox {
    top: f32,
    left: f32,
    bottom: f32,
    right: f32,
}

impl BBox {
    /// Construct a new BBox from the given max and min latitude and longitude. Returns `None` if
    /// the lat or lon is invalid, e.g. a lon of 200
    pub fn new(top: f32, left: f32, bottom: f32, right: f32) -> Option<BBox> {
        //let top = if top > bottom { top } else { bottom };
        //let bottom = if top > bottom { bottom } else { top };
        //let left = if right > left { left } else { right };
        //let right = if right > left { right } else { left };

        if top <= 90. && top >= -90. && bottom <= 90. && bottom >= -90.
             && left <= 180. && left >= -180. && right <= 180. && right >= -180. {
             Some(BBox{ top: top, left: left, bottom: bottom, right: right })
        } else {
            None
        }
    }

    /// Given a string like "$MINLON $MINLAT $MAXLON $MAXLAT" parse that into a BBox. Returns None
    /// if there is no match.
    pub fn new_from_string(string: &str) -> Option<BBox> {
        lazy_static! {
            //static ref num_regex: &'static str = r"-?[0-9]{1,3}(\.[0-9]{1,10})?";
            static ref SIMPLE_COPY_SPACE: Regex = Regex::new(r"^(?P<minlon>-?[0-9]{1,3}(\.[0-9]{1,10})?) (?P<minlat>-?[0-9]{1,3}(\.[0-9]{1,10})?) (?P<maxlon>-?[0-9]{1,3}(\.[0-9]{1,10})?) (?P<maxlat>-?[0-9]{1,3}(\.[0-9]{1,10})?)$").unwrap();
            static ref SIMPLE_COPY_COMMA: Regex = Regex::new(r"^(?P<minlon>-?[0-9]{1,3}(\.[0-9]{1,10})?),(?P<minlat>-?[0-9]{1,3}(\.[0-9]{1,10})?),(?P<maxlon>-?[0-9]{1,3}(\.[0-9]{1,10})?),(?P<maxlat>-?[0-9]{1,3}(\.[0-9]{1,10})?)$").unwrap();
        }
        let caps = SIMPLE_COPY_SPACE.captures(string).or_else(|| { SIMPLE_COPY_COMMA.captures(string) } );
        if caps.is_none() {
            return None;
        }
        let caps = caps.unwrap();

        let minlat = caps.name("minlat");
        let maxlat = caps.name("maxlat");
        let minlon = caps.name("minlon");
        let maxlon = caps.name("maxlon");

        if minlat.is_none() || maxlat.is_none() || minlon.is_none() || maxlon.is_none() {
            return None;
        }

        let minlat = minlat.unwrap().parse();
        let maxlat = maxlat.unwrap().parse();
        let minlon = minlon.unwrap().parse();
        let maxlon = maxlon.unwrap().parse();

        if minlat.is_err() || maxlat.is_err() || minlon.is_err() || maxlon.is_err() {
            return None;
        }

        let minlat = minlat.unwrap();
        let maxlat = maxlat.unwrap();
        let minlon = minlon.unwrap();
        let maxlon = maxlon.unwrap();

        BBox::new(minlon, minlat, maxlon, maxlat)

    }

    /// Given two points, return the bounding box specified by those 2 points
    pub fn new_from_points(topleft: &LatLon, bottomright: &LatLon) -> BBox {
        BBox{ top: topleft.lat, left: topleft.lon, bottom: bottomright.lat, right: bottomright.lon }
    }

    /// Given a list of points, return the bouding box specified by those points
    pub fn new_from_points_list(all: &Vec<LatLon>) -> Option<BBox> {
        let mut latmin: f32 = 999f32;
        let mut latmax: f32 = -999f32;
        let mut lonmin: f32 = 999f32;
        let mut lonmax: f32 = -999f32;
        
        for p in all {
            if p.lat() < latmin { latmin = p.lat() }
            if p.lat() > latmax { latmax = p.lat() }
            if p.lon() < lonmin { lonmin = p.lon() }
            if p.lon() > lonmax { lonmax = p.lon() }
        }
        BBox::new(latmax, lonmin, latmin, lonmax)
    }

    /// Return true iff this point is in this bbox
    pub fn contains_point(&self, point: &LatLon) -> bool {
        (point.lat <= self.top && point.lat > self.bottom && point.lon >= self.left && point.lon < self.right)
    }


    /// Returns true iff this bbox and `other` share at least one point
    pub fn overlaps_bbox(&self, other: &BBox) -> bool {
        // FXME check top & left edges
        (self.left < other.right && self.right > other.left && self.top > other.bottom && self.bottom < other.top)
    }

    /// Iterate over all the tiles from z0 onwards that this bbox is in
    pub fn tiles(&self) -> BBoxTilesIterator {
        BBoxTilesIterator::new(&self)
    }

    /// Return the top value of this bbox
    pub fn top(&self) -> f32 { self.top }

    /// Return the bottom value of this bbox
    pub fn bottom(&self) -> f32 { self.bottom }

    /// Return the left value of this bbox
    pub fn left(&self) -> f32 { self.left }

    /// Return the right value of this bbox
    pub fn right(&self) -> f32 { self.right }
}

pub struct BBoxTilesIterator<'a> {
    bbox: &'a BBox,
    tiles: Vec<Tile>,
    tile_index: usize,
}

impl<'a> BBoxTilesIterator<'a> {
    pub fn new(bbox: &'a BBox) -> BBoxTilesIterator<'a> {
        // Everything is in 0/0/0, so start with that.
        BBoxTilesIterator{ bbox: bbox, tiles: vec![Tile::new(0, 0, 0).unwrap()], tile_index: 0 }
    }
}

impl<'a> Iterator for BBoxTilesIterator<'a> {
    type Item = Tile;

    fn next(&mut self) -> Option<Tile> {
        if self.tile_index >= self.tiles.len() {
            // We've sent off all the existing tiles, so start looking at the children
            let mut new_tiles: Vec<Tile> = Vec::with_capacity(self.tiles.len()*4);
            for t in self.tiles.iter() {
                match t.subtiles() {
                    None => { },
                    Some(sub) => {
                        if self.bbox.overlaps_bbox(&sub[0].bbox()) { new_tiles.push(sub[0]); }
                        if self.bbox.overlaps_bbox(&sub[1].bbox()) { new_tiles.push(sub[1]); }
                        if self.bbox.overlaps_bbox(&sub[2].bbox()) { new_tiles.push(sub[2]); }
                        if self.bbox.overlaps_bbox(&sub[3].bbox()) { new_tiles.push(sub[3]); }
                    }
                }
            }

            new_tiles.shrink_to_fit();
            self.tiles = new_tiles;
            self.tile_index = 0;
        }

        let tile = self.tiles[self.tile_index].clone();
        self.tile_index += 1;
        Some(tile)
    }
}


/// Convert x & y to a TileCache (tc) directory parts
fn xy_to_tc(x: u32, y: u32) -> [String; 6] {
    [
        format!("{:03}", x/1_000_000),
        format!("{:03}", (x / 1_000) % 1_000),
        format!("{:03}", x % 1_000),
        format!("{:03}", y/1_000_000),
        format!("{:03}", (y / 1_000) % 1_000),
        format!("{:03}", y % 1_000),
    ]
}

/// Convert x & y to a MapProxy (mp) directory parts
fn xy_to_mp(x: u32, y: u32) -> [String; 4] {
    [
        format!("{:04}", x/10_000),
        format!("{:04}", x % 10_000),
        format!("{:04}", y/10_000),
        format!("{:04}", y % 10_000),
    ]
}

/// Convert x & y to a TileStash (ts) safe directory parts
fn xy_to_ts(x: u32, y: u32) -> [String; 4] {
    [
        format!("{:03}", x/1_000),
        format!("{:03}", x % 1_000),
        format!("{:03}", y/1_000),
        format!("{:03}", y % 1_000),
    ]
}

/// How many times are in this soom level? Returns None if there would be a usize overflow
fn num_tiles_in_zoom(zoom: u8) -> Option<usize> {
    // From experience it looks like you can't calc above zoom >= 6
    if zoom == 0 {
        // Special case of known value
        Some(1)
    } else if zoom <= 5 {
        Some(2u64.pow(2u32.pow(zoom as u32)) as usize)
    } else { 
        None
    }
}

// TODO do mod_tile tile format

mod test {

    #[test]
    fn tc() {
        use super::xy_to_tc;

        let res = xy_to_tc(3, 4);
        assert_eq!(res[0], "000");
        assert_eq!(res[1], "000");
        assert_eq!(res[2], "003");
        assert_eq!(res[3], "000");
        assert_eq!(res[4], "000");
        assert_eq!(res[5], "004");
    }

    #[test]
    fn mp() {
        use super::xy_to_mp;

        let res = xy_to_mp(3, 4);
        assert_eq!(res[0], "0000");
        assert_eq!(res[1], "0003");
        assert_eq!(res[2], "0000");
        assert_eq!(res[3], "0004");
    }

    #[test]
    fn ts() {
        use super::xy_to_ts;

        let res = xy_to_ts(656, 1582);
        assert_eq!(res[0], "000");
        assert_eq!(res[1], "656");
        assert_eq!(res[2], "001");
        assert_eq!(res[3], "582");
    }

    #[test]
    fn tiles_parsing() {
        use super::Tile;

        let tile = Tile::new(1, 5, 5);
        assert!(tile.is_none());

        assert!(Tile::new(4, 8, 9).is_some());

        let tile = Tile::new(1, 0, 0);
        assert!(tile.is_some());

        assert!(Tile::new(100, 0, 0).is_none());
    }

    #[test]
    fn tiles() {
        use super::{Tile, LatLon};
        let tile = Tile::new(1, 0, 0);

        assert!(tile.is_some());
        let tile = tile.unwrap();

        assert_eq!(tile.zoom(), 1);
        assert_eq!(tile.x(), 0);
        assert_eq!(tile.y(), 0);

        let parent = tile.parent();
        assert!(parent.is_some());
        let parent = parent.unwrap();
        assert_eq!(parent, Tile::new(0, 0, 0).unwrap());

        assert_eq!(parent.centre_point(), LatLon::new(0f32, 0f32).unwrap());
        assert_eq!(parent.nw_corner(), LatLon::new(85.05112, -180.0).unwrap());
        assert_eq!(parent.ne_corner(), LatLon::new(85.05112, 180.0).unwrap());
        assert_eq!(parent.sw_corner(), LatLon::new(-85.05112, -180.0).unwrap());
        assert_eq!(parent.se_corner(), LatLon::new(-85.05112, 180.0).unwrap());

        assert_eq!(parent.top(), 85.05112);
        assert_eq!(parent.bottom(), -85.05112);
        assert_eq!(parent.left(), -180.0);
        assert_eq!(parent.right(), 180.0);


        assert_eq!(parent.tc_path("png"), "0/000/000/000/000/000/000.png");
        assert_eq!(parent.mp_path("png"), "0/0000/0000/0000/0000.png");
        assert_eq!(parent.ts_path("png"), "0/000/000/000/000.png");
        assert_eq!(parent.zxy_path("png"), "0/0/0.png");

        let children = parent.subtiles();
        assert_eq!(children.is_none(), false);
        let children: [Tile; 4] = children.unwrap();
        assert_eq!(children[0], Tile::new(1, 0, 0).unwrap());
        assert_eq!(children[0].tc_path("png"), "1/000/000/000/000/000/000.png");

        assert_eq!(children[1], Tile::new(1, 1, 0).unwrap());
        assert_eq!(children[1].tc_path("png"), "1/000/000/001/000/000/000.png");

        assert_eq!(children[2], Tile::new(1, 0, 1).unwrap());
        assert_eq!(children[2].tc_path("png"), "1/000/000/000/000/000/001.png");

        assert_eq!(children[3], Tile::new(1, 1, 1).unwrap());
        assert_eq!(children[3].tc_path("png"), "1/000/000/001/000/000/001.png");
        assert_eq!(children[3].zxy_path("png"), "1/1/1.png")
        
    }

    #[test]
    fn tile_from_tms() {
        use super::Tile;

        fn known_good(tms: &str, zoom: u8, x: u32, y: u32) {
            let tile = Tile::from_tms(tms);
            assert!(tile.is_some());
            let tile = tile.unwrap();
            assert_eq!(tile.zoom, zoom);
            assert_eq!(tile.x, x);
            assert_eq!(tile.y, y);
        }

        fn known_bad(tms: &str) {
            let tile = Tile::from_tms(tms);
            assert!(tile.is_none());
        }

        known_good("/0/0/0.png", 0, 0, 0);
        known_good("/17/1/1234.png", 17, 1, 1234);
        known_good("17/1/1234", 17, 1, 1234);
        known_good("17/1/1234.jpeg", 17, 1, 1234);
        known_good("/17/1/1234.jpeg", 17, 1, 1234);

        known_bad("foo");
        known_bad("/17/1/1234.jpegz");
        known_bad("/17z/1/1234.jpegz");
        known_bad("/0/1/1.png");
        known_bad("/100/1/1.png");

        known_good("http://tile.example.org/17/1/1234", 17, 1, 1234);
        known_good("http://tile.example.org/17/1/1234.png", 17, 1, 1234);
        known_bad("http://tile.example.org/17/1");
        known_bad("http://tile.example.org/17");
        known_bad("http://tile.example.org/17/1/1234.png/foo/bar");
    }

    #[test]
    fn all_tiles() {
        use super::Tile;

        let mut it = Tile::all();

        assert_eq!(it.next(), Tile::new(0, 0, 0));
        assert_eq!(it.next(), Tile::new(1, 0, 0));
        assert_eq!(it.next(), Tile::new(1, 0, 1));
        assert_eq!(it.next(), Tile::new(1, 1, 0));
        assert_eq!(it.next(), Tile::new(1, 1, 1));
        assert_eq!(it.next(), Tile::new(2, 0, 0));
        assert_eq!(it.next(), Tile::new(2, 0, 1));
        assert_eq!(it.next(), Tile::new(2, 0, 2));
        assert_eq!(it.next(), Tile::new(2, 0, 3));
        assert_eq!(it.next(), Tile::new(2, 1, 0));

        let it = Tile::all();
        let z5_tiles: Vec<Tile> = it.skip_while(|t| { t.zoom < 5 }).take(1).collect();
        assert_eq!(z5_tiles[0], Tile::new(5, 0, 0).unwrap());

    }

    #[test]
    fn latlon_create() {
        use super::LatLon;

        let p1 = LatLon::new(54.9, 5.5).unwrap();
        assert_eq!(p1.lat(), 54.9);
        assert_eq!(p1.lon(), 5.5);
    }

    #[test]
    fn bbox_create() {
        use super::{BBox, LatLon};

        // left=5.53 bottom=47.23 right=15.38 top=54.96
        let b1: Option<BBox> = BBox::new(54.9, 5.5, 47.2, 15.38);
        assert!(b1.is_some());
        let b1 = b1.unwrap();
        assert_eq!(b1.top, 54.9);

        let p1 = LatLon::new(54.9, 5.5).unwrap();
        let p2 = LatLon::new(47.2, 15.38).unwrap();
        let b2: BBox = BBox::new_from_points(&p1, &p2);
        assert_eq!(b1, b2);

        let p1 = LatLon::new(54.9, 7.5).unwrap();
        let p2 = LatLon::new(50.2, 15.38).unwrap();
        let p3 = LatLon::new(47.2, 5.5).unwrap();
        let b3: BBox = BBox::new_from_points_list(&vec![p1, p2, p3]).unwrap();
        assert_eq!(b1, b3);
        
    }

    #[test]
    fn bbox_from_string() {
        use super::BBox;

        let bbox = BBox::new_from_string("10 20 30 40");
        assert!(bbox.is_some());
        let bbox = bbox.unwrap();
        assert_eq!(bbox.top(), 10.);
        assert_eq!(bbox.left(), 20.);
        assert_eq!(bbox.bottom(), 30.);
        assert_eq!(bbox.right(), 40.);

        let bbox = BBox::new_from_string("10,20,30,40");
        assert!(bbox.is_some());
        let bbox = bbox.unwrap();
        assert_eq!(bbox.top(), 10.);
        assert_eq!(bbox.left(), 20.);
        assert_eq!(bbox.bottom(), 30.);
        assert_eq!(bbox.right(), 40.);

        let bbox = BBox::new_from_string("71.6,-25.93,35.55,48.9");
        assert!(bbox.is_some());
        let bbox = bbox.unwrap();
        assert_eq!(bbox.top(), 71.6);
        assert_eq!(bbox.left(), -25.93);
        assert_eq!(bbox.bottom(), 35.55);
        assert_eq!(bbox.right(), 48.9);

        fn known_bad(s: &str) {
            assert!(BBox::new_from_string(s).is_none());
        }
        known_bad("foo");
        known_bad("1.1.1.1");
        known_bad("1  1  1  1");
    }

    #[test]
    fn bbox_tile() {
        use super::{BBox, Tile};
        let t = Tile::new(0, 0, 0).unwrap();
        assert_eq!(t.bbox(), BBox::new(85.05112, -180., -85.05112, 180.).unwrap());
    }

    #[test]
    fn bbox_contains_point(){
        use super::{Tile, LatLon};
        // triangle from London, to Bristol to Birmingham
        let tile = Tile::new(7, 63, 42).unwrap();
        let bbox = tile.bbox();
        let point1 = LatLon::new(51.75193, -1.25781).unwrap();  // oxford
        let point2 = LatLon::new(48.7997, 2.4218).unwrap();     // paris

        assert!(bbox.contains_point(&point1));
        assert!(!bbox.contains_point(&point2));

        // only the top and left borders are included in the bbox
        let nw_corner = tile.nw_corner();
        assert!(bbox.contains_point(&nw_corner));

        // Create  new point on the top edge along to the right from the NW corner
        let nw_right = LatLon::new(nw_corner.lat, nw_corner.lon+0.001).unwrap();
        assert!(bbox.contains_point(&nw_right));

        
        assert!(!bbox.contains_point(&tile.sw_corner()));
        assert!(!bbox.contains_point(&tile.ne_corner()));
        assert!(!bbox.contains_point(&tile.se_corner()));
    }

    #[test]
    fn bbox_overlaps() {
        use super::Tile;

        let tile = Tile::new(7, 63, 42).unwrap();
        let parent_tile = tile.parent().unwrap();

        assert!(parent_tile.bbox().overlaps_bbox(&tile.bbox()));

        let tile2 = Tile::new(7, 63, 43).unwrap();
        assert!(!tile.bbox().overlaps_bbox(&tile2.bbox()));
    }

    #[test]
    fn bbox_tile_iter() {
        use super::{BBox, Tile};

        // left=-11.32 bottom=51.11 right=-4.97 top=55.7
        let ie_bbox = BBox::new(55.7, -11.32, 51.11, -4.97).unwrap();
        let mut tiles = ie_bbox.tiles();
        assert_eq!(tiles.next(), Tile::new(0, 0, 0));
        assert_eq!(tiles.next(), Tile::new(1, 0, 0));
        assert_eq!(tiles.next(), Tile::new(2, 1, 1));
        assert_eq!(tiles.next(), Tile::new(3, 3, 2));
        assert_eq!(tiles.next(), Tile::new(4, 7, 5));
        assert_eq!(tiles.next(), Tile::new(5, 14, 10));
        assert_eq!(tiles.next(), Tile::new(5, 15, 10));
        assert_eq!(tiles.next(), Tile::new(6, 29, 20));
        assert_eq!(tiles.next(), Tile::new(6, 29, 21));

    }

    #[test]
    fn test_num_tiles_in_zoom() {
        use super::num_tiles_in_zoom;

        assert_eq!(num_tiles_in_zoom(0), Some(1));
        assert_eq!(num_tiles_in_zoom(1), Some(4));
        assert_eq!(num_tiles_in_zoom(2), Some(16));
        assert_eq!(num_tiles_in_zoom(3), Some(256));
        assert_eq!(num_tiles_in_zoom(4), Some(65_536));
        assert_eq!(num_tiles_in_zoom(5), Some(4_294_967_296));

        assert_eq!(num_tiles_in_zoom(6), None);

        // Can't do these because the integers overflow
        //assert_eq!(num_tiles_in_zoom(17), 17_179_869_184);
        //assert_eq!(num_tiles_in_zoom(18), 68_719_476_736);
        //assert_eq!(num_tiles_in_zoom(19), 274_877_906_944);
    }

    #[test]
    fn test_remaining_in_zoom() {
        use super::remaining_in_this_zoom;

        assert_eq!(remaining_in_this_zoom(0, 0, 0), Some(1));

        assert_eq!(remaining_in_this_zoom(1, 0, 0), Some(4));
        assert_eq!(remaining_in_this_zoom(1, 0, 1), Some(3));
        assert_eq!(remaining_in_this_zoom(1, 1, 0), Some(2));
        assert_eq!(remaining_in_this_zoom(1, 1, 1), Some(1));

        assert_eq!(remaining_in_this_zoom(2, 0, 0), Some(16));
    }

    #[test]
    fn all_tiles_to_zoom_iter() {
        use super::Tile;

        let mut it = Tile::all_to_zoom(1);

        assert_eq!(it.next(), Tile::new(0, 0, 0));
        assert_eq!(it.next(), Tile::new(1, 0, 0));
        assert_eq!(it.next(), Tile::new(1, 0, 1));
        assert_eq!(it.next(), Tile::new(1, 1, 0));
        assert_eq!(it.next(), Tile::new(1, 1, 1));
        assert_eq!(it.next(), None);


        assert_eq!(Tile::all_to_zoom(0).count(), 1);
        assert_eq!(Tile::all_to_zoom(1).count(), 5);
        assert_eq!(Tile::all_to_zoom(2).count(), 21);
        assert_eq!(Tile::all_to_zoom(3).count(), 85);

        assert_eq!(Tile::all_to_zoom(2).last(), Tile::new(2, 3, 3));

        // check the size hints
        assert_eq!(Tile::all_to_zoom(0).size_hint(), (1, Some(1)));

        let mut it = Tile::all_to_zoom(1);
        assert_eq!(it.size_hint(), (5, Some(5)));
        assert!(it.next().is_some());
        assert_eq!(it.size_hint(), (4, Some(4)));
        assert!(it.next().is_some());
        assert_eq!(it.size_hint(), (3, Some(3)));
        assert!(it.next().is_some());
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert!(it.next().is_some());
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert!(it.next().is_some());
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert!(it.next().is_none());

        assert_eq!(Tile::all_to_zoom(2).size_hint(), (21, Some(21)));

        assert_eq!(Tile::all_to_zoom(3).size_hint(), (277, Some(277)));
        assert_eq!(Tile::all_to_zoom(4).size_hint(), (65_813, Some(65_813)));
        assert_eq!(Tile::all_to_zoom(5).size_hint(), (4_295_033_109, Some(4_295_033_109)));
        assert_eq!(Tile::all_to_zoom(6).size_hint(), (18_446_744_073_709_551_615, None));
        assert_eq!(Tile::all_to_zoom(7).size_hint(), (18_446_744_073_709_551_615, None));
        assert_eq!(Tile::all_to_zoom(8).size_hint(), (18_446_744_073_709_551_615, None));
        assert_eq!(Tile::all_to_zoom(9).size_hint(), (18_446_744_073_709_551_615, None));
        assert_eq!(Tile::all_to_zoom(10).size_hint(), (18_446_744_073_709_551_615, None));
        assert_eq!(Tile::all_to_zoom(11).size_hint(), (18_446_744_073_709_551_615, None));
        assert_eq!(Tile::all_to_zoom(12).size_hint(), (18_446_744_073_709_551_615, None));
        assert_eq!(Tile::all_to_zoom(13).size_hint(), (18_446_744_073_709_551_615, None));
        assert_eq!(Tile::all_to_zoom(14).size_hint(), (18_446_744_073_709_551_615, None));
        assert_eq!(Tile::all_to_zoom(15).size_hint(), (18_446_744_073_709_551_615, None));
        assert_eq!(Tile::all_to_zoom(16).size_hint(), (18_446_744_073_709_551_615, None));

    }

    #[test]
    fn all_sub_tiles_iter() {
        use super::Tile;
        let mut it = Tile::new(4, 7, 5).unwrap().all_subtiles_iter();
        assert_eq!(it.next(), Tile::new(5, 14, 10));
        assert_eq!(it.next(), Tile::new(5, 15, 10));
        assert_eq!(it.next(), Tile::new(5, 14, 11));
        assert_eq!(it.next(), Tile::new(5, 15, 11));

        let z10tiles: Vec<Tile> = Tile::new(4, 7, 5).unwrap().all_subtiles_iter().take_while(|t| t.zoom() < 11).filter(|t| t.zoom() == 10).collect();
        assert_eq!(z10tiles.len(), 4096);
        assert_eq!(z10tiles[0].zoom(), 10);
        assert_eq!(z10tiles[z10tiles.len()-1].zoom(), 10);
        
    }
}
