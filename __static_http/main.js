///////////////////////
// View / Zoom
///////////////////////
let view = new ol.View({
	center: [12898411.077810172, -3757643.0263860035],
	zoom: 5.5,
});

if (localStorage.view) {
	try {
		let ls_view = JSON.parse(localStorage.view)
		view.setCenter(ls_view.center)
		view.setZoom(ls_view.zoom)
	} catch (e) { }
}

function reset_view() {
	view.setCenter([12898411.077810172, -3757643.0263860035]);
	view.setZoom(5.5);
	delete localStorage.view;
}


function zoom_to_loaded_features() {
	let target_extent = layer_geojson.getSource().getExtent();
	let resolution    = view.getResolutionForExtent(target_extent);
	let target_zoom   = view.getZoomForResolution(resolution) / 1.01;
	let target_center = ol.extent.getCenter(target_extent);

	view.animate({
		zoom     : target_zoom,
		center   : target_center,
		duration : 1500,
		easing   : ol.easing.easeOut
	});
}

///////////////////////
// Projection
///////////////////////

let featureProjection = view.getProjection();

let dataProjection = new ol.format.GeoJSON().readProjection({ "crs": { "type": "EPSG", "properties": { "code": 4326 } } });


///////////////////////
// Map Layers
///////////////////////

let layer_osm = new ol.layer.Tile({
	source: new ol.source.OSM()
})

let geojson_source = new ol.source.Vector({})
let layer_geojson = new ol.layer.Vector({
	source: geojson_source,
	style: [
		new ol.style.Style({
			stroke: new ol.style.Stroke({
				//color: '#319FD3',
				color: 'white',
				width: 7,
			}),
			image: new ol.style.Circle({
				radius: 8,
				stroke: new ol.style.Stroke({ color: 'white', width: 7 }),
			}),
		}),
		new ol.style.Style({
			stroke: new ol.style.Stroke({
				//color: '#319FD3',
				color: 'red',
				width: 3,
			}),
			image: new ol.style.Circle({
				radius: 8,
				stroke: new ol.style.Stroke({ color: 'red', width: 3 }),
			}),
		})
	]
});

///////////////////////
// Map Object
///////////////////////

let map = new ol.Map({
	layers: [layer_osm, layer_geojson],
	target: 'map',
	view
});

///////////////////////
// Map Events
///////////////////////

map.on("moveend", (e) => {
	let view = map.getView();
	localStorage.setItem("view", JSON.stringify({
		center: view.getCenter(),
		zoom: view.getZoom()
	}))
})


///////////////////////
// On Load, Fetch Features based on URL, and add to map
///////////////////////

add_features(new URLSearchParams(window.location.search)).then(success => success && zoom_to_loaded_features());

//////////////////////////////////////////////////////////////////////
// Get Geometry
// Optionally, use a Fetch_Queue to avoid browser error from too many parallel requests
// For which the fetch API itself provides no convienient work-around
///////////////////////////////////////////////////////////////////////

async function add_features(url_params, fetch_pool = undefined) {
	f = url_params.get("f") ?? "geojson";
	if(!(f.toLowerCase()==="latlon" || f.toLowerCase()==="latlondir")){
		url_params.set("f","wkt");// geojson is default
	}

	let url_to_fetch = "/?" + url_params.toString();

	let fetcher;

	if (fetch_pool) {
		fetcher = fetch_pool.fetch(url_to_fetch)
	} else {
		fetcher = fetch(url_to_fetch)
	}

	return fetcher
		.then(resp => {
			if (resp.ok) return resp;
			throw new Error(`Response not ok: ${resp.statusText}`)
		})
		.then(response => response.text())
		.then(response_text => {

			if(f.toLowerCase()==="latlon"){
				layer_geojson.getSource().addFeature(new ol.Feature(new ol.geom.Point(ol.proj.fromLonLat(
					response_text.split(",").map(parseFloat).reverse()
				))));
			}else if(f.toLowerCase()==="latlondir"){
				const pointer_len = 0.0003;
				let [dir, ...lonlat] = response_text.split(",").map(parseFloat).reverse()
				let lonlat2 = [lonlat[0]+Math.cos(dir/180*Math.PI)*pointer_len,lonlat[1]+Math.sin(dir/180*Math.PI)*pointer_len]
				lonlat = ol.proj.fromLonLat(lonlat)
				lonlat2 = ol.proj.fromLonLat(lonlat2)
				layer_geojson.getSource().addFeature(new ol.Feature(new ol.geom.LineString([lonlat,lonlat2])));
				layer_geojson.getSource().addFeature(new ol.Feature(new ol.geom.Point(lonlat)));
			}else{
				let read_features = new ol.format.WKT().readFeatures(response_text,{ featureProjection, dataProjection });
				layer_geojson.getSource().addFeatures(read_features);
			}

			return true;
		});
}




