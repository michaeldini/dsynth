//! Parameter system

/// Parameter ID type
pub type ParamId = u32;

/// Parameter value type
#[derive(Debug, Clone, PartialEq)]
pub enum ParamType {
    /// Floating point parameter
    Float { min: f32, max: f32, default: f32 },
    /// Boolean parameter
    Bool { default: bool },
    /// Integer parameter
    Int { min: i32, max: i32, default: i32 },
    /// Enum parameter
    Enum {
        variants: Vec<String>,
        default: usize,
    },
}

/// Parameter descriptor
#[derive(Debug, Clone)]
pub struct ParamDescriptor {
    pub id: ParamId,
    pub name: String,
    pub module: String,
    pub param_type: ParamType,
    pub unit: Option<String>,
    pub is_automatable: bool,
    pub is_hidden: bool,
}

impl ParamDescriptor {
    /// Create a float parameter
    pub fn float(
        id: ParamId,
        name: impl Into<String>,
        module: impl Into<String>,
        min: f32,
        max: f32,
        default: f32,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            module: module.into(),
            param_type: ParamType::Float { min, max, default },
            unit: None,
            is_automatable: true,
            is_hidden: false,
        }
    }

    /// Create a boolean parameter
    pub fn bool(
        id: ParamId,
        name: impl Into<String>,
        module: impl Into<String>,
        default: bool,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            module: module.into(),
            param_type: ParamType::Bool { default },
            unit: None,
            is_automatable: true,
            is_hidden: false,
        }
    }

    /// Set the unit (e.g., "Hz", "dB")
    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Mark parameter as non-automatable
    pub fn not_automatable(mut self) -> Self {
        self.is_automatable = false;
        self
    }

    /// Normalize a value to 0.0-1.0 range
    pub fn normalize(&self, value: f32) -> f32 {
        match &self.param_type {
            ParamType::Float { min, max, .. } => {
                if max == min {
                    0.0
                } else {
                    ((value - min) / (max - min)).clamp(0.0, 1.0)
                }
            }
            ParamType::Bool { .. } => value,
            ParamType::Int { min, max, .. } => {
                if max == min {
                    0.0
                } else {
                    ((value - *min as f32) / (*max - *min) as f32).clamp(0.0, 1.0)
                }
            }
            ParamType::Enum { variants, .. } => {
                if variants.len() <= 1 {
                    0.0
                } else {
                    (value / (variants.len() - 1) as f32).clamp(0.0, 1.0)
                }
            }
        }
    }

    /// Denormalize a value from 0.0-1.0 range
    pub fn denormalize(&self, normalized: f32) -> f32 {
        match &self.param_type {
            ParamType::Float { min, max, .. } => min + normalized * (max - min),
            ParamType::Bool { .. } => normalized,
            ParamType::Int { min, max, .. } => (*min as f32) + normalized * ((*max - *min) as f32),
            ParamType::Enum { variants, .. } => (normalized * (variants.len() - 1) as f32).round(),
        }
    }

    /// Get default value
    pub fn default_value(&self) -> f32 {
        match &self.param_type {
            ParamType::Float { default, .. } => *default,
            ParamType::Bool { default } => {
                if *default {
                    1.0
                } else {
                    0.0
                }
            }
            ParamType::Int { default, .. } => *default as f32,
            ParamType::Enum { default, .. } => *default as f32,
        }
    }

    /// Check if parameter is stepped (bool, int, enum)
    pub fn is_stepped(&self) -> bool {
        matches!(
            &self.param_type,
            ParamType::Bool { .. } | ParamType::Int { .. } | ParamType::Enum { .. }
        )
    }
}

/// Plugin parameters trait
pub trait PluginParams: Send + Sync + 'static {
    /// Get total number of parameters
    fn param_count() -> u32;

    /// Get parameter descriptor by index
    fn param_descriptor(index: u32) -> Option<ParamDescriptor>;

    /// Get parameter descriptor by ID
    fn param_descriptor_by_id(id: ParamId) -> Option<ParamDescriptor>;

    /// Get parameter value (normalized 0.0-1.0)
    fn get_param(id: ParamId) -> Option<f32>;

    /// Set parameter value (normalized 0.0-1.0)
    fn set_param(id: ParamId, value: f32);

    /// Save current state
    fn save_state() -> crate::PluginState;

    /// Load state
    fn load_state(state: &crate::PluginState);

    /// Get parameter value as text for display (value is normalized 0.0-1.0)
    fn format_param(_id: ParamId, value: f32) -> String {
        format!("{:.2}", value)
    }

    /// Parse parameter value from text (return normalized 0.0-1.0)
    fn parse_param(_id: ParamId, text: &str) -> Option<f32> {
        text.parse().ok()
    }
}
