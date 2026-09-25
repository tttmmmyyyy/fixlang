use inkwell::targets::TargetMachine;

/// The CPU of the machine a build runs on, as LLVM names it.
///
/// A build compiles for the machine it runs on: the code is generated for this CPU, or under
/// valgrind for a baseline model with a part of its features (`Configuration::target_cpu_name` and
/// `Configuration::target_cpu_features`), so an object file holds only instructions this CPU
/// has.
#[derive(Clone, PartialEq, Eq)]
pub struct HostCpu {
    /// The model name, such as `skylake-avx512`.
    pub name: String,
    /// The features the CPU supports, as a comma-separated list of `+name` and `-name`.
    pub features: String,
}

impl HostCpu {
    /// The CPU this compiler is running on.
    pub fn of_this_machine() -> HostCpu {
        HostCpu {
            name: TargetMachine::get_host_cpu_name().to_string(),
            features: TargetMachine::get_host_cpu_features().to_string(),
        }
    }
}

// A struct to parse and manipulate CPU features obtained by `TargetMachine::get_host_cpu_features()`.
pub struct CpuFeatures {
    data: Vec<(String, FeatureState)>,
}

enum FeatureState {
    Enabled,
    Disabled,
}

impl CpuFeatures {
    pub fn parse(features: &str) -> CpuFeatures {
        let mut data = vec![];
        for feature in features.split(',') {
            if feature.len() == 0 {
                continue;
            }
            // `feature` is in the form of `(+/-)name`.
            let (state, name) = feature.split_at(1);
            let state = match state {
                "+" => FeatureState::Enabled,
                "-" => FeatureState::Disabled,
                _ => panic!("Invalid feature state: {}", state),
            };
            data.push((name.to_string(), state));
        }
        CpuFeatures { data }
    }

    pub fn to_string(&self) -> String {
        self.data
            .iter()
            .map(|(name, state)| match state {
                FeatureState::Enabled => format!("+{}", name),
                FeatureState::Disabled => format!("-{}", name),
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Keeps only the features the list turns on and `names` holds, dropping every other entry.
    /// Given the baseline CPU model, the features the list turns on are then the whole of what the
    /// code may use.
    pub fn keep_enabled_only(&mut self, names: &[&str]) {
        self.data.retain(|(name, state)| {
            matches!(state, FeatureState::Enabled) && names.contains(&name.as_str())
        });
    }

    // Disable CPU features whose names match any of the given regexes.
    pub fn disable_by_regexes(&mut self, regexes: &[String]) {
        // All regexes are valid because they are validated by `validate_disable_cpu_features()`
        let regexes = regexes
            .iter()
            .map(|s| regex::Regex::new(s).unwrap())
            .collect::<Vec<_>>();
        for (name, state) in &mut self.data {
            for re in &regexes {
                if re.is_match(name) {
                    *state = FeatureState::Disabled;
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CpuFeatures;

    /// Keeping a set of features keeps those the list turns on, in their order, and drops the rest,
    /// the features the list turns off included.
    #[test]
    fn test_keep_enabled_only_keeps_the_enabled_features_named() {
        let mut features = CpuFeatures::parse("+avx2,+gfni,-sse4a,+sse2,-avx512f");
        features.keep_enabled_only(&["sse2", "avx2", "sse4a"]);
        assert_eq!(features.to_string(), "+avx2,+sse2");
    }
}
