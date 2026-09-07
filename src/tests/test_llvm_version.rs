//! The LLVM release the compiler links, and the places that tell someone how to obtain it.
//!
//! The project file pins the release through inkwell's feature. Six other places name it — the
//! environment variable the build reads, the two workflows that install it, and the installation
//! instructions of both manuals — and each of them is followed by someone who then builds the
//! compiler. A release named in one of them and not in the project file sends that person after an
//! LLVM this compiler cannot link.

#[cfg(test)]
mod tests {
    use regex::Regex;
    use std::fs;
    use std::path::{Path, PathBuf};

    /// An LLVM release as the places naming it write one: a major version, and the minor version
    /// where the place spells one out.
    type Release = (u32, Option<u32>);

    /// The path of `name` in the repository.
    ///
    /// # Parameters
    /// * `name` - The path of the file, relative to the root of the repository.
    fn repository_path(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(name)
    }

    /// The text of the file at `name` in the repository.
    ///
    /// # Parameters
    /// * `name` - The path of the file, relative to the root of the repository.
    fn text_of(name: &str) -> String {
        let path = repository_path(name);
        fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
    }

    /// Every LLVM release `text` names, in the six ways this repository writes one.
    ///
    /// # Parameters
    /// * `text` - The text to read.
    ///
    /// # Examples
    /// The feature `llvm22-1`, the tag `llvmorg-22.1.8` and the prose `LLVM 22.1.x` each yield
    /// `(22, Some(1))`; the Homebrew formula `llvm@22` yields `(22, None)`, since it names no minor
    /// version.
    fn releases_named_in(text: &str) -> Vec<Release> {
        // The order of the alternatives decides nothing: the shapes do not overlap.
        let shapes = [
            // inkwell's feature, as the project file writes it.
            r"llvm(?<major>\d+)-(?<minor>\d+)",
            // The prefix `llvm-sys` reads, whose digits are the major version and one minor digit.
            r"LLVM_SYS_(?<major>\d+)(?<minor>\d)_PREFIX",
            // The tag of a release, and the name of the archive it carries.
            r"llvmorg-(?<major>\d+)\.(?<minor>\d+)\.\d+",
            r"LLVM-(?<major>\d+)\.(?<minor>\d+)\.\d+",
            // The manuals, which name the release the compiler requires.
            r"LLVM (?<major>\d+)\.(?<minor>\d+)\.x",
            // The Homebrew formula, which is named after the major version alone.
            r"llvm@(?<major>\d+)",
        ];
        let mut releases = vec![];
        for shape in shapes {
            let pattern = Regex::new(shape).expect("the shapes are written here");
            for found in pattern.captures_iter(text) {
                let major = found["major"].parse().expect("the shape matches digits");
                let minor = found
                    .name("minor")
                    .map(|m| m.as_str().parse().expect("the shape matches digits"));
                releases.push((major, minor));
            }
        }
        releases
    }

    /// The release the project file pins, which is the one the compiler links.
    fn pinned_release() -> Release {
        let project_file = text_of("Cargo.toml");
        let pattern =
            Regex::new(r#"inkwell = \{[^}]*"llvm(\d+)-(\d+)""#).expect("the shape is written here");
        let found = pattern
            .captures(&project_file)
            .expect("`Cargo.toml` should pin inkwell at an LLVM feature");
        (
            found[1].parse().expect("the shape matches digits"),
            Some(found[2].parse().expect("the shape matches digits")),
        )
    }

    /// Every place that tells someone which LLVM to obtain names the release the project file pins.
    /// A place left behind at the previous release hands whoever follows it an LLVM the compiler
    /// cannot link, and the build fails where they have no reason to look.
    #[test]
    fn test_every_place_naming_llvm_names_the_release_the_project_file_pins() {
        let (major, minor) = pinned_release();
        for name in [
            "Cargo.toml",
            "src/build/build.rs",
            ".github/workflows/test.yml",
            ".github/workflows/release.yml",
            "Document.md",
            "Document-ja.md",
        ] {
            let named = releases_named_in(&text_of(name));
            // A shape that stops matching would leave a file silently unchecked.
            assert!(
                !named.is_empty(),
                "{} names no LLVM release, so nothing here reads it",
                name
            );
            for release in named {
                assert_eq!(
                    release.0, major,
                    "{} names LLVM {}, where `Cargo.toml` pins LLVM {}",
                    name, release.0, major
                );
                if let Some(named_minor) = release.1 {
                    assert_eq!(
                        Some(named_minor),
                        minor,
                        "{} names LLVM {}.{}, where `Cargo.toml` pins LLVM {}.{}",
                        name,
                        release.0,
                        named_minor,
                        major,
                        minor.expect("the project file spells out a minor version")
                    );
                }
            }
        }
    }
}
