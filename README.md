# Linear Referencing Geocoding Server<br> (Rust Version)

This is a rust implementation of my previous project written in python: https://github.com/thehappycheese/linear_referencing_geocoding_server

This version aims to be orders of magnitude faster.

## Purpose
Runs a REST server on localhost:8025 which will slice
portions of the Main Roads Western Australia road network based on a query parameters `&road=...` number, `&slk_from=...`, `&slk_to=...` and return a GeoJSON (or WKT) feature.
(SLK means Straight Line Kilometer)

The main way that this different from the GeoJSON service available at http://data.wa.gov.au is that LineString geometry will be properly truncated at the requested SLK interval endpoints rather than SQL style filtering of features based on attributes.

## See previous project
For now ... please see the rest of the documentation on my previous python project here https://github.com/thehappycheese/linear_referencing_geocoding_server

I am aiming for a similar featureset.