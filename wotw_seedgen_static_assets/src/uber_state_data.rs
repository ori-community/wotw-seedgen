use lazy_static::lazy_static;
use wotw_seedgen_assets::UberStateData;

lazy_static! {
    pub static ref UBER_STATE_DATA: UberStateData = ciborium::from_reader(
        include_bytes!(concat!(env!("OUT_DIR"), "/uber_state_data")).as_slice()
    )
    .unwrap();
}
