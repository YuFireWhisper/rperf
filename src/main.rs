use rperf::core::virtual_user::VirtualUser;

#[tokio::main]
async fn main() {
    let url = "http://35.194.179.59";

    let mut virtual_user = VirtualUser::new(url, std::time::Duration::from_secs(1));

    virtual_user.start();
    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    virtual_user.stop().await;

    let metrics = virtual_user.metrics();
    let m = metrics.lock().await;
    println!("{:#?}", m);
}
