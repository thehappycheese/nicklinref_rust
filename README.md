
# NickLinRef <!-- omit in toc -->

NickLinRef, or just 'LinRef', is designed to accurately extract portions of the
[Western Australia Road Network](https://catalogue.data.wa.gov.au/dataset/mrwa-road-network)
geometry. You can query this service to receive the road segment data in either
`GeoJSON` or `WKT` formats. It easily integrates with Excel via the
[`=WEBSERVICE()` formula](https://support.microsoft.com/en-us/office/webservice-function-0546a35a-ecc6-4739-aed7-c0b7ce1562c4).
or PowerBI using the
[`Web.Contents` function](https://learn.microsoft.com/en-us/powerquery-m/web-contents)
and is used by my Custom PowerBI Visual
[NickMapBI](https://github.com/thehappycheese/nickmap-bi) to pull dynamically
create geometry.

![NickMapBI](https://github.com/thehappycheese/nickmap-bi/blob/main/readme_extras/v4.2.0-screenshot.png)

## Table of Contents: <!-- omit in toc -->

- [1. Getting Started](#1-getting-started)
- [2. Distinguishing Features](#2-distinguishing-features)
- [3. Usage - GET - Text Response (GeoJSON / WKT / JSON / LATLON)](#3-usage---get---text-response-geojson--wkt--json--latlon)
  - [3.1. Example - Get Lines - WKT](#31-example---get-lines---wkt)
  - [3.2. Example - Get Points - GeoJSON](#32-example---get-points---geojson)
  - [3.3. Example - Get Point - Simple Latitude Longitude](#33-example---get-point---simple-latitude-longitude)
- [4. Usage - Browser - `/show/` Page](#4-usage---browser---show-page)
- [5. Usage - POST - `/batch/` Route for app Integration](#5-usage---post---batch-route-for-app-integration)
- [6. Notes](#6-notes)
  - [6.1. SLK, True Distance and Chainage](#61-slk-true-distance-and-chainage)
  - [6.2. Supported Network Types](#62-supported-network-types)
  - [6.3. Coordinate Reference System (CRS)](#63-coordinate-reference-system-crs)
- [7. Running the Server Yourself](#7-running-the-server-yourself)
  - [7.1. Installation](#71-installation)
  - [7.2. Compilation](#72-compilation)
    - [7.2.1. Compiling for Ubuntu / Debian](#721-compiling-for-ubuntu--debian)
    - [7.2.2. Compiling for Windows](#722-compiling-for-windows)
  - [7.3. Configuration](#73-configuration)
    - [7.3.1. Configuration - Command Line Interface (CLI)](#731-configuration---command-line-interface-cli)
    - [7.3.2. Configuration - Environment Variables](#732-configuration---environment-variables)
  - [7.4. Data Download and Refresh](#74-data-download-and-refresh)
- [8. Related Projects](#8-related-projects)
  - [8.1. Megalinref](#81-megalinref)
  - [8.2. NickMapBI](#82-nickmapbi)
  - [8.3. Python version (Predecessor to this Rust version)](#83-python-version-predecessor-to-this-rust-version)

## 1. Getting Started

To get started, download and extract a pre-compiled version of this application
from the [releases](/releases) page. Run `nicklinref.exe` and navigate to
<http://localhost:8080/?road=H001&slk_from=1.5&slk_to=3> test the server is
working.

Next See the [Usage](#3-usage---get---text-response-geojson--wkt--json--latlon)
section below.

## 2. Distinguishing Features

NickLinRef is different from existing web services because it can accurately
truncate road centreline geometry at the requested `slk_from` and `slk_to`
endpoints. It can also correctly interpolate to find a Latitude Longitude point
at a requested `slk`. These features are not available in other existing APIs
such as those available from <https://data.wa.gov.au>.

- Superior performance under high traffic (for example if users are calling it
  from a large excel sheet using `=WEBSERVICE()`)
- Low server resource consumption (CPU, RAM, storage etc).
- Support for GeoJSON or WKT output formats.
- Support for "Offset" operation which can assist in visualizing data for
  different lanes.
- Bandwidth-efficient `/batch/` mode to integrate with applications<br>
  (Such as custom PowerBI visuals)

## 3. Usage - GET - Text Response (GeoJSON / WKT / JSON / LATLON)

When the web service is running locally (on your own machine) it can be accessed
at the following address by default:

<http://localhost:8080/?>...

The  parameters are summarised in the table below:

| Name       | Description                                                                                                                                                                                           | Example Value               | Lines                              | Points | Default   |
| ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------- | ---------------------------------- | ------ | --------- |
| `road`     | Main Roads Road Number or Local Government Road Number (case sensitive)                                                                                                                               | `road=H001`                 | ✔️                                  | ✔️      | -         |
| `slk_from` | Straight Line Kilometre to start the segment                                                                                                                                                          | `slk_from=1.55`             | ✔️                                  | ❌      | -         |
| `slk_to`   | Straight Line Kilometre to end the segment                                                                                                                                                            | `slk_to=2.3`                | ✔️                                  | ❌      | -         |
| `slk`      | Straight Line Kilometre to a point. (should not be combined with `slk_from` and `slk_to`, see notes below)                                                                                            | `slk=3`                     | ❌                                  | ✔️      | -         |
| `cwy`      | Filter for the carriageway. Must be some combination of the letters `L`, `R` and `S` (not case sensitive).                                                                                            | `cway=LS` or `cway=RS`      | ❔                                  | ❔      | `LRS`     |
| `offset`   | Number of metres to offset the resulting line segments. Large values may not produce any output. Negative values are to the left of the road (in slk direction) and positive values are to the right. | `offset=4` or `offset=-3.5` | ❔                                  | ❔      | `0`       |
| `f`        | Desired response format. Must be `geojson`, `wkt`, `json`, `latlon` or `latlondir`. (see notes below)                                                                                                 | `f=geojson`                 | ❔<br> ~~`latlon`~~ ~~`latlondir`~~ | ❔      | `geojson` |
| `m`        | EXPERIMENTAL<br>Option to include `M` linear slk coordinates in `geojson`, `wkt` or `json` modes.                                                                                                     | `m=true`                    | ❔                                  | ❌      | `false`   |

> **Note:**
>
> 1. Parameters are case insensitive; `CWY=LS` should work the same as `cwy=ls`.
> 1. For `f=wkt` and `f=geojson` the result is always a `MultiPoint` or
>    `MultiLineString` even if the result consists of only one `Point` or
>    `LineString`.
> 1. There is a known issue where empty `MultiPoint` and empty `MultiLineString`
>    objects can sometimes be returned when there is no resulting geometry.
>    In a future version this will be addressed by making the server return an
>    error instead.
> 1. When `f=GeoJSON` responses are always wrapped in a `Feature`.
> 1. The `f=json` format is a nested array like the `"coordinates":...`
>    attribute  in the same format as the geojson `MultiLineString` or
>    `MultiPoint`  array format.
> 1. The `f=latlon` and `f=latlondir` formats are special:
>    - These formats are only valid when using the `slk=` mode.
>    - It will always return a single comma separated latitude longitude pair;
>    - If multiple points would have been returned (left and right carriageway)
>      then the average of these positions is returned
>    - `latlondir` is the same but followed by another comma and then the direction,
>       in degrees. When the direction is averaged (between left and right carriageway)
>       weird things might happen if the carriageways are going in very different directions.
>       Direction is measured anti-clockwise-positive from east.

### 3.1. Example - Get Lines - WKT

The following example fetches the Left and Single carriageway portions of Albany
Highway from slk 1km to slk 2km and offsets them 10 metres to the left of the
road. The result is returned in WKT format as a single `MULTILINESTRING`
feature:

<http://localhost:8080/?road=H001&slk_from=1&slk_to=2&cwy=LS&offset=-10&f=wkt>

```wkt
MULTILINESTRING ((115.88771097361135 -31.967604589743765,115.88776331305647 -31.96753166223028,115.88782456479156 -31.967494045166685,115.88808285746482 -31.967581573012584,115.88842643824691 -31.967706811122067,115.88865106830647 -31.967794863020398,115.88878639134748 -31.967856418305686,115.88961385220324 -31.968270404421514),(115.88961222617614 -31.96826961114868,115.89009682355594 -31.968500014510138),(115.89009709103813 -31.968500142226866,115.8908060320806 -31.96884008302064,115.89130780129135 -31.96906658240955),(115.89129847166095 -31.9690630113479,115.8924861744535 -31.96944832848648),(115.89248599022535 -31.969448268938134,115.89367451654047 -31.969831024006037),(115.89367516412221 -31.96983123526756,115.89489443528633 -31.970234120891217),(115.89489484503049 -31.970234348879462,115.8952199535067 -31.97034351139344,115.89552559070945 -31.970457312501807,115.89572276324779 -31.97054445312055,115.89588899502093 -31.97062796284781,115.89603052161054 -31.9707213605839),(115.89603611706656 -31.97072540301373,115.8961699852627 -31.970830173406412,115.89636973106218 -31.970999046961516,115.89654509709025 -31.971164236270756,115.89708949236724 -31.971705035229636),(115.88735210575929 -31.967327078117492,115.88761740846113 -31.967472091243042),(115.88761495220085 -31.96747075121283,115.88782449298621 -31.967576711138406))
```

### 3.2. Example - Get Points - GeoJSON

The following example fetches the a point for both the Left and right
carriageway portions of Albany Highway at `slk=2`. The format returned in the
default GeoJSON as a `MultiPoint` feature, because there is both a left and
right carriageway at this location:

<http://localhost:8080/?road=H001&slk=2>

```json
{"type":"Feature", "geometry":{"type":"MultiPoint", "coordinates":[[115.89702617983814,-31.97176876372234],[115.89692159880637,-31.97178473847775]]}}
```

### 3.3. Example - Get Point - Simple Latitude Longitude

The following example includes the format specifier `f=latlon`. It returns a
single point on Albany Highway at slk 2 km. If there were multiple locations
with `slk=2` (due to multiple carriageways, or due to slk gaps and overlaps)
then the average position of all such points is returned:

<http://localhost:8080/?road=H001&slk=2&f=latlon>

```text
-31.971776751100045,115.89697388932225
```

## 4. Usage - Browser - `/show/` Page

Show mode works the same as described above, except that instead of returning
raw data, it displays an interactive map when viewed in a web browser. This is
useful to confirm that queries are working as intended. Simply add `/show/` to
the url before the query parameters:

[http://localhost:8080<b>/show/</b>?road=H001&slk_from=1&slk_to=2&cwy=LS&offset=-10&f=wkt](http://localhost:8080/show/?road=H001&slk_from=1&slk_to=2&cwy=LS&offset=-10&f=wkt)
![show_demo.jpg](./readme_extras/show_demo.jpg)

Query mode can easily be used from Excel with the `=WEBSERVICE()` formula, or
from Power BI using the `=Web.Contents()` function.

## 5. Usage - POST - `/batch/` Route for app Integration

`/batch/` mode is a bandwidth efficient alternative query method meant for
integration with apps and PowerBI custom visuals.

This mode expects a `POST` request to <http://localhost:8080/batch/> and does
not use url query parameters. See details below.

<details>

<summary style="color:red; font-weight:bold; font-size:1.1em;margin:2em 0">
Click to expand details of `/batch/` Mode
</summary>

The body of the request must be binary data consisting of a series of frames
with the format shown below. Any number of frames can be packed into a single
request.

Frame format:

| Byte Length | Type                  | Value                                  |
| ----------- | --------------------- | -------------------------------------- |
| 1           | Uint8                 | Number of bytes in `road` string `x`   |
| `x`         | Utf8 String           | `road` number                          |
| 4           | Float32 Little Endian | `slk_from` in kilometres               |
| 4           | Float32 Little Endian | `slk_to` in kilometres                 |
| 4           | Float32 Little Endian | `offset` in metres                     |
| 1           | Uint8                 | `cwy` (carriageways) (see table below) |

Value of `cwy`:

| `cwy` | Carriageway    | Binary          | Decimal |
| ----- | -------------- | --------------- | ------- |
| `R`   | Right only     | `0b0000_0001`   | 1       |
| `S`   | Single only    | `0b0000_0010`   | 2       |
| `RS`  | Right & Single | `0b0000_0011`   | 3       |
| `L`   | Left only      | `0b0000_0100`   | 4       |
| `LR`  | Left & Right   | `0b0000_0101`   | 5       |
| `LS`  | Left & Single  | `0b0000_0110`   | 6       |
| `LRS` | All            | any other value |         |

There is an example batch query implementation in `__static_http/main.js`
however a simplified version is shown below:

```javascript
// =========== Helper functions: ===========
let CWY_LOOKUP = {
    L: 0b0000_0100,
    R: 0b0000_0001,
    S: 0b0000_0010,
    LR: 0b0000_0101,
    LS: 0b0000_0110,
    RS: 0b0000_0011,
    LRS: 0b0000_0111
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
fetch("http://localhost:8080/batch/", {
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

</details>

## 6. Notes

### 6.1. SLK, True Distance and Chainage

SLK stands for "Straight Line Kilometre" and is sometimes called 'chainage' or
'kilometrage' in other contexts.

At Main Roads Western Australia SLK refers to an "adjusted" linear measure which
has discontinuities called 'Points of Equation' (there are between 100 and 200
points of equation throughout the state road network) where there is an abrupt
increase or decrease in SLK. This is done so that when asset locations are
recorded by SLK, these records are not invalidated when a road realignment
project modifies the length of a road.

This software has no special compensation to handle POE discontinuities. Please
expect results at POEs to have gaps or overlaps.

The non-adjusted linear measure is called "True Distance".

This software is only capable of looking up Lat/Lon from SLK. True distance is
not yet supported.

### 6.2. Supported Network Types

This tool is capable of querying all road network types included in this dataset
<https://portal-mainroads.opendata.arcgis.com/datasets/mainroads::road-network/about>

| Network Type               | Support |
| -------------------------- | ------- |
| State Roads                | ✔️       |
| Proposed State Roads       | ✔️       |
| Local Government Roads     | ✔️       |
| Main Roads Controlled Path | ✔️       |
| Miscellaneous Road         | ✔️       |
| Crossover                  | ✔️       |

### 6.3. Coordinate Reference System (CRS)

The coordinate system of the returned geometry depends on the coordinate system
downloaded from `NLR_DATA_SOURCE_URL`.

However, `offset=` feature will only work correctly with EPSG:4326 (which is
also called WGS84. See <https://spatialreference.org/ref/epsg/wgs-84/>) This is
because the `&offset=...` uses an approximation to convert from meters to
degrees assuming that there are about `111320` metres per degree.

## 7. Running the Server Yourself

### 7.1. Installation

Windows and most Linux systems are supported, however only windows binaries are
distributed from the [releases](/releases) page.

To use a pre-compiled version of this application please visit the
[releases](/releases) page and download and extract one of the zip files.

Run `nicklinref.exe` then visit
<http://localhost:8080/?road=H001&slk_from=1.5&slk_to=3> to test if it is
working.

If possible, the best way to use this software is to clone this repository and
build it yourself. The main branch is kept at the latest release version.

### 7.2. Compilation

#### 7.2.1. Compiling for Ubuntu / Debian

Install required packages:

```bash
sudo apt update
sudo apt-get install build-essential pkg-config git
```

> Note: On some linux distros, the packages `libssl-dev` and `build-essential`
> may have different names. `build-essential` contains the `libc` package which
> is needed by some rust packages to interact with other platform code.

> Note: In previous version `libssl-dev` and `pkg-config` may be needed. Newer
> version use rust-tls and do not depend on openssl.

Install Rust:

Follow the this guide <https://www.rust-lang.org/tools/install>
Probably something like this:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Clone this repository

```bash
git clone https://github.com/thehappycheese/nicklinref_rust
cd nicklinref_rust
```

Build and run:

```bash
cargo run --release
```

#### 7.2.2. Compiling for Windows

Install rust: <https://www.rust-lang.org/tools/install>. You may be prompted to
install some microsoft visual C++ thing which is used for linking native
executables.

Clone this repository

```powershell
git clone https://github.com/thehappycheese/nicklinref_rust
cd nicklinref_rust
```

Build and run:

```powershell
cargo run --release
```

### 7.3. Configuration

LinRef can be configured using either environment variable or command line
arguments. (Previous versions supported a `config.json` option, but support for
this is dropped because it was never used)

#### 7.3.1. Configuration - Command Line Interface (CLI)

nicklinref supports a help command flag which will print out the most current
command line documentation.

```powershell
nicklinref.exe --help
```

```text
Usage: nicklinref [OPTIONS]

Options:
      --addr <NLR_ADDR>
          The IP address to listen on [env: NLR_ADDR=] [default: 127.0.0.1]
      --port <NLR_PORT>
          The port to listen on [env: NLR_PORT=] [default: 8080]
      --data-file <NLR_DATA_FILE>
          File path to where the cache data file is/will be stored, including file name [env: NLR_DATA_FILE=] [default: ./data/data.json.lz4]
      --static-http <NLR_STATIC_HTTP>
          Folder path containing static http files for the /show/ route [env: NLR_STATIC_HTTP=] [default: ./__static_http]
      --force-update-data
          Cause the old data cache file to be deleted and re-downloaded [env: NLR_FORCE_UPDATE_DATA=]
      --data-source-url <NLR_DATA_SOURCE_URL>
          Url of the esri rest service hosting the road network data [env: NLR_DATA_SOURCE_URL=] [default: https://mrgis.ma...]
  -h, --help
          Print help
```

#### 7.3.2. Configuration - Environment Variables

As an alternative to command line options, environment variables can be used  instead.

Configuration options set using environment variables are overridden by any
command line options.

1. Check to see if a config file has been specified on the command line using
   the `--config` option:

```shell
nicklinref.exe
```

1. Finally, environment variables with matching names will be used to overwrite
   any options loaded so far.
   - If there is an error while processing an environment variable, the
     previously loaded option will be used instead. (Note: This may be changed
     to a fatal error in the future.)

The following table describes the configuration options in more detail:

| Property              | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `NLR_ADDR`            | A string containing an IPV4 or IPV6 address. Using 127.0.0.1 will limit traffic to your own machine for testing purposes. 0.0.0.0 will allow requests from anywhere on the local network.                                                                                                                                                                                                                                                                                                                                                                                          |
| `NLR_PORT`            | A port number.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `NLR_DATA_FILE`       | The filename of the data cached from `NLR_DATA_SOURCE_URL`. **The directory must already exist**. If the file does not already exist then it will be created and fresh data will be downloaded.                                                                                                                                                                                                                                                                                                                                                                                    |
| `NLR_DATA_SOURCE_URL` | This is the ArcGIS REST service where the road network is downloaded from. It is assumed that multiple requests are needed and the `&resultOffset=...` parameter is used to repeatedly fetch more data. Only certain fields are fetched `outFields=ROAD,START_SLK,END_SLK,CWY` and the output spatial reference is specified `&outSR=4326`. ESRI's own json format (`&f=json`) is expected because `&f=geojson` does not seem to work properly. Also note that currently the field names `ROAD`, `START_SLK`, `END_SLK`, `CWY` are hard-coded and must exist on the incoming data. |
| `NLR_STATIC_HTTP`     | Used by the `/show/` feature to display an interactive map. The directory specified by this config option should exist or I think the application may crash on startup. The directory can probably be empty though if it is not required. The `__static_http` folder in this repo contains the files required.                                                                                                                                                                                                                                                                     |

### 7.4. Data Download and Refresh

To refresh your data, simply manually delete the file specified by the
`NLR_DATA_FILE` option and restart the application. Alternatively add the
`--force-update-data` flag to the command line when launching the server. Fresh
data will be downloaded.


## 8. Related Projects

### 8.1. Megalinref

[Megalinref](https://github.com/thehappycheese/megalinref) is an attempt to
bring the functionality of this server directly to python. It is a rust-powered
python library that will do all the same things as this server, but without the
overhead of running a rest service on localhost.

### 8.2. NickMapBI

[NickMapBI](https://github.com/thehappycheese/nickmap-bi/) is a custom PowerBI
visual which calls into a running instance of NickLinRef.

### 8.3. Python version (Predecessor to this Rust version)

This repo is a rust implementation of my previous project written in python:
<https://github.com/thehappycheese/linear_referencing_geocoding_server>
