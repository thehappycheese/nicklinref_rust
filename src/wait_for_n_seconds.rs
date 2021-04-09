
use tokio::time::{sleep, Duration};
pub async fn wait_for_n_seconds(n:u64){
	print!(
		">> Begin waiting {} seconds for RAM check....",
		n	
	);
	sleep(Duration::new(n, 0)).await;
	println!(" DONE");
}