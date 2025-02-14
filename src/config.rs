use crate::tool::Tool;

pub struct ToolConfig {
    pub python_tools: Vec<Tool>,
}

impl ToolConfig {
    pub fn new() -> Self {
        Self {
            python_tools: vec![Tool::RuffFormat, Tool::RuffCheck],
        }
    }
}
