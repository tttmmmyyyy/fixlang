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

    pub fn disable_avx512(&mut self) {
        for (name, state) in &mut self.data {
            if name.starts_with("avx512") {
                *state = FeatureState::Disabled;
            }
        }
    }
}
