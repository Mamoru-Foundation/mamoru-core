mod validation_chain;

use std::sync::Once;

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(env_logger::init);
}
