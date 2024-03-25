use lazy_static::lazy_static;
use wotw_seedgen_assets::LocData;

lazy_static! {
    pub static ref LOC_DATA: LocData =
        ciborium::from_reader(include_bytes!(concat!(env!("OUT_DIR"), "/loc_data")).as_slice())
            .unwrap();
}
