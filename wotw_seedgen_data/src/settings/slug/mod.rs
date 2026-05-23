mod slugstrings;
#[cfg(test)]
mod tests;

use sha2::{Digest, Sha256};

use crate::{settings::slug::slugstrings::SLUGSTRINGS, UniverseSettings};

impl UniverseSettings {
    /// Returns a slug unique to these settings
    pub fn slugify(&self) -> String {
        let json = serde_json::to_vec(self).unwrap();
        let hash = u64::from_be_bytes(*Sha256::digest(json).first_chunk().unwrap());

        SLUGSTRINGS
            .iter()
            .enumerate()
            .map(|(index, slug_strings)| {
                let length = slug_strings.len();

                let mut shift = 1;
                loop {
                    if length < 2_usize.pow(shift) {
                        shift -= 1;
                        break;
                    }
                    shift += 1;
                }

                #[allow(clippy::cast_possible_truncation)]
                let word_index =
                    (hash >> (index as u32 * shift)) as usize & (2_usize.pow(shift) - 1);
                slug_strings[word_index]
            })
            .collect()
    }
}
