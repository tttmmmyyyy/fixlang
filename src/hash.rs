//! The hashes the compiler names things by: the caches it may reuse an entry from, and the
//! temporary files it saves a generated source under.

/// The MD5 digest of `text`, in hexadecimal.
pub fn md5_hex(text: &str) -> String {
    format!("{:x}", md5::compute(text))
}

/// The first eight bytes of the MD5 digest of `text`, as a number.
///
/// It serves a decision that reads a hash instead of showing one, where the text a thing is named
/// by has to decide the same way on every machine and every build.
pub fn md5_u64(text: &str) -> u64 {
    u64::from_le_bytes(md5::compute(text).0[..8].try_into().unwrap())
}

/// The values a hash is taken over, appended one at a time.
///
/// Each value goes in as a hash of its own, of a length the value does not change, so the value
/// cannot run into the one appended next: `"xy"` followed by `"z"` gives a different source from
/// `"x"` followed by `"yz"`. Appending through this type is what holds that, so the text it
/// accumulates is its own.
#[derive(Default)]
pub struct HashSource(String);

impl HashSource {
    /// Appends `text`.
    pub fn push_text(&mut self, text: &str) {
        self.0.push_str(&md5_hex(text));
    }

    /// Appends `items`. The count comes first, so a list's items cannot be read as the next list's.
    pub fn push_list(&mut self, items: &[String]) {
        self.push_text(&items.len().to_string());
        for item in items {
            self.push_text(item);
        }
    }

    /// The hash of everything appended.
    pub fn finish(&self) -> String {
        md5_hex(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::HashSource;

    /// A hash source tells apart what was appended to it: two values cannot be read as one, and one
    /// cannot be read as two, whatever the values are. Everything keyed by a hash source — the
    /// object files of a build, the type-check result of a value — leans on this.
    #[test]
    fn test_a_hash_source_separates_the_values_appended_to_it() {
        let hash_of = |append: fn(&mut HashSource)| {
            let mut hash_source = HashSource::default();
            append(&mut hash_source);
            hash_source.finish()
        };

        assert_ne!(
            hash_of(|hash_source| {
                hash_source.push_text("xy");
                hash_source.push_text("z");
            }),
            hash_of(|hash_source| {
                hash_source.push_text("x");
                hash_source.push_text("yz");
            }),
            "two texts appended one after the other are the pair of them, not the text they \
             concatenate to"
        );
        assert_ne!(
            hash_of(|hash_source| {
                hash_source.push_list(&["a".to_string(), "b".to_string()]);
                hash_source.push_list(&[]);
            }),
            hash_of(|hash_source| {
                hash_source.push_list(&["a".to_string()]);
                hash_source.push_list(&["b".to_string()]);
            }),
            "an item belongs to the list it was appended with"
        );
    }
}
