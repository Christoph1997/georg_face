use std::str::FromStr;

/// Face state selected from the ROS 2 emotion topic.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Emotion {
    #[default]
    Idle,
    Happy,
    Sleepy,
    Surprised,
}

impl Emotion {
    pub const ALL: [Emotion; 4] = [
        Emotion::Idle,
        Emotion::Happy,
        Emotion::Sleepy,
        Emotion::Surprised,
    ];
}

impl FromStr for Emotion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "idle" | "neutral" => Ok(Emotion::Idle),
            "happy" => Ok(Emotion::Happy),
            "sleepy" | "tired" => Ok(Emotion::Sleepy),
            "surprised" => Ok(Emotion::Surprised),
            other => Err(format!("unknown emotion '{other}'")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_case_insensitive() {
        assert_eq!("HAPPY".parse::<Emotion>(), Ok(Emotion::Happy));
        assert_eq!(" Sleepy ".parse::<Emotion>(), Ok(Emotion::Sleepy));
    }

    #[test]
    fn unknown_is_error() {
        assert!("angry-ish".parse::<Emotion>().is_err());
    }
}