////////////////////////////////////////
// Demo that sends heaps of requests
////////////////////////////////////////


let demo_tour = [
	{ road: "H001", slk_from: 0, slk_to:  50, step: 0.1  }, // 500 features
	{ road: "H005", slk_from: 0, slk_to: 500, step: 1    }, // 500 features
	{ road: "H016", slk_from: 0, slk_to:  20, step: 0.01 }, // 2000 features
	{ road: "H015", slk_from: 0, slk_to:  20, step: 0.01 }, // 2000 features
	{ road: "H023", slk_from: 0, slk_to:  15, step: 0.01 }, // 1500 features
]

function run_demo() {
	let fetch_pool = new Fetch_Queue(200);

	layer_geojson.getSource().clear();

	for (item of demo_tour) {
		for (let i = item.slk_from; i < item.slk_to; i += item.step) {
			add_features(
				new URLSearchParams({
					road: item.road,
					slk_from: i,
					slk_to: i + item.step
				}),
				fetch_pool
			);
		}
	}

	fetch_pool.then(arr => zoom_to_loaded_features());
}

////////////////////////////////////////
// Demo that sends batch requests
////////////////////////////////////////

let demo_tour_batch = [
	{ road: "H002", slk_from: 0, slk_to: 50, step: 0.1 }, // 500 features
	{ road: "H052", slk_from: 0, slk_to: 500, step: 1 }, // 500 features
	{ road: "H018", slk_from: 0, slk_to: 30, step: 0.01 }, // 2000 features
	{ road: "H015", slk_from: 20, slk_to: 40, step: 0.01 }, // 2000 features
	{ road: "H038", slk_from: 0, slk_to: 15, step: 0.01 }, // 1500 features
]


function fetch_batch() {

	layer_geojson.getSource().clear();
	let request_batchs = [];

	// one request per road
	for (item of demo_tour_batch) {

		let total_byte_length = 0;
		let request_batch_array = [];

		for (let i = item.slk_from; i < item.slk_to; i += item.step) {
			let bytes = binary_encode_request(item.road, i, i + item.step, 0, CWY.LRS)
			request_batch_array.push(bytes);
			total_byte_length += bytes.byteLength;
		}

		let request_batch_binary = new Uint8Array(total_byte_length);
		let offset = 0;
		request_batch_array.forEach(byte_array => {
			request_batch_binary.set(byte_array, offset);
			offset += byte_array.byteLength;
		})
		request_batchs.push(request_batch_binary)

	}

	let fetches = [];
	request_batchs.forEach(request_batch_binary => {
		fetches.push(
			fetch("/batch/", {
					method: "POST",
					body: request_batch_binary
				}
			)
			.then(response => {
				return response.text()
			})
			.then(text => {
				
				let json = JSON.parse(text);
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
				let read_features = new ol.format.GeoJSON({ featureProjection, dataProjection }).readFeatures(
					{
						type:"FeatureCollection",
						features
					}
				);
				layer_geojson.getSource().addFeatures(read_features);
			})
		)
	});
	Promise.all(fetches).then(()=>{
		zoom_to_loaded_features()
	});
}

