
# Linear Referencing Geocoding Server <br> (Rust Version) <!-- omit in toc -->

## Table of Contents: <!-- omit in toc -->

- [1. Introduction](#1-introduction)
	- [1.1. Purpose](#11-purpose)
	- [1.2. Doesn't this already exist? (No)](#12-doesnt-this-already-exist-no)
	- [1.3. But What For? (PowerBI Visuals)](#13-but-what-for-powerbi-visuals)
		- [1.3.1. 1.3.1 Icon Map](#131-131-icon-map)
		- [1.3.2. 1.3.2 NickMap (My Custom Visual)](#132-132-nickmap-my-custom-visual)
- [2. Usage](#2-usage)
	- [2.1. Usage - `/query/` Mode](#21-usage---query-mode)
		- [2.1.1. Example `/query/`](#211-example-query)
	- [2.2. Usage - `/show/` Mode](#22-usage---show-mode)
	- [2.3. Usage - `/batch/` Mode](#23-usage---batch-mode)
	- [2.4. Usage - Configuration](#24-usage---configuration)
	- [2.5. Usage - Data Download and Refresh](#25-usage---data-download-and-refresh)
	- [2.6. Usage - Coordinate Reference System (CRS)](#26-usage---coordinate-reference-system-crs)
- [3. Docker](#3-docker)
- [4. Bugs, Issues, Future Features](#4-bugs-issues-future-features)
- [5. Comparison with previous python version](#5-comparison-with-previous-python-version)

## 1. Introduction

### 1.1. Purpose

This application is a REST service that can slice portions of the Western
Australia road network geometry and return either `GeoJSON` or `WKT` features.

To use it, you need to compile and run it, then visit / fetch data from the
server URL (eg `https://localhost:8080/query/?...`), with the URL parameters
`?road=...&slk_from=...&slk_to=...`. Optionally, the parameters `&cwy=...` and
`&offset=...` can be used to select a specific carriageway, and/or offset the
resulting geometry. Use the `&f=...` parameter to select WKT or GeoJSON output
(See usage section below).

### 1.2. Doesn't this already exist? (No)

This REST service will properly truncate the geometry at the requested
`slk_from` and `slk_to` endpoints. This is different from the from the REST
services already available at <http://data.wa.gov.au> which can only filter
records that intersect the requested range according to the row structure of the
underlying storage table.

Each row (in the database storing the Road Network) has a fixed `START_SLK` and
`END_SLK`, contains the road geometry for that section. Each row typically
represents a section of road from one intersection to the next. The REST
services at <http://data.wa.gov.au> can only return whole rows, and are not able
to return only a portion of the road geometry if the requested range partly
intersects with a row's SLK range.

### 1.3. But What For? (PowerBI Visuals)

This application can be conveniently used from Excel using the
`=WEBSERVICE(...)` formula or from PowerBI using the `=Web.Contents(...)`
function to augment any table of data with a geometry column. This can then be
used to visualise the data in Power BI.

#### 1.3.1. 1.3.1 Icon Map

IconMap can be used with any table of data containing a column consisting of WKT.
<https://icon-map.com/>
IconMap is an excellent visual which is finished and polished and is easy to download and use.

#### 1.3.2. 1.3.2 NickMap (My Custom Visual)

See my related PowerBI custom visual project here:
<https://github.com/thehappycheese/powerbi-visual-geojson-map-1> This visual is
A work-in-progress and there is no compiled visual ready for distribution.

If everything goes to plan, this visual might be better than IconMap:

![screenshot](https://github.com/thehappycheese/powerbi-visual-geojson-map-1/raw/main/git_docs/hero_image.jpg)

> **Note:** the `live` branch of this project uses the new `/batch/`
> feature described below to _dynamically_ georeference the data, so that
> augmenting the source data ahead of time is not required :)

![Live Georeferencing field wells](./readme_extras/live_georeferencing.jpg)


## 2. Usage

### 2.1. Usage - `/query/` Mode

By default query mode can be accessed at the following address:

<http://localhost:8025/query/?>...

The parameters are summarised in the table below:
| Name       | Description                                                                                                                                                                                             | Example Value               | Optional | Default   |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------- | -------- | --------- |
| `road`     | Main Roads Road Number or Local Government Road Number (case sensitive)                                                                                                                                 | `road=H001`                 | No       | -         |
| `slk_from` | Straight Line Kilometre to start the segment                                                                                                                                                            | `slk_from=1.55`             | No       | -         |
| `slk_to`   | Straight Line Kilometre to end the segment                                                                                                                                                              | `slk_to=2.3`                | No       | -         |
| `cwy`      | Filter for the carriageway. Must be some combination of the letters `L`, `R` and `S` (not case sensitive).                                                                                              | `cway=LS` or `cway=RS`      | Yes      | `LRS`     |
| `offset`   | Number of meters to offset the resulting line segments. Large values may not produce any output. Negative values are to the left of the road (in slk direction) and positive values are to the right.   | `offset=4` or `offset=-3.5` | Yes      | `0`       |
| `f`        | Desired response format. Must be `geojson`, `wkt` or `json` (not case sensitive). The `json` format is a nested array format which can be used as the `"coordinates":...` value for a `MultiLineString` | `f=geojson` or `f=wkt`      | Yes      | `geojson` |

#### 2.1.1. Example `/query/`

The following example fetches the Left and Single carriageway portions of Albany
Highway from slk 1km to 2km and offsets them 10 metres to the left of the road.
The result is returned in WKT format as a single `MULTILINESTRING` feature:

<http://localhost:8025/query/?road=H001&slk_from=1&slk_to=2&cwy=LS&offset=-10&f=wkt>

The response looks like this:

```wkt
MULTILINESTRING ((115.88771097361135 -31.967604589743765,115.88776331305647 -31.96753166223028,115.88782456479156 -31.967494045166685,115.88808285746482 -31.967581573012584,115.88842643824691 -31.967706811122067,115.88865106830647 -31.967794863020398,115.88878639134748 -31.967856418305686,115.88961385220324 -31.968270404421514),(115.88961222617614 -31.96826961114868,115.89009682355594 -31.968500014510138),(115.89009709103813 -31.968500142226866,115.8908060320806 -31.96884008302064,115.89130780129135 -31.96906658240955),(115.89129847166095 -31.9690630113479,115.8924861744535 -31.96944832848648),(115.89248599022535 -31.969448268938134,115.89367451654047 -31.969831024006037),(115.89367516412221 -31.96983123526756,115.89489443528633 -31.970234120891217),(115.89489484503049 -31.970234348879462,115.8952199535067 -31.97034351139344,115.89552559070945 -31.970457312501807,115.89572276324779 -31.97054445312055,115.89588899502093 -31.97062796284781,115.89603052161054 -31.9707213605839),(115.89603611706656 -31.97072540301373,115.8961699852627 -31.970830173406412,115.89636973106218 -31.970999046961516,115.89654509709025 -31.971164236270756,115.89708949236724 -31.971705035229636),(115.88735210575929 -31.967327078117492,115.88761740846113 -31.967472091243042),(115.88761495220085 -31.96747075121283,115.88782449298621 -31.967576711138406))
```

### 2.2. Usage - `/show/` Mode

Show mode works the same as `/query/` mode except that instead of returning raw
data, it displays an interactive map when viewed in the browser. This is useful
to confirm that queries are working as intended.

<http://localhost:8025/show/?road=H001&slk_from=1&slk_to=2&cwy=LS&offset=-10&f=wkt>
![show_demo.jpg](./readme_extras/show_demo.jpg)

Query mode can easily be used from Excel with the `=WEBSERVICE()` formula, or
from Power BI using the `=Web.Contents()` function.

### 2.3. Usage - `/batch/` Mode

`/batch/` mode is an advanced feature that allows ultra-fast georeferecing with minimal network traffic.
This mode exists to integrate with PowerBI custom visuals.

This mode expects a `POST` request to <http://localhost:8025/batch/> by default.
The body of the request must be binary data consisting of a series of frames
with the format shown in the table below. Any number of frames can be packed
into a single request.

| Byte Length | Type                  | Value                                  |
| ----------- | --------------------- | -------------------------------------- |
| 1           | Uint8                 | Number of bytes in `road` string `x`   |
| `x`         | Utf8 String           | `road` number                          |
| 4           | Float32 Little Endian | `slk_from` in kilometres               |
| 4           | Float32 Little Endian | `slk_to` in kilometres                 |
| 4           | Float32 Little Endian | `offset` in metres                     |
| 1           | Uint8                 | `cwy` (carriageways) (see table below) |

`cwy` is packed into a Uint8 as follows:
| `cwy` | Carriageway    | Binary          | Decimal |
| ----- | -------------- | --------------- | ------- |
| `R`   | Right only     | `0b0000_0001`   | 1       |
| `S`   | Single only    | `0b0000_0010`   | 2       |
| `RS`  | Right & Single | `0b0000_0011`   | 3       |
| `L`   | Left only      | `0b0000_0100`   | 4       |
| `LR`  | Left & Right   | `0b0000_0101`   | 5       |
| `LS`  | Left & Single  | `0b0000_0110`   | 6       |
| `LRS` | All            | any other value |         |

Th `/batch/` mode is impossible to use from Excel, and would require a very
complex PowerBI M script to work. It is intended to be used by other software
(eg. The custom PowerBI visual linked above).

There is an example batch query implementation in `__static_http/main.js`
however a simplified version is shown below:

```javascript
// =========== Helper functions: ===========
let CWY_LOOKUP = {
    L: 0b0000_0100,	R: 0b0000_0001,	S: 0b0000_0010,	LR: 0b0000_0101,
    LS: 0b0000_0110, RS: 0b0000_0011, LRS: 0b0000_0111
}
function binary_encode_request(road, slk_from, slk_to, offset, cwy) {
    let text_encoder = new TextEncoder();
    let road_bytes = text_encoder.encode(road);
    
    let buffer = new ArrayBuffer(1 + road_bytes.length + 4 + 4 + 4 + 1);

    let road_name_chunk = new Uint8Array(buffer, 0, 1 + road_bytes.length);
    road_name_chunk[0] = road_bytes.length;
    road_name_chunk.set(road_bytes, 1);

    let data_view = new DataView(buffer, 1 + road_bytes.length);
    data_view.setFloat32(0, slk_from, true) // LITTLE ENDIAN
    data_view.setFloat32(4, slk_to, true) // LITTLE ENDIAN
    data_view.setFloat32(8, offset, true) // LITTLE ENDIAN
    data_view.setUint8(12, CWY_LOOKUP[cwy.toUpperCase()] ?? 0); // use 0 if lookup fails

    return new Uint8Array(buffer);
}

// =========== Perform Batch Query ============

// Build batch query:
let request_body_parts = [
    binary_encode_request("H001", 1.0, 1.1, 0, "LRS"),
    binary_encode_request("H001", 3.0, 3.2, 0, "LS"),
    binary_encode_request("H002", 4.1, 4.2, 20, "LS")
];

// Find total query length in bytes:
let request_body_byte_length = request_body_parts.reduce(
    (total, item) => total + item.byteLength, 
    0 // initial value of total
);

// Pack all queries into a single byte array:
let request_body = new Uint8Array(request_body_byte_length);
request_body_parts.reduce((offset, byte_array) => {
        request_body.set(byte_array, offset);
        return offset + byte_array.byteLength;
    },
    0 // initial offset
)

// Send the request to the server
fetch("http://localhost:8025/batch/", {
        method: "POST",
        body: request_body
    }
)
.then(response => response.json())
.then(json => {
    let features = [];
    for (multi_line_string_coordinates of json) {
        if (multi_line_string_coordinates == null) continue;
        features.push({
            type: "Feature",
            geometry: {
                type: "MultiLineString",
                coordinates: multi_line_string_coordinates
            }
        });
    }
    let result = {
        type:"FeatureCollection",
        features
    }
    // DONE! 'result' is now a standard GeoJSON feature collection.
    //  ready to be used in a map or for other purposes.
    console.log(JSON.stringify(result));
});
```

The output of the script above is shown below:

```json
{
    "type":"FeatureCollection",
    "features":[
        {"type":"Feature","geometry":{"type":"MultiLineString","coordinates":[[[115.88778395521496,-31.967656968008896],[115.88783629466008,-31.96758404049541],[115.88784873900369,-31.967580563399395],[115.88804803150612,-31.96766437971354],[115.88831038467791,-31.967760094769094]],[[115.88778395521496,-31.967656968008896],[115.8877412235351,-31.967713333565143],[115.887742230063,-31.967723398843077],[115.8879828817074,-31.96785406445076]],[[115.8879828817074,-31.96785406445076],[115.88808710308484,-31.967912260057517]],[[115.8873090202252,-31.96740590345527],[115.88757432292704,-31.96755091658082]],[[115.8875744144296,-31.96755091658082],[115.88778395521496,-31.967656876506396]]]}},
        {"type":"Feature","geometry":{"type":"MultiLineString","coordinates":[[[115.90371854775746,-31.97874013724704],[115.90387696204294,-31.978944811166286],[115.90481214543112,-31.980341530482082]]]}},
        {"type":"Feature","geometry":{"type":"MultiLineString","coordinates":[[[115.79676132053224,-32.08478062909291],[115.79696805522983,-32.08570245636619]]]}}
    ]
}
```

### 2.4. Usage - Configuration

To load configuration, the application will take the following steps:

1. Check to see if a config file has been specified on the command line using the `--config` option:

```shell
nicklinref.exe --config ./config.json
```

2. If no `--config` was specified, then load the hard-coded default options. The defaults are shown below:

```json
{
	"NLR_ADDR":"0.0.0.0",
	"NLR_PORT":8080,
	"NLR_DATA_FILE":"./data/data.json.lz4",
	"NLR_DATA_SOURCE_URL":"https://mrgis.mainroads.wa.gov.au/arcgis/rest/services/OpenData/RoadAssets_DataPortal/MapServer/17/query?where=1%3D1&outFields=ROAD,START_SLK,END_SLK,CWY&outSR=4326&f=json",
	"NLR_STATIC_HTTP":"./__static_http",
	"NLR_CERT_PATH":"./certs/public.crt",
	"NLR_PRIVATE_KEY_PATH":"./certs/decrypted_private.key"
}
```

3. Override any options with any available environment variables with matching names.

The following table describes the configuration options in more detail:

| Property               | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| ---------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `NLR_ADDR`             | A string containing an IPV4 address. 0.0.0.0 will allow requests from anywhere. Using 127.0.0.1 wil limit traffic to your own machine for testing purposes.                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `NLR_PORT`             | A port number.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `NLR_DATA_FILE`        | The filename of the data cached from `NLR_DATA_SOURCE_URL`. **The directory must already exist**. If the file does not already exist then it will be created and fresh data will be downloaded.                                                                                                                                                                                                                                                                                                                                                                                    |
| `NLR_DATA_SOURCE_URL`  | This is the ArcGIS REST service where the road network is downloaded from. It is assumed that multiple requests are needed and the `&resultOffset=...` parameter is used to repeatedly fetch more data. Only certain fields are fetched `outFields=ROAD,START_SLK,END_SLK,CWY` and the output spatial reference is specified `&outSR=4326`. ESRI's own json format (`&f=json`) is expected because `&f=geojson` does not seem to work properly. Also note that currently the field names `ROAD`, `START_SLK`, `END_SLK`, `CWY` are hard-coded and must exist on the incoming data. |
| `NLR_STATIC_HTTP`      | Used by the `/show/` feature to display an interactive map. the directory specified by this config option should exist or I think the application will crash on startup. The directory can probably be empty though if it is not required. The `__static_http` folder in this repo contains the files required.                                                                                                                                                                                                                                                                    |
| `NLR_CERT_PATH`        | A certificate (public key) Used to serve HTTPS.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `NLR_PRIVATE_KEY_PATH` | A unencrypted private key used to serve HTTPS.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |

> Note:
>
> For testing purposes A suitable self signed certificate and key can be
> generated like this:
>
> `openssl req -new -newkey rsa:4096 -days 365 -nodes -x509 -subj "/C=AU/ST=SomeState/L=SomeTown/O=You/CN=localhost" -keyout decrypted_private.key -out public.crt`
>
> This will cause a nasty security notice when accessed via the browser.

### 2.5. Usage - Data Download and Refresh

To refresh your data, simply manually delete the file specified by the
`NLR_DATA_FILE` option and restart the application. Fresh data will be
downloaded.

### 2.6. Usage - Coordinate Reference System (CRS)

The default CRS is EPSG:4326 which is also called WGS84 for eldritch reasons
beyond mortal comprehension (see <https://spatialreference.org/ref/epsg/wgs-84/>)

This software would almost work with any reference system except for the fact
that it needs to convert `&offset=[metres]` into degrees. It uses a simple
approximation for this which will not work if the CRS is changed.

It is therefore important that the input data is loaded in EPSG:4326. I believe
the `&outSR=4326` parameter in the following URL accomplishes this.

## 3. Docker

You can build a version using the dockerfile that I put here:
<https://github.com/thehappycheese/nicklinref_rust/wiki/Dockerfile>

> **Note:**
>
> Currently the dockerfile generates a self-signed certificate
> as part of the build to enable HTTPS.
>
> This is for testing purposes only and causes a warning when
> visiting the `/query/` or `/show/` page in the browser.
>
> The `/batch/` function cannot be used by the PowerBI custom visual
> i am working on unless the user manually adds the certificates 
> to their own trusted root certificate store.

## 4. Bugs, Issues, Future Features

- The current version serves all traffic over HTTPS. However, only the `/batch/` mode used by my custom PowerBI visual requires this HTTPS. I could potentially serve them separately if that helps.
- I have discovered `FastCGI`; this is the exact description of what I need.
- Prevent crash when missing `static_dir`
- Allow multiple features to be requested in `/query/` mode using the POST
  method with a json request body
- Allow requests where `slk_from` == `slk_to`, return `Point` and / or
  `MultiPoint` features in this case.
  - And / or, create a new path `/query_points/` which expects point requests only

## 5. Comparison with previous python version

This is a rust implementation of my previous project written in python:
<https://github.com/thehappycheese/linear_referencing_geocoding_server>

Note there are some differences in the REST API from the previous python
version; For example:

- The `&cway=...` parameter has been renamed to `&cwy=...` in this version.
- Only `MultiLineString` features are returned; this means that `&slk_from`
  should not be equal to `&slk_to` for valid results.
- etc... please read the new documentation below carefully.

I plan to abandon the python version and maintain this rust version in the
future. This is because

| Issue        | Python                                                                                                                                                                                                                   | Rust                                                                                                                                                                                                                                                                                                                                                              |
| ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Speed        | Slow. 5 minutes + to georeference 12000 items.                                                                                                                                                                           | **Super fast**. The network connection is the bottleneck. Less than 30 seconds to georeference 12000 rows when used one row at a time, effectively instantaneous in `/batch/` mode.                                                                                                                                                                               |
| RAM          | Over 700Mb                                                                                                                                                                                                               | 70Mb (180Mb peek)                                                                                                                                                                                                                                                                                                                                                 |
| Startup time | Very slow.                                                                                                                                                                                                               | Also a bit slow (due to reading the input data and decompressing it in memory on every startup) but still much faster than the python version.                                                                                                                                                                                                                    |
| Dependencies | Depends on geopandas therefore it actually requires a 1GB+ stack of packages required by geopandas. On windows a simple `pip install` doesn't even work since pre-compiled binaries are required for pandas and shapely. | Needs to be compiled for the target platform, It produces a single statically linked binary which is very easy to distribute. Due to the magic of Cargo and the rust toolchain, it is incredibly easy to clone from github and compile without complicated setup. The only complication I have experienced on Debian was the need to manually install libssl-dev. |
| Deployment   | Requires a lot of setup to run in cloud environment... heavy resource requirements                                                                                                                                       | Using multi stage docker build it could probably be squished into a container that is about 50Mb in size. It shares some problems with the python version; it is slow to start, and expects to be always-running. This always running problem forfeits the possible cost benefits of running it on Azure Functions or similar.                                    |
