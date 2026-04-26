#[derive(Debug, Default, Clone)]
pub struct ConditionValues {
    inner: Vec<bool>,
}

impl ConditionValues {
    pub fn register(&mut self, value: bool) -> usize {
        let id = self.inner.len();
        self.inner.push(value);
        id
    }

    pub fn get(&mut self, id: usize) -> &mut bool {
        &mut self.inner[id]
    }
}
