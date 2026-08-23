//! The hashes the compiler names things by: the caches it may reuse an entry from, and the
//! temporary files it saves a generated source under.

/// The MD5 digest of `text`, in hexadecimal.
pub fn md5_hex(text: &str) -> String {
    format!("{:x}", md5::compute(text))
}

/// The values a hash is taken over, appended one at a time.
///
/// Each value goes in with its length in front of it, so a value cannot run into the one appended
/// next: `"xy"` followed by `"z"` gives a different source from `"x"` followed by `"yz"`. Appending
/// through this type is what holds that, and it takes the values as they come rather than keeping
/// them, so a value as large as a whole source file costs nothing to append.
#[derive(Clone)]
pub struct HashSource(md5::Context);

impl Default for HashSource {
    fn default() -> Self {
        HashSource(md5::Context::new())
    }
}

impl HashSource {
    /// Appends `text`.
    pub fn push_text(&mut self, text: &str) {
        self.0.consume(text.len().to_string());
        self.0.consume(":");
        self.0.consume(text);
    }

    /// Appends `items`. The count comes first, so a list's items cannot be read as the next list's,
    /// which is what asks the items for a length known before they are appended.
    pub fn push_list<I>(&mut self, items: I)
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
        I::IntoIter: ExactSizeIterator,
    {
        let items = items.into_iter();
        self.push_text(&items.len().to_string());
        for item in items {
            self.push_text(item.as_ref());
        }
    }

    /// The hash of everything appended.
    pub fn finish(&self) -> String {
        format!("{:x}", self.0.clone().compute())
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
                hash_source.push_list(["a", "b"]);
                hash_source.push_list::<[&str; 0]>([]);
            }),
            hash_of(|hash_source| {
                hash_source.push_list(["a"]);
                hash_source.push_list(["b"]);
            }),
            "an item belongs to the list it was appended with"
        );
        assert_ne!(
            hash_of(|hash_source| hash_source.push_text("3:abc")),
            hash_of(|hash_source| {
                hash_source.push_text("3");
                hash_source.push_text("abc");
            }),
            "a text spelling the length and the body of two values is still one value"
        );
    }
}
