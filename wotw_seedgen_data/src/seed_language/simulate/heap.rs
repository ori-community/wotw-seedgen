use rustc_hash::FxHashMap;

#[derive(Debug, Default, Clone)]
pub struct Heap {
    booleans: FxHashMap<usize, bool>,
    integers: FxHashMap<usize, i32>,
    floats: FxHashMap<usize, f32>,
    strings: FxHashMap<usize, String>,
}

impl Heap {
    #[inline]
    pub fn set_boolean(&mut self, id: usize, value: bool) {
        self.booleans.insert(id, value);
    }

    #[inline]
    pub fn set_integer(&mut self, id: usize, value: i32) {
        self.integers.insert(id, value);
    }

    #[inline]
    pub fn set_float(&mut self, id: usize, value: f32) {
        self.floats.insert(id, value);
    }

    #[inline]
    pub fn set_string(&mut self, id: usize, value: String) {
        self.strings.insert(id, value);
    }

    #[inline]
    pub fn get_boolean(&self, id: usize) -> bool {
        self.booleans.get(&id).copied().unwrap_or_default()
    }

    #[inline]
    pub fn get_integer(&self, id: usize) -> i32 {
        self.integers.get(&id).copied().unwrap_or_default()
    }

    #[inline]
    pub fn get_float(&self, id: usize) -> f32 {
        self.floats.get(&id).copied().unwrap_or_default()
    }

    #[inline]
    pub fn get_string(&self, id: usize) -> String {
        self.strings.get(&id).cloned().unwrap_or_default()
    }
}

// TODO would've liked to try this out by benchmarking launch_fragments but will have to do that later, see comment in the benchmark
// use std::{cmp::Ordering, iter};

// #[derive(Debug, Default, Clone)]
// pub struct Heap {
//     booleans: Vec<bool>,
//     integers: Vec<i32>,
//     floats: Vec<f32>,
//     strings: Vec<String>,
// }

// impl Heap {
//     #[inline]
//     pub fn set_boolean(&mut self, id: usize, value: bool) {
//         set(&mut self.booleans, id, value);
//     }

//     #[inline]
//     pub fn set_integer(&mut self, id: usize, value: i32) {
//         set(&mut self.integers, id, value);
//     }

//     #[inline]
//     pub fn set_float(&mut self, id: usize, value: f32) {
//         set(&mut self.floats, id, value);
//     }

//     #[inline]
//     pub fn set_string(&mut self, id: usize, value: String) {
//         set(&mut self.strings, id, value);
//     }

//     #[inline]
//     pub fn get_boolean(&self, id: usize) -> bool {
//         self.booleans.get(id).copied().unwrap_or_default()
//     }

//     #[inline]
//     pub fn get_integer(&self, id: usize) -> i32 {
//         self.integers.get(id).copied().unwrap_or_default()
//     }

//     #[inline]
//     pub fn get_float(&self, id: usize) -> f32 {
//         self.floats.get(id).copied().unwrap_or_default()
//     }

//     // TODO ref not clone?
//     #[inline]
//     pub fn get_string(&self, id: usize) -> String {
//         self.strings.get(id).cloned().unwrap_or_default()
//     }
// }

// fn set<T: Clone + Default>(vec: &mut Vec<T>, id: usize, value: T) {
//     let len = vec.len();
//     match id.cmp(&len) {
//         Ordering::Less => vec[id] = value,
//         Ordering::Equal => vec.push(value),
//         Ordering::Greater => {
//             vec.reserve(id - len);
//             vec.extend(iter::repeat_with(T::default).take(id - len - 1));
//             vec.push(value);
//         }
//     }
// }
