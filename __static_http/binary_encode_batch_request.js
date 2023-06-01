////////////////////////////////////////
// Binary encoder for batch requests
////////////////////////////////////////

let CWY = {
	L:   0b0000_0100,
	R:   0b0000_0001,
	S:   0b0000_0010,
	LR:  0b0000_0101,
	LS:  0b0000_0110,
	RS:  0b0000_0011,
	LRS: 0b0000_0111,
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