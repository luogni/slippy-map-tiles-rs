<a name="v0.10.0"></a>
## v0.10.0 (2017-02-10)


#### Bug Fixes

*   Set LatLon costructor public ([23553c53](23553c53))

#### Features

*   Support simple Z/X/Y paths ([d20635a0](d20635a0))



<a name="v0.9.0"></a>
## v0.9.0 (2016-12-17)


#### Features

*   Iterate on all child tiles ([b2ce8759](b2ce8759))
*   Tiles can be hashed. Can use as keys in Hashmap ([3ad16113](3ad16113))



<a name="v0.8.0"></a>
## v0.8.0 (2016-09-28)


#### Features

*   Add a TileStash safe output ([48f7d138](48f7d138))

<a name="v0.7.0"></a>
## v0.7.0 (2016-06-14)


#### Features

*   Change licence to GNU GPL v3 (or later)



<a name="v0.6.0"></a>
## v0.6.0 (2016-03-18)


#### Bug Fixes

*   Correct BBox::new_from_string order it's TLBR ([29db0639](29db0639))

#### Features

*   Add BBox::new_from_string ([7553bd1e](7553bd1e))
*   LatLon now have .lat()/.lon() as getters ([e64472a8](e64472a8))
*   Tile::from_tms can now take a full URL ([c2ccdbba](c2ccdbba))



<a name="v0.5.0"></a>
## v0.5.0 (2016-03-17)


#### Features

*   Regex for TMS parsing now statically created ([ce9fa5f4](ce9fa5f4))



<a name="v0.4.0"></a>
## v0.4.0 (2016-03-07)


#### Bug Fixes

*   Make num_tiles_in_zoom more robust & return Option ([d340bb0d](d340bb0d))

#### Features

*   Add size_hint to AllTilesToZoomIterator ([048632b2](048632b2))
*   Add iterator that stops after a certain zoom ([621454e6](621454e6))
*   Add num_tiles_in_zoom function ([b8890c28](b8890c28))



<a name="v0.3.1"></a>
### v0.3.1 (2016-03-07)


#### Bug Fixes

*   Make Tile.{zoom/x/y} pub so we can access them ([6dca0ac3](6dca0ac3))



<a name="v0.3.0"></a>
## v0.3.0 (2016-03-03)


#### Bug Fixes

*   Reject zoom >= 100 ([1127a422](1127a422))

#### Features

*   Add Tile::from_tms to parse a TMS string ([1bf8accb](1bf8accb))



