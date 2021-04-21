# Linear Referencing Geocoding Server<br> (Rust Version)

This is a rust implementation of my previous project written in python: https://github.com/thehappycheese/linear_referencing_geocoding_server

This version aims to be orders of magnitude faster.

> Note:
> the `&cway=...` parameter has been renamed to `&cwy=...` in this Rust version.

> Note:
> Only a single query per request is currently working.
> In the python version you could request two chunks of road like this `?road=H001,H002&slk_from=0,2&slk_to=1,3`
> - this is not working in the current version

> Note:
> the current response from the server is neither GeoJSON nor WKT.
> To make it valid geojson all you need is to wrap the response like this:
> 
> `{“type”:”Feature”, “geometry”:{“type”:”MultiLineString”, “coordinates”:` 👉 insert response here 👈 `}}`




## Purpose
Runs a REST server on localhost:8025 which will slice
portions of the Main Roads Western Australia road network
based on a query parameters and return a GeoJSON Feature.

The query parameters `&road=...`, `&slk_from=...`, `&slk_to=...`  are required.
The parameters `&cwy=...` and `&offset=...` are optional.

The main way that this different from the GeoJSON service available at http://data.wa.gov.au 
is that the `LineString` geometry will be properly truncated at the requested SLK interval
 endpoints rather than the simple SQL style filtering used by ESRI rest services.

## See previous project
For now ... please see the rest of the documentation on my previous python project here https://github.com/thehappycheese/linear_referencing_geocoding_server

I am aiming for a similar featureset.