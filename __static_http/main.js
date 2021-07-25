

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

let featureProjection = view.getProjection();

let dataProjection = new ol.format.GeoJSON().readProjection({ "crs": { "type": "EPSG", "properties": { "code": 4326 } } });

let layer_osm = new ol.layer.Tile({
	source: new ol.source.OSM()
})

let layer_geojson = new ol.layer.Vector({
	source: new ol.source.Vector({}),
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

let map = new ol.Map({
	layers: [layer_osm, layer_geojson],
	target: 'map',
	view
});

map.on("moveend", (e) => {
	let view = map.getView();
	localStorage.setItem("view", JSON.stringify({
		center: view.getCenter(),
		zoom: view.getZoom()
	}))
})

fetch("secrets.json")
	.then(resp => {
		if (resp.ok) return resp
		throw new Error(`Request for secrets failed: ${resp.statusText}`)
	})
	.then(resp => resp.json())
	.catch(err => {
		throw new Error(`Decoding secrets failed: ${err}`)
	})
	.then(secrets => {

		window.layer_metro_map = new ol.layer.Tile({
			source: new ol.source.XYZ({
				url: secrets.metromap,
			})
		});

		window.layer_skyview_tiles = new ol.layer.Tile({
			source: new ol.source.TileArcGISRest({
				params: {
					FORMAT: "jpgpng",
					compressionQuality: 75,
					TRANSPARENT: false
				},
				// crossOrigin:"Anonymous", // Required if we need to retrieve canvas pixle data later.
				url: secrets.skyview
			})
		});

	});



add_features(new URLSearchParams(window.location.search)).then(success => success && zoom_to_loaded_features());

async function add_features(url_params, fetch_pool = undefined) {
	f = url_params.get("f")
	url_params.delete("f");// geojson is default

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


			let json_features;
			try {
				json_features = JSON.parse(response_text);
			} catch (e) {
				throw new Error(`Unable parse response: ${e.message}\n${response_text}`);
			}

			if (json_features?.geometry?.type == "MultiPoint" && f=="latlon"){
				// handle the wierd case where the latlon option averages the resulting point position into a single point.
				let average_x = 0;
				let average_y = 0;
				for(item of json_features.geometry.coordinates){
					average_x+=item[0]
					average_y+=item[1]
				}
				average_x /= json_features.geometry.coordinates.length
				average_y /= json_features.geometry.coordinates.length
				json_features = {type:"Feature",geometry:{type:"Point",coordinates:[average_x, average_y]}}
			}
			let read_features = new ol.format.GeoJSON({ featureProjection, dataProjection }).readFeatures(json_features);
			
			layer_geojson.getSource().addFeatures(read_features);


			return true;
		});
}

function zoom_to_loaded_features() {
	let target_extent = layer_geojson.getSource().getExtent();
	let resolution = view.getResolutionForExtent(target_extent);
	let target_zoom = view.getZoomForResolution(resolution) / 1.01;
	let target_center = ol.extent.getCenter(target_extent);

	view.animate({
		zoom: target_zoom,
		center: target_center,
		duration: 1500,
		easing: ol.easing.easeOut
	});
}

function radio_update(e) {
	set_background(e.target.value);
}

function set_background(val) {
	try {
		map.removeLayer(layer_osm)
		map.removeLayer(layer_skyview_tiles)
		map.removeLayer(layer_metro_map)

		switch (val) {
			case "openstreetmap":
				map.getLayers().insertAt(0, layer_osm)
				break
			case "metromap":
				map.getLayers().insertAt(0, layer_metro_map)
				break
			case "skyview":
				map.getLayers().insertAt(0, layer_skyview_tiles)
				break
		}
	} catch (e) {
		console.log(e)
		alert("Error loading layer. Reverting to Open Street Maps.");
		map.removeLayer(layer_osm);
		map.getLayers().insertAt(0, layer_osm);
	}
}


let demo_tour = [
	{ road: "H001", slk_from: 0, slk_to: 50, step: 0.1 }, // 500 features
	{ road: "H005", slk_from: 0, slk_to: 500, step: 1 }, // 500 features
	{ road: "H016", slk_from: 0, slk_to: 20, step: 0.01 }, // 2000 features
	{ road: "H015", slk_from: 0, slk_to: 20, step: 0.01 }, // 2000 features
	{ road: "H023", slk_from: 0, slk_to: 15, step: 0.01 }, // 1500 features
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



let CWY = {
	L: 0b0000_0100,
	R: 0b0000_0001,
	S: 0b0000_0010,
	LR: 0b0000_0101,
	LS: 0b0000_0110,
	RS: 0b0000_0011,
	LRS: 0b0000_0111
}
/** Encodes a request into an ArrayBuffer as follows:
 * 	
 * [road_name_length:u8,  road_name:UTF8...,  slk_from:f32, slk_to:f32, offset:f32, cwy:u8]
 * 
 * where floats are encoded in Little Endian
 */
function binary_encode_request(road, slk_from, slk_to, offset, cwy) {

	let text_encoder = new TextEncoder();
	let road_bytes = text_encoder.encode(road);

	let buffer = new ArrayBuffer(1 + road_bytes.length + 4 + 4 + 4 + 1);


	let road_name_chunk = new Uint8Array(buffer, 0, 1 + road_bytes.length);
	road_name_chunk[0] = road_bytes.length;
	road_name_chunk.set(road_bytes, 1);

	let dataview = new DataView(buffer, 1 + road_bytes.length);
	dataview.setFloat32(0, slk_from, true) // LITTLE ENDIAN
	dataview.setFloat32(4, slk_to, true) // LITTLE ENDIAN
	dataview.setFloat32(8, offset, true) // LITTLE ENDIAN
	dataview.setUint8(12, cwy);

	return new Uint8Array(buffer);
}