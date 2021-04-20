

let view = new ol.View({
	center: [12898411.077810172, -3757643.0263860035],
	zoom: 5.5,
});

if(localStorage.view){
	try{
		let ls_view = JSON.parse(localStorage.view)
		view.setCenter(ls_view.center)
		view.setZoom(ls_view.zoom)
	}catch(e){}
}

let featureProjection = view.getProjection();

let dataProjection = new ol.format.GeoJSON().readProjection({"crs":{"type":"EPSG","properties":{"code":4326}}});

let layer_osm = new ol.layer.Tile({
	source: new ol.source.OSM()
})

let layer_geojson = new ol.layer.Vector({
	source: new ol.source.Vector({}),
	style:[
		new ol.style.Style({
			stroke: new ol.style.Stroke({
				//color: '#319FD3',
				color: 'white',
				width: 7,
			}),
			image: new ol.style.Circle({
				radius: 8,
				stroke: new ol.style.Stroke({color: 'white', width:7}),
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
				stroke: new ol.style.Stroke({color: 'red', width:3}),
			}),
		})
	]
});

let map = new ol.Map({
	layers: [layer_osm, layer_geojson],
	target: 'map',
	view
});

map.on("moveend",(e)=>{
	let view = map.getView();
	localStorage.setItem("view",JSON.stringify({
		center:view.getCenter(),
		zoom:view.getZoom()
	}))
})

fetch("secrets.json")
	.then(resp=>{
		if(resp.ok) return resp
		throw new Error(`Cannot continue, initial response to request for secrets not ok. ${resp.statusText}`)
	})
	.then(resp=>resp.json())
	.catch(err=>{
		throw new Error(`Cannot continue, secrets cannot be decoded:  ${err}`)
	})
	.then(secrets=>{

		window.layer_metro_map = new ol.layer.Tile({
			source: new ol.source.XYZ({
				url:secrets.metromap,
			})
		});
		
		window.layer_skyview_tiles = new ol.layer.Tile({
			source: new ol.source.TileArcGISRest({
				params:{
					FORMAT:"jpgpng",
					compressionQuality:75,
					TRANSPARENT:false
				},
				// crossOrigin:"Anonymous", // Required if we need to retrieve canvas pixle data later.
				url:secrets.skyview
			})
		});

	});



add_features(new URLSearchParams(window.location.search)).then(success => success && zoom_to_loaded_features());

async function add_features(url_params, fetch_pool=undefined) {

	url_params.delete("format");
	let url_to_fetch = "/query/?" + url_params.toString();
	
	let fetching;

	if (fetch_pool){
		fetching = fetch_pool.fetch(url_to_fetch)
	}else{
		fetching = fetch(url_to_fetch)
	}

	return fetching
		.then(resp=>{
			if(resp.ok) return resp
			throw new Error(`Cannot continue, initial response not ok. ${resp.statusText}`)
		})
		// .then(r => r.json())
		// .catch(err=>{
		// 	console.log(err)
		// 	alert("Error in response. GeoJSON could not be loaded or parsed");
		// 	throw new Error("Error in response. GeoJSON could not be loaded or parsed")
		// })
		.then(r => r.text())
		.then(text_geojson => {

			// temporary code to parse Rust debug output.
			// let pat = /\[(Vector2 { x:.*?, y:.*?},? ?)+\]+/g;
			// let mls = [];
			// for (match of text_geojson.match(pat)){
			// 	console.log("outer match "+ match);
			// 	let coords = [...match.matchAll(/{ x:(.*?), y:(.*?)}/g)].map(item=>[parseFloat(item[1]),parseFloat(item[2])])
			// 	mls.push(coords)
			// 	//console.log(coords);
			// }
			let mls;
			try{
				mls = JSON.parse(text_geojson);
			}catch(e){
				return false;
			}
			
			//console.log(mls)
			let GEOJSON = {
				type:"Feature",
				geometry:{
					type:"MultiLineString",
					coordinates:mls
				}		
			};
			//let GEOJSON = JSON.parse(text_geojson);
			
			let read_features = new ol.format.GeoJSON({featureProjection, dataProjection}).readFeatures(GEOJSON);
			layer_geojson.getSource().addFeatures(read_features);

			
			return true;
		});
}

function zoom_to_loaded_features(){
	let target_extent = layer_geojson.getSource().getExtent();
	let resolution = view.getResolutionForExtent(target_extent);
	let target_zoom = view.getZoomForResolution(resolution)/1.01;
	let target_center = ol.extent.getCenter(target_extent);

	view.animate({
		zoom:target_zoom,
		center:target_center,
		duration: 1500,
		easing: ol.easing.easeOut
	});
}

function radio_update(e){
	set_background(e.target.value);
}

function set_background(val){
	try{
		map.removeLayer(layer_osm)
		map.removeLayer(layer_skyview_tiles)
		map.removeLayer(layer_metro_map)

		switch(val){
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
	}catch(e){
		console.log(e)
		alert("Error loading layer. Reverting to Open Street Maps.");
		map.removeLayer(layer_osm);
		map.getLayers().insertAt(0, layer_osm);
	}
}


function run_demo(){
	let fetch_pool = new Fetch_Queue();
	
	layer_geojson.getSource().clear();

	for(let i = 0; i < 50; i += 0.01){
		add_features(
			new URLSearchParams({
				road:		"H001",
				slk_from:	i,
				slk_to:		i+0.01
			}),
			fetch_pool
		);
	}
	fetch_pool.all().then(arr=>zoom_to_loaded_features())
}