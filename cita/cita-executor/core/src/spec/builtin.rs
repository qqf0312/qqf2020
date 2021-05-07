// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Spec builtin deserialization.

/// Linear pricing.
#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct Linear {
    /// Base price.
    pub base: usize,
    /// Price for word.
    pub word: usize,
}

/// Pricing for modular exponentiation.
#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct Modexp {
    /// Price divisor.
    pub divisor: usize,
}

/// Pricing variants.
#[derive(Debug, PartialEq, Deserialize, Clone)]
pub enum Pricing {
    /// Linear pricing.
    #[serde(rename = "linear")]
    Linear(Linear),
}

/// Spec builtin.
#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct Builtin {
    /// Builtin name.
    pub name: String,
    /// Builtin pricing.
    pub pricing: Pricing,
    /// Activation block.
    pub activate_at: Option<u64>,
}

#[cfg(test)]
mod tests {
    use crate::spec::builtin::{Builtin, Linear, Pricing};
    use serde_json;

    #[test]
    fn builtin_deserialization() {
        let s = r#"{
            "name": "ecrecover",
            "pricing": { "linear": { "base": 3000, "word": 0 } }
        }"#;
        let deserialized: Builtin = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized.name, "ecrecover");
        assert_eq!(
            deserialized.pricing,
            Pricing::Linear(Linear {
                base: 3000,
                word: 0
            })
        );
        assert!(deserialized.activate_at.is_none());
    }

    #[test]
    fn activate_at() {
        let s = r#"{
            "name": "late_start",
            "activate_at": 66666,
            "pricing": { "linear": { "base": 3000, "word": 0 } }
        }"#;

        let deserialized: Builtin = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized.name, "late_start");
        assert_eq!(
            deserialized.pricing,
            Pricing::Linear(Linear {
                base: 3000,
                word: 0
            })
        );
        assert_eq!(deserialized.activate_at, Some(66666));
    }
}
