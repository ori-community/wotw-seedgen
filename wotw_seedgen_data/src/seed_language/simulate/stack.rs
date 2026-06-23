const STACK_LIMIT: usize = 1000;

#[derive(Debug, Clone)]
pub struct Stack {
    frames: Vec<StackFrame>,
    current: usize,
}

impl Default for Stack {
    fn default() -> Self {
        Self {
            frames: Vec::new(),
            current: usize::MAX,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct StackFrame {
    booleans: Vec<bool>,
    integers: Vec<i32>,
    floats: Vec<f32>,
    strings: Vec<String>,
}

impl Stack {
    #[inline]
    pub fn push(&mut self) {
        self.current = self.current.wrapping_add(1);

        assert!(self.current < STACK_LIMIT, "stack overflow");

        if self.current == self.frames.len() {
            self.frames.push(StackFrame::default());
        } else {
            self.current_frame_mut().clear();
        }
    }

    #[inline]
    pub fn pop(&mut self) {
        self.current = self.current.wrapping_sub(1);
    }

    #[inline]
    pub fn push_boolean(&mut self, value: bool) {
        self.current_frame_mut().booleans.push(value);
    }

    #[inline]
    pub fn push_integer(&mut self, value: i32) {
        self.current_frame_mut().integers.push(value);
    }

    #[inline]
    pub fn push_float(&mut self, value: f32) {
        self.current_frame_mut().floats.push(value);
    }

    #[inline]
    pub fn push_string(&mut self, value: String) {
        self.current_frame_mut().strings.push(value);
    }

    #[inline]
    pub fn get_boolean(&self, index: usize) -> bool {
        self.current_frame().booleans[index]
    }

    #[inline]
    pub fn get_integer(&self, index: usize) -> i32 {
        self.current_frame().integers[index]
    }

    #[inline]
    pub fn get_float(&self, index: usize) -> f32 {
        self.current_frame().floats[index]
    }

    #[inline]
    pub fn get_string(&self, index: usize) -> String {
        self.current_frame().strings[index].clone()
    }

    #[inline]
    fn current_frame(&self) -> &StackFrame {
        &self.frames[self.current]
    }

    #[inline]
    fn current_frame_mut(&mut self) -> &mut StackFrame {
        &mut self.frames[self.current]
    }
}

impl StackFrame {
    fn clear(&mut self) {
        self.booleans.clear();
        self.integers.clear();
        self.floats.clear();
        self.strings.clear();
    }
}
