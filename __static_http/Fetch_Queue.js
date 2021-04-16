

class Fetch_Queue {

	queue = [];

	constructor(size) {
		if (size <= 0) {
			throw new Error("Fetch_Pool(size) size must be positive number");
		}
		this.size = size;
	}

	async resolve_handler(resolved_promise, fetch_result) {
		this.unresolved.delete(resolved_promise);
		return fetch_result;
	}

	async fetch(url) {
		let resolver;
		let np = new Promise((resolve, reject) => {
			resolver = resolve;
		});
		this.queue.push([url, resolver]);
		return np;
	}

	*chunk(iter, n) {
		let batch = [];
		for (let item of iter) {
			batch.push(item);
			if (batch.length > n) {
				yield batch
				batch = [];
			}
		}
		if (batch) yield batch;
	}

	async all() {
		for (let batch of this.chunk(this.queue, 30)) {
			let promises = [];
			for (let [url, resolver] of batch) {
				promises.push(
					fetch(url).then(result => resolver(result))
				);
			}
			await Promise.all(promises);
		}
		this.queue = [];
	}

}