use crate::tool::Tool;

pub struct ToolConfig {
    pub python_tools: Vec<Box<dyn Tool + Send + Sync>>,
}

impl ToolConfig {
    pub fn new() -> Self {
        Self {
            python_tools: vec![
                Box::new(crate::tool::RuffFormat),
                Box::new(crate::tool::RuffCheck),
            ],
        }
    }
}
