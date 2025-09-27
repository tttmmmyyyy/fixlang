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
