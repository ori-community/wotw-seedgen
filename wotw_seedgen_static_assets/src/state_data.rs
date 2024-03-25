use lazy_static::lazy_static;
use wotw_seedgen_assets::StateData;

lazy_static! {
    pub static ref STATE_DATA: StateData =
        ciborium::from_reader(include_bytes!(concat!(env!("OUT_DIR"), "/state_data")).as_slice())
            .unwrap();
}
