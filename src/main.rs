use rperf::core::virtual_user_manager::{VirtualUserConfig, VirtualUserManager};

#[tokio::main]
async fn main() {
    const URL: &str = "http://35.194.179.59";
    let config = VirtualUserConfig::new(URL);

    let mut virtual_user_manager = VirtualUserManager::new(config);
    virtual_user_manager.add_plan(std::time::Duration::from_secs(10), 120);
    
    virtual_user_manager.run().await;

    let metrics = virtual_user_manager.get_overall_metrics();
    println!("{:#?}", metrics);
    println!("ave total_latency: {}", metrics.total_latency.average().unwrap());
}
