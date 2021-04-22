

class Fetch_Queue {

	concurrency_limit = 30;
	number_of_running_tasks = 0;
	queue = [];
	completion_awaiters = [];

	constructor(concurrency_limit = 30) {
		if (concurrency_limit <= 0) {
			throw new Error(`Fetch_Pool(concurrency_limit=${concurrency_limit}) concurrency_limit must be positive number`);
		}
		this.concurrency_limit = concurrency_limit;
	}

	fetch(url, options={}) {
		let record;
		let np = new Promise((resolve, reject) => {
			record = {url, options, resolve, reject}
			this.queue.push(record);
		});
		this.fill_pool();
		return np;
	}

	fill_pool(){
		// try to empty queue until
		if(this.queue.length<=0){
			if(this.number_of_running_tasks==0){
				this.finish();
			}
		}else{
			while(this.number_of_running_tasks<this.concurrency_limit && this.queue.length>0){
				this.number_of_running_tasks++;
				let new_task = this.queue.pop();

				fetch(new_task.url, new_task.options)
					.then(
						(response)=>{
							this.number_of_running_tasks--;
							this.fill_pool();
							new_task.resolve(response);
						},
						(rejection)=>{
							this.number_of_running_tasks--;
							this.fill_pool();
							new_task.reject(rejection);
						}
					);
			}
		}
	}

	finish(){
		for(let awaiter of this.completion_awaiters){
			awaiter();
		}
		this.completion_awaiters = [];
	}

	then(func) {
		return new Promise((resolve, reject)=>{
			this.completion_awaiters.push(resolve)
		}).then(()=>func())
	}

}