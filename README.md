
# Linear Referencing Geocoding Server<br> (Rust Version) <!-- omit in toc -->

#### Table of Contents: <!-- omit in toc -->

- [1. Introduction](#1-introduction)
	- [1.1. Purpose](#11-purpose)
	- [1.2. Yes, But What For? (PowerBI Visuals)](#12-yes-but-what-for-powerbi-visuals)
	- [1.3. Previous project](#13-previous-project)
- [2. Usage](#2-usage)
	- [2.1. /query/](#21-query)
	- [2.2. /show/](#22-show)
	- [2.3. /batch/](#23-batch)
- [3. Configuration](#3-configuration)
- [4. Notes](#4-notes)
	- [4.1. Data download and refresh](#41-data-download-and-refresh)
	- [4.2. __static_http folder](#42-__static_http-folder)
	- [4.3. Coordinate Reference System (CRS)](#43-coordinate-reference-system-crs)

## 1. Introduction
### 1.1. Purpose
Runs a REST server on localhost:8025 which will slice
portions of the Main Roads Western Australia road network
based on a query parameters and return a GeoJSON or WKT Feature.

The query parameters `&road=...`, `&slk_from=...`, `&slk_to=...`  are required.
The parameters `&cwy=...` and `&offset=...` are optional.

The main way that this different from the GeoJSON service available at http://data.wa.gov.au 
is that the `LineString` geometry will be properly truncated at the requested SLK interval
 endpoints rather than the simple SQL style filtering used by ESRI rest services.

### 1.2. Yes, But What For? (PowerBI Visuals)

The REST service can be conveniently called from Excel or PowerBI to augment any table of data with a geometry column.
This can then be used to visualise the data in Power BI.

See my related PowerBI custom visual project here: https://github.com/thehappycheese/powerbi-visual-geojson-map-1 <br>
In particular, note the branch called 'live' which aspires to dynamically
georeference data features by relying on some instance of this REST service and the /batch/ feature described below.

### 1.3. Previous project

This is a rust implementation of my previous project written in python: https://github.com/thehappycheese/linear_referencing_geocoding_server

I plan to abandon the python version and maintain this rust version in the future.
This is because
 1. this rust version is several orders of magnitude faster than the python version
 2. python code might seem more portable, but because it depended on geopandas, the python version actually requires a 1GB+ stack of packages to be installed, and on windows a simple `pip install` doesnt even work since precompiled binaries are required for pandas and shapely.
 3. This version is SO fast, when used in batch mode we can 

Note there are some differences in the REST API from the previous python version; For example:
 - The `&cway=...` parameter has been renamed to `&cwy=...` in this version.
 - Only `MultiLineString` features are returned; this means that `&slk_from` should not be equal to `&slk_to` for valid results.
 - etc... please read the new documentation below carefully.

##  2. Usage

### 2.1. /query/ 

By default query mode can be accessed at the following address:

[http://localhost:8025/query/?]()...

The parameters are summarised in the table below:
| Name       | Description                                                                                                                                                                                             | Example Value               | Optional | Default   |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------- | -------- | --------- |
| `road`     | Main Roads Road Number or Local Government Road Number (case sensitive)                                                                                                                                 | `road=H001`                 | No       | -         |
| `slk_from` | Straight Line Kilometer to start the segment                                                                                                                                                            | `slk_from=1.55`             | No       | -         |
| `slk_to`   | Straight Line Kilometer to end the segment                                                                                                                                                              | `slk_to=2.3`                | No       | -         |
| `cwy`      | Filter for the carriageway. Must be some combination of the letters `L`, `R` and `S` (not case sensitive).                                                                                              | `cway=LS` or `cway=RS`      | Yes      | `LRS`     |
| `offset`   | Number of meters to offset the resulting line segments. Large values may not produce any output. Negative values are to the left of the road (in slk direction) and positive values are to the right.   | `offset=4` or `offset=-3.5` | Yes      | `0`       |
| `f`        | Desired response format. Must be `geojson`, `wkt` or `json` (not case sensitive). The `json` format is a nested array format which can be used as the `"coordinates":...` value for a `MultiLineString` | `f=geojson` of `f=wkt`      | Yes      | `geojson` |


The following example fetches the Left and Single carriageway portions of Albany Highway from slk 1km to 2km and offsets them 10 metres to the left of the road. The result is returned in WKT format as a single `MULTILINESTRING` feature:

http://localhost:8025/query/?road=H001&slk_from=1&slk_to=2&cwy=LS&offset=-10&f=wkt

The response looks like this:
```wkt
MULTILINESTRING ((115.88771097361135 -31.967604589743765,115.88776331305647 -31.96753166223028,115.88782456479156 -31.967494045166685,115.88808285746482 -31.967581573012584,115.88842643824691 -31.967706811122067,115.88865106830647 -31.967794863020398,115.88878639134748 -31.967856418305686,115.88961385220324 -31.968270404421514),(115.88961222617614 -31.96826961114868,115.89009682355594 -31.968500014510138),(115.89009709103813 -31.968500142226866,115.8908060320806 -31.96884008302064,115.89130780129135 -31.96906658240955),(115.89129847166095 -31.9690630113479,115.8924861744535 -31.96944832848648),(115.89248599022535 -31.969448268938134,115.89367451654047 -31.969831024006037),(115.89367516412221 -31.96983123526756,115.89489443528633 -31.970234120891217),(115.89489484503049 -31.970234348879462,115.8952199535067 -31.97034351139344,115.89552559070945 -31.970457312501807,115.89572276324779 -31.97054445312055,115.89588899502093 -31.97062796284781,115.89603052161054 -31.9707213605839),(115.89603611706656 -31.97072540301373,115.8961699852627 -31.970830173406412,115.89636973106218 -31.970999046961516,115.89654509709025 -31.971164236270756,115.89708949236724 -31.971705035229636),(115.88735210575929 -31.967327078117492,115.88761740846113 -31.967472091243042),(115.88761495220085 -31.96747075121283,115.88782449298621 -31.967576711138406))
```

### 2.2. /show/

Show mode works the same as /query/ mode except that it shows a map. This is useful for manually checking query results.

http://localhost:8025/show/?road=H001&slk_from=1&slk_to=2&cwy=LS&offset=-10&f=wkt
![show_demo.jpg](readme_img/show_demo.jpg)

Query mode can easily be used from Excel with the `=WEBSERVICE()` formula, or from Power BI using the `=Web.Contents()` function.

### 2.3. /batch/

Batch queries allow bulk georeferecing with minimal network traffic. This mode is much more difficult to use however, and

## 3. Configuration
The applicaiton is hard-coded to look for a file called `config.json` in the current working directory.
If not present the application will `panic!`™ and exit with a non-zero exit code.

The content of the config file should look like this:
```json
{
	"server":"0.0.0.0",
	"port":8025,
	"data_dir":"./__data_dir",
	"data_url":"https://mrgis.mainroads.wa.gov.au/arcgis/rest/services/OpenData/RoadAssets_DataPortal/MapServer/17/query?where=1%3D1&outFields=ROAD,START_SLK,END_SLK,CWY&outSR=4326&f=json",
	"static_dir":"./__static_http"
}
```

TODO: document options in config.


## 4. Notes

### 4.1. Data download and refresh

The applicaiton looks for a file called `output.json.lz4` in the directory specified by the `"data_dir"` field in `config.json`.
If the file or the directory are not found, the application will try to 
delete the specified directory (**and all of it's contents! be careful what directory you specify and where the working directory is!**) 
and download replacement data from the `"data_url"` specified in `config.json`.

To refresh your data, simply manually delete the folder and restart the server.

### 4.2. __static_http folder

The applicaiton is hard-coded to look for a folder called `__static_http` in the current working directory.

This is required for the /show/ function described above.

TODO: im not sure if it crashes if this folder is missing.

### 4.3. Coordinate Reference System (CRS)
The default CRS is EPSG:4326 which is also called WGS84 for eldrich reasons beyond mortal comprehension (see https://spatialreference.org/ref/epsg/wgs-84/)

This software would almost work with any reference system except for the fact that it needs to convert `offset=[metres]` into degrees. It uses a simple approximation for this which will not work if the CRS is changed.

It is therefore important that the input data is loaded in EPSG:4326. I belive the `&outSR=4326` parameter in the following URL accomplishes this; this is the URL from the config.json file.
 https://mrgis.mainroads.wa.gov.au/arcgis/rest/services/OpenData/RoadAssets_DataPortal/MapServer/17/query?where=1%3D1&outFields=ROAD,START_SLK,END_SLK,CWY&outSR=4326&f=json