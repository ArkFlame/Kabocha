use pumpkin_data::block_rotation::Rotation;
use pumpkin_util::identifier::Identifier;
use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub enum TestType {
    #[serde(rename = "minecraft:block_based")]
    BlockBased,
    #[serde(rename = "minecraft:function")]
    Function,
}

impl TestType {
    #[must_use]
    pub const fn serialized_name(self) -> &'static str {
        match self {
            Self::BlockBased => "minecraft:block_based",
            Self::Function => "minecraft:function",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
pub enum GameTestRotation {
    #[default]
    #[serde(rename = "none")]
    None,
    #[serde(rename = "clockwise_90")]
    Clockwise90,
    #[serde(rename = "180")]
    Clockwise180,
    #[serde(rename = "counterclockwise_90")]
    Counterclockwise90,
}

impl GameTestRotation {
    #[must_use]
    pub const fn serialized_name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Clockwise90 => "clockwise_90",
            Self::Clockwise180 => "180",
            Self::Counterclockwise90 => "counterclockwise_90",
        }
    }

    #[must_use]
    pub const fn as_block_rotation(self) -> Rotation {
        match self {
            Self::None => Rotation::None,
            Self::Clockwise90 => Rotation::Clockwise90,
            Self::Clockwise180 => Rotation::Rotate180,
            Self::Counterclockwise90 => Rotation::CounterClockwise90,
        }
    }

    /// Combines the datapack's base rotation with an additional controller rotation.
    /// This mirrors vanilla `Rotation::getRotated`/`GameTestInfo` extra rotation.
    #[must_use]
    pub const fn then(self, extra: Self) -> Self {
        match self.as_block_rotation().then(extra.as_block_rotation()) {
            Rotation::None => Self::None,
            Rotation::Clockwise90 => Self::Clockwise90,
            Rotation::Rotate180 => Self::Clockwise180,
            Rotation::CounterClockwise90 => Self::Counterclockwise90,
        }
    }

    #[must_use]
    pub const fn from_steps(steps: i32) -> Self {
        match steps.rem_euclid(4) {
            0 => Self::None,
            1 => Self::Clockwise90,
            2 => Self::Clockwise180,
            _ => Self::Counterclockwise90,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct GameTestDefinition {
    #[serde(rename = "type")]
    pub instance_type: TestType,
    pub environment: Value,
    pub structure: String,
    #[serde(default)]
    pub function: Option<String>,
    pub max_ticks: i32,
    #[serde(default)]
    pub setup_ticks: i32,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub rotation: GameTestRotation,
    #[serde(default)]
    pub manual_only: bool,
    #[serde(default = "default_one")]
    pub max_attempts: i32,
    #[serde(default = "default_one")]
    pub required_successes: i32,
    #[serde(default)]
    pub sky_access: bool,
    #[serde(default)]
    pub padding: i32,
}

impl GameTestDefinition {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        let function_valid = match self.instance_type {
            TestType::BlockBased => true,
            TestType::Function => self.function.as_ref().is_some_and(|f| !f.is_empty()),
        };

        // Vanilla's TestEnvironmentDefinition.CODEC is a registry holder codec: a string
        // resource-key reference, or an inline object dispatched by its type field. Any other
        // shape fails to decode on the client.
        let environment_valid = match &self.environment {
            Value::String(id) => id.parse::<Identifier>().is_ok(),
            Value::Object(map) => map.contains_key("type"),
            _ => false,
        };

        environment_valid
            && function_valid
            && self.max_ticks > 0
            && self.setup_ticks >= 0
            && self.max_attempts > 0
            && self.required_successes > 0
            && (0..=128).contains(&self.padding)
    }
}

const fn default_true() -> bool {
    true
}

const fn default_one() -> i32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn definition(environment: Value) -> GameTestDefinition {
        serde_json::from_value(serde_json::json!({
            "type": "minecraft:function",
            "environment": environment,
            "function": "minecraft:always_pass",
            "structure": "minecraft:empty",
            "max_ticks": 1
        }))
        .expect("valid test definition")
    }

    #[test]
    fn is_valid_accepts_holder_shaped_environment() {
        assert!(definition(Value::String("minecraft:default".into())).is_valid());
        assert!(
            definition(serde_json::json!({ "type": "minecraft:weather", "weather": "clear" }))
                .is_valid()
        );
    }

    #[test]
    fn is_valid_rejects_non_holder_environment() {
        assert!(!definition(Value::Null).is_valid());
        assert!(!definition(serde_json::json!(42)).is_valid());
        assert!(!definition(serde_json::json!(true)).is_valid());
        assert!(!definition(serde_json::json!([])).is_valid());
        assert!(!definition(serde_json::json!({ "weather": "clear" })).is_valid());
    }
}
