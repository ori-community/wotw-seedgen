use ordered_float::OrderedFloat;
use rustc_hash::FxHashMap;

// TODO use Vecs?
#[derive(Debug, Default, Clone)]
pub struct Variables {
    booleans: FxHashMap<usize, bool>,
    integers: FxHashMap<usize, i32>,
    floats: FxHashMap<usize, OrderedFloat<f32>>,
    strings: FxHashMap<usize, String>,
}

impl Variables {
    #[inline]
    pub fn set_boolean(&mut self, id: usize, value: bool) {
        self.booleans.insert(id, value);
    }

    #[inline]
    pub fn set_integer(&mut self, id: usize, value: i32) {
        self.integers.insert(id, value);
    }

    #[inline]
    pub fn set_float(&mut self, id: usize, value: OrderedFloat<f32>) {
        self.floats.insert(id, value);
    }

    #[inline]
    pub fn set_string(&mut self, id: usize, value: String) {
        self.strings.insert(id, value);
    }

    #[inline]
    pub fn get_boolean(&self, id: &usize) -> bool {
        self.booleans.get(id).copied().unwrap_or_default()
    }

    #[inline]
    pub fn get_integer(&self, id: &usize) -> i32 {
        self.integers.get(id).copied().unwrap_or_default()
    }

    #[inline]
    pub fn get_float(&self, id: &usize) -> OrderedFloat<f32> {
        self.floats.get(id).copied().unwrap_or_default()
    }

    #[inline]
    pub fn get_string(&self, id: &usize) -> String {
        self.strings.get(id).cloned().unwrap_or_default()
    }
}
