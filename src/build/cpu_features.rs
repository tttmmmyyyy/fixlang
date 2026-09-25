use inkwell::targets::TargetMachine;
use std::env::consts::ARCH;

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

/// The CPU a program run under valgrind is built for: the architecture's baseline model, and the
/// features valgrind 3.22 decodes that LLVM uses in ordinary code. The build turns on those of the
/// features the host has, and no other.
///
/// valgrind stops a program with SIGILL at the first instruction it cannot decode, and each CPU
/// generation adds features whose instructions it lacks: AVX-512, GFNI and APX on x86-64, SVE,
/// RCpc, dot product and I8MM on AArch64. LLVM emits them in ordinary code, from vectorized loops
/// and atomic loads. Listing what valgrind decodes, rather than what it does not, leaves a feature
/// nobody has checked turned off.
pub struct ValgrindCpu {
    /// The baseline model, as LLVM names it.
    pub name: &'static str,
    /// The features valgrind decodes, as LLVM names them. The program is built with those of them
    /// the host has.
    pub decodable_features: &'static [&'static str],
}

impl ValgrindCpu {
    /// The entry for the architecture the compiler runs on and generates code for. An architecture
    /// outside x86-64 and AArch64 has none, since nobody has checked valgrind's decoder against
    /// it, and its program is built for the host's CPU as it is.
    pub fn of_this_architecture() -> Option<ValgrindCpu> {
        match ARCH {
            "x86_64" => Some(ValgrindCpu {
                name: "x86-64",
                decodable_features: &[
                    "64bit", "cmov", "cx8", "cx16", "fxsr", "mmx", "sahf", "sse", "sse2", "sse3",
                    "ssse3", "sse4.1", "sse4.2", "crc32", "popcnt", "avx", "avx2", "fma", "f16c",
                    "bmi", "bmi2", "lzcnt", "movbe", "aes", "pclmul",
                ],
            }),
            "aarch64" => Some(ValgrindCpu {
                name: "generic",
                decodable_features: &["fp-armv8", "neon", "crc", "lse", "aes", "sha2", "rdm"],
            }),
            _ => None,
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
