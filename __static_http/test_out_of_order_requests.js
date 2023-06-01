




let test_out_of_order_requests_requests = [
	{ id:2, road: "H018", slk_from: 0, slk_to: 30, step: 0.01 }, // 2000 features
	{ id:4, road: "H038", slk_from: 0, slk_to: 0.5, step: 0.1 }, // 50 features
]


function test_out_of_order_requests() {

	layer_geojson.getSource().clear();
	let request_batches = [];

	// one request per road
	for (item of test_out_of_order_requests_requests) {

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
		request_batches.push({id:item.id, request_batch_binary})
	}

	let fetches = [];
	request_batches.forEach(request_batch_binary => {
        console.log(`Requesting id:${request_batch_binary.id} len:${request_batch_binary.request_batch_binary.byteLength}`)
		fetches.push(
			fetch("/batch/", {
					method: "POST",
					body: request_batch_binary.request_batch_binary,
                    headers: {
                        "x-request-id": request_batch_binary.id
                    }
				}
			)
			.then(response => {
                if (!response.ok) {
                    console.log("error testing")
                    return;
                }
                console.log("response ok")
                console.log(response.headers)
                for (header of ["x-request-id"]) {
                    console.log(`${header}: ${response.headers.get(header)}`)
                }
			})
		)
    });
	Promise.all(fetches).then(()=>{
		console.log("done")
	});
}
