use std::ops::Range;

use rand::Rng;

use crate::util::extensions::StrExtension;

use super::trim_comment;

type Replacements = Vec<(Range<usize>, String)>;
fn determine_replacements(input: &str, rng: &mut impl Rng) -> Result<Replacements, String> {
    let mut pool = vec![];
    let mut replacements = vec![];

    for range in input.line_ranges() {
        let line = &input[range.clone()];

        let mut last_take_index = 0;
        while let Some(mut take_index) = line[last_take_index..].find("!!take") {
            if pool.is_empty() {
                return Err("Cannot !!take on an empty pool. Use !!pool first".to_string());
            }

            take_index += last_take_index;
            let after_match = take_index + 6;
            last_take_index = after_match;

            let random_item = pool.remove(rng.gen_range(0..pool.len()));
            replacements.push((range.start + take_index..range.start + after_match, random_item));
        }

        if let Some(pool_item) = line.strip_prefix("!!pool ") {
            let pool_item = trim_comment(pool_item).trim_start().to_string();
            pool.push(pool_item);
            replacements.push((range, String::new()));
        } else if let Some(should_be_empty) = line.strip_prefix("!!flush") {
            if !trim_comment(should_be_empty).is_empty() {
                continue;
            }
            pool.clear();
            replacements.push((range, String::new()));
        }
    }

    Ok(replacements)
}

fn apply_replacements(input: &mut String, replacements: Replacements) {
    // Traverse in reverse direction so the offsets caused by replacements don't invalidate our ranges
    for (range, replace_with) in replacements.into_iter().rev() {
        input.replace_range(range, &replace_with);
    }
}

/// Process all `!!pool`, `!!flush` and `!!take` statements before evaluation of the syntax
/// 
/// `!!pool` and `!!flush` will only be accepted at the start of lines, `!!take` can be in any location
pub(crate) fn preprocess(input: &mut String, rng: &mut impl Rng) -> Result<(), String> {
    let replacements = determine_replacements(input, rng)?;
    apply_replacements(input, replacements);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_take() {
        let mut input = format!("{}\n{}\n{}",
            "!!pool happy",
            "!!pool sad",
            "3|0|6|Today's mood: !!take",
        );
        let mut rng = rand::thread_rng();

        preprocess(&mut input, &mut rng).unwrap();

        assert!(input == "3|0|6|Today's mood: happy" || input == "3|0|6|Today's mood: sad");
    }
}
