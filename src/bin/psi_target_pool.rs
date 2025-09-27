use log::info;

use psitool::logger;
use psitool::rvuid::Rvuid;

fn main() {
    logger::init(true);
    let uuid = Rvuid::from_bytes(b"foobar");
    info!("uuid is: {}", uuid);
}
