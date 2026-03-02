// LSP integration tests for "Find All References" and "Call Hierarchy" features.
//
// Each test case corresponds to a case in agents/test-refs.20260301/test_plan.md.
// Each symbol-type group has its own Fix project under cases/ to keep tests simple
// and resilient to line-number changes.

#[cfg(test)]
mod tests {
    use super::super::lsp_client::LspClient;
    use crate::tests::test_util::{copy_dir_recursive, install_fix};
    use serde_json::{json, Value};
    use std::{
        path::{Path, PathBuf},
        time::Duration,
    };
    use tempfile::TempDir;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn get_test_cases_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_lsp/cases");
        path
    }

    fn setup_test_env(project_name: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_case_src = get_test_cases_dir().join(project_name);
        let test_case_dst = temp_dir.path().join(project_name);
        copy_dir_recursive(&test_case_src, &test_case_dst).expect("Failed to copy test case");
        (temp_dir, test_case_dst)
    }

    /// A convenience wrapper around `LspClient` that provides high-level
    /// helpers for common test patterns (find-refs, call hierarchy, etc.).
    struct LspTestCtx {
        client: LspClient,
        project_dir: PathBuf,
        _temp_dir: TempDir,
    }

    impl LspTestCtx {
        /// Set up a project, start the LSP, open the given files and wait until
        /// the server is ready (diagnostics published).
        fn setup(project_name: &str, files: &[&str]) -> Self {
            install_fix();
            let (temp_dir, project_dir) = setup_test_env(project_name);
            let mut client = LspClient::new(&project_dir).expect("Failed to start LSP");
            client
                .initialize(&project_dir, Duration::from_secs(5))
                .expect("Failed to initialize LSP");
            for f in files {
                client
                    .open_document(Path::new(f))
                    .expect(&format!("Failed to open {}", f));
            }
            let trigger_file = files.last().unwrap();
            client.trigger_and_wait_for_diagnostics(Path::new(trigger_file));
            Self {
                client,
                project_dir,
                _temp_dir: temp_dir,
            }
        }

        fn file_uri(&self, file: &str) -> String {
            format!("file://{}", self.project_dir.join(file).display())
        }

        /// Send textDocument/references and return the result array.
        fn find_refs(&mut self, file: &str, line: u32, col: u32, include_decl: bool) -> Vec<Value> {
            let uri = self.file_uri(file);
            let id = self
                .client
                .send_request(
                    "textDocument/references",
                    json!({
                        "textDocument": { "uri": uri },
                        "position": { "line": line, "character": col },
                        "context": { "includeDeclaration": include_decl }
                    }),
                )
                .expect("Failed to send references request");
            self.client.wait_for_server(Duration::from_secs(5));
            let response = self
                .client
                .get_response(id)
                .expect("Should receive a references response");
            let result = response
                .get("result")
                .expect("Response should have a result field");
            assert!(
                result.is_array(),
                "Result should be an array, got {:?} for {}:{}:{}",
                result,
                file,
                line,
                col
            );
            result.as_array().unwrap().clone()
        }

        /// Prepare call hierarchy and return the items.
        fn prepare_call_hierarchy(&mut self, file: &str, line: u32, col: u32) -> Vec<Value> {
            let uri = self.file_uri(file);
            let id = self
                .client
                .send_request(
                    "textDocument/prepareCallHierarchy",
                    json!({
                        "textDocument": { "uri": uri },
                        "position": { "line": line, "character": col }
                    }),
                )
                .expect("Failed to send prepareCallHierarchy request");
            self.client.wait_for_server(Duration::from_secs(5));
            let response = self
                .client
                .get_response(id)
                .expect("Should receive prepareCallHierarchy response");
            let result = response
                .get("result")
                .expect("Response should have a result field");
            assert!(result.is_array(), "Result should be an array");
            result.as_array().unwrap().clone()
        }

        /// Get incoming callers for a call hierarchy item.
        fn incoming_calls(&mut self, item: &Value) -> Vec<Value> {
            let id = self
                .client
                .send_request("callHierarchy/incomingCalls", json!({ "item": item }))
                .expect("Failed to send incomingCalls request");
            self.client.wait_for_server(Duration::from_secs(5));
            let response = self
                .client
                .get_response(id)
                .expect("Should receive incomingCalls response");
            let result = response
                .get("result")
                .expect("Response should have a result field");
            assert!(result.is_array(), "Result should be an array");
            result.as_array().unwrap().clone()
        }

        /// Get outgoing calls from a call hierarchy item.
        #[allow(dead_code)]
        fn outgoing_calls(&mut self, item: &Value) -> Vec<Value> {
            let id = self
                .client
                .send_request("callHierarchy/outgoingCalls", json!({ "item": item }))
                .expect("Failed to send outgoingCalls request");
            self.client.wait_for_server(Duration::from_secs(5));
            let response = self
                .client
                .get_response(id)
                .expect("Should receive outgoingCalls response");
            let result = response
                .get("result")
                .expect("Response should have a result field");
            assert!(result.is_array(), "Result should be an array");
            result.as_array().unwrap().clone()
        }

        fn shutdown(mut self) {
            self.client
                .shutdown(Duration::from_millis(500))
                .expect("Failed to shutdown LSP");
            self.client
                .finish()
                .expect("Reader thread should not have errors");
        }

        /// Assert that the text at each reference range is a valid suffix of `full_name`.
        fn assert_ref_texts(&self, locations: &[Value], full_name: &str) {
            for loc in locations {
                let uri = loc["uri"].as_str().unwrap();
                let file_path = PathBuf::from(uri.strip_prefix("file://").unwrap());
                let range = loc.get("range").unwrap();
                let text = read_text_at_range(&file_path, range);
                assert!(
                    is_valid_name_suffix(full_name, &text),
                    "Text '{}' at reference range is not a valid suffix of '{}'. Location: {:?}",
                    text,
                    full_name,
                    loc
                );
            }
        }

        /// Assert exact count, no duplicates, and valid name text at each range.
        fn assert_refs(&self, locations: &[Value], expected: usize, full_name: &str) {
            assert_no_duplicate_refs(locations, full_name);
            assert_eq!(
                locations.len(),
                expected,
                "Expected exactly {} references to `{}`, got {}. Locations: {:?}",
                expected,
                full_name,
                locations.len(),
                locations
            );
            self.assert_ref_texts(locations, full_name);
        }
    }

    // ---- Assertion helpers ----

    /// Assert that every location in the result set is unique (no duplicates).
    fn assert_no_duplicate_refs(locations: &[Value], symbol: &str) {
        let keys: Vec<String> = locations
            .iter()
            .filter_map(|loc| {
                let uri = loc.get("uri")?.as_str()?;
                let range = loc.get("range")?;
                let start = range.get("start")?;
                let line = start.get("line")?.as_u64()?;
                let ch = start.get("character")?.as_u64()?;
                Some(format!("{}:{}:{}", uri, line, ch))
            })
            .collect();
        let mut seen = std::collections::HashSet::new();
        for key in &keys {
            assert!(
                seen.insert(key),
                "Duplicate reference to `{}` at {}. All locations: {:?}",
                symbol,
                key,
                keys
            );
        }
    }

    /// Read the text at an LSP range from a source file.
    fn read_text_at_range(file_path: &Path, range: &Value) -> String {
        let content = std::fs::read_to_string(file_path)
            .expect(&format!("Failed to read file: {:?}", file_path));
        let lines: Vec<&str> = content.lines().collect();
        let start_line = range["start"]["line"].as_u64().unwrap() as usize;
        let start_char = range["start"]["character"].as_u64().unwrap() as usize;
        let end_line = range["end"]["line"].as_u64().unwrap() as usize;
        let end_char = range["end"]["character"].as_u64().unwrap() as usize;

        if start_line == end_line {
            lines[start_line][start_char..end_char].to_string()
        } else {
            let mut text = lines[start_line][start_char..].to_string();
            for i in (start_line + 1)..end_line {
                text.push('\n');
                text.push_str(lines[i]);
            }
            text.push('\n');
            text.push_str(&lines[end_line][..end_char]);
            text
        }
    }

    /// Check if `text` is a valid suffix of `full_name` at a `::` boundary.
    /// E.g., "Vec2" is a valid suffix of "Lib::Vec2",
    /// and "MyIterator::Yielded" is a valid suffix of "Lib::MyIterator::Yielded".
    fn is_valid_name_suffix(full_name: &str, text: &str) -> bool {
        text == full_name || full_name.ends_with(&format!("::{}", text))
    }

    fn assert_has_ref_in_file(locations: &[Value], file_name: &str) {
        assert!(
            locations
                .iter()
                .any(|loc| loc
                    .get("uri")
                    .and_then(|u| u.as_str())
                    .map_or(false, |u| u.contains(file_name))),
            "Should have a reference in {}. Locations: {:?}",
            file_name,
            locations
        );
    }

    fn call_names(calls: &[Value], direction: &str) -> Vec<String> {
        calls
            .iter()
            .filter_map(|call| {
                call.get(direction)
                    .and_then(|f| f.get("name"))
                    .and_then(|n| n.as_str())
                    .map(String::from)
            })
            .collect()
    }

    fn assert_has_caller(calls: &[Value], name_fragment: &str) {
        let names = call_names(calls, "from");
        assert!(
            names.iter().any(|n| n.contains(name_fragment)),
            "Expected incoming caller containing '{}'. Found: {:?}",
            name_fragment,
            names
        );
    }

    #[allow(dead_code)]
    fn assert_has_callee(calls: &[Value], name_fragment: &str) {
        let names = call_names(calls, "to");
        assert!(
            names.iter().any(|n| n.contains(name_fragment)),
            "Expected outgoing callee containing '{}'. Found: {:?}",
            name_fragment,
            names
        );
    }

    // =======================================================================
    // GV: Global Value tests (project: refs_gv)
    // =======================================================================
    //
    // lib.fix lines (0-indexed):
    //   4: helper : I64 -> I64;          (declaration)
    //   5: helper = |x| x + 1;          (definition)
    //   9: double = |x| helper(helper(x));  (usage in GV RHS)
    //  12: truth : I64 = 42;             (combined decl+def)
    //  20: process = |n| helper(n);      (usage in trait impl RHS)
    //
    // main.fix lines (0-indexed):
    //   6: use_helper = |x| Lib::helper(x);  (cross-file usage)
    //  10: use_truth = Lib::truth + 1;       (cross-file usage)

    /// GV-1: refs from declaration LHS
    #[test]
    fn test_refs_gv1_declaration() {
        let mut ctx = LspTestCtx::setup("refs_gv", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 4, 0, true);
        ctx.assert_refs(&locs, 6, "Lib::helper");
        assert_has_ref_in_file(&locs, "lib.fix");
        assert_has_ref_in_file(&locs, "main.fix");

        let items = ctx.prepare_call_hierarchy("lib.fix", 4, 0);
        assert_eq!(items.len(), 1);
        let incoming = ctx.incoming_calls(&items[0]);
        assert_has_caller(&incoming, "double");
        assert_has_caller(&incoming, "use_helper");
        ctx.shutdown();
    }

    /// GV-2: refs from definition LHS
    #[test]
    fn test_refs_gv2_definition() {
        let mut ctx = LspTestCtx::setup("refs_gv", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 5, 0, true);
        ctx.assert_refs(&locs, 6, "Lib::helper");
        assert_has_ref_in_file(&locs, "main.fix");

        let items = ctx.prepare_call_hierarchy("lib.fix", 5, 0);
        assert_eq!(items.len(), 1);
        let incoming = ctx.incoming_calls(&items[0]);
        assert_has_caller(&incoming, "double");
        ctx.shutdown();
    }

    /// GV-3: refs from combined declaration+definition
    #[test]
    fn test_refs_gv3_combined_decl_def() {
        let mut ctx = LspTestCtx::setup("refs_gv", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 12, 0, true);
        ctx.assert_refs(&locs, 2, "Lib::truth");
        assert_has_ref_in_file(&locs, "main.fix");
        ctx.shutdown();
    }

    /// GV-4: refs from usage in GV RHS
    #[test]
    fn test_refs_gv4_usage_in_gv_rhs() {
        let mut ctx = LspTestCtx::setup("refs_gv", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 9, 13, true);
        ctx.assert_refs(&locs, 6, "Lib::helper");
        assert_has_ref_in_file(&locs, "main.fix");

        let items = ctx.prepare_call_hierarchy("lib.fix", 9, 13);
        assert_eq!(items.len(), 1);
        let incoming = ctx.incoming_calls(&items[0]);
        assert_has_caller(&incoming, "double");
        ctx.shutdown();
    }

    /// GV-5: refs from usage in trait impl RHS
    #[test]
    fn test_refs_gv5_usage_in_impl_rhs() {
        let mut ctx = LspTestCtx::setup("refs_gv", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 20, 18, true);
        ctx.assert_refs(&locs, 6, "Lib::helper");

        let items = ctx.prepare_call_hierarchy("lib.fix", 20, 18);
        assert_eq!(items.len(), 1);
        let incoming = ctx.incoming_calls(&items[0]);
        assert_has_caller(&incoming, "process");
        ctx.shutdown();
    }

    // =======================================================================
    // TM: Trait Member tests (project: refs_tm)
    // =======================================================================
    //
    // lib.fix lines (0-indexed):
    //   4: describe : a -> String;          (trait member declaration)
    //   9: describe = |n| n.to_string;      (impl member definition)
    //  14: show = |x| x.describe;           (usage in GV RHS)
    //  29: describe = |v| v.@x.my_format..  (my_format usage in impl RHS)
    //
    // main.fix lines (0-indexed):
    //   6: show_i64 = |n| n.describe;       (cross-file usage)

    /// TM-1: refs from trait member declaration
    #[test]
    fn test_refs_tm1_trait_member_declaration() {
        let mut ctx = LspTestCtx::setup("refs_tm", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 4, 4, true);
        ctx.assert_refs(&locs, 3, "Lib::Describable::describe");
        assert_has_ref_in_file(&locs, "main.fix");

        let items = ctx.prepare_call_hierarchy("lib.fix", 4, 4);
        assert_eq!(items.len(), 1);
        let incoming = ctx.incoming_calls(&items[0]);
        assert_has_caller(&incoming, "show");
        ctx.shutdown();
    }

    /// TM-2: refs from impl member definition LHS
    #[test]
    fn test_refs_tm2_impl_member_definition() {
        let mut ctx = LspTestCtx::setup("refs_tm", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 9, 4, true);
        ctx.assert_refs(&locs, 3, "Lib::Describable::describe");
        assert_has_ref_in_file(&locs, "main.fix");
        ctx.shutdown();
    }

    /// TM-3: refs from trait member usage in GV RHS
    #[test]
    fn test_refs_tm3_usage_in_gv_rhs() {
        let mut ctx = LspTestCtx::setup("refs_tm", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 14, 14, true);
        ctx.assert_refs(&locs, 3, "Lib::Describable::describe");

        let items = ctx.prepare_call_hierarchy("lib.fix", 14, 14);
        assert_eq!(items.len(), 1);
        let incoming = ctx.incoming_calls(&items[0]);
        assert_has_caller(&incoming, "show");
        ctx.shutdown();
    }

    /// TM-4: refs from trait member usage in impl RHS (my_format)
    #[test]
    fn test_refs_tm4_usage_in_impl_rhs() {
        let mut ctx = LspTestCtx::setup("refs_tm", &["lib.fix", "main.fix"]);
        // `    describe = |v| v.@x.my_format + ", " + v.@y.my_format;`
        //  col:                        ^--- col 24 = start of `my_format`
        let locs = ctx.find_refs("lib.fix", 29, 24, true);
        ctx.assert_refs(&locs, 3, "Lib::Formatter::my_format");
        ctx.shutdown();
    }

    // =======================================================================
    // Ty: Type tests (project: refs_ty)
    // =======================================================================
    //
    // lib.fix lines (0-indexed):
    //   3: type Vec2 = unbox struct { ... };      (Ty-1: type def)
    //   6: origin : Vec2;                         (Ty-2: GV type sig)
    //   7: origin = Vec2 { x: 0.0, y: 0.0 };     (Ty-3: MakeStruct)
    //  14: get_x = |Vec2 { x: x, y: _ }| x;      (Ty-4: struct pattern)
    //  18: annotated = |v| v : Vec2;               (Ty-5: expr annotation)
    //  23: let p : Vec2 = v;                       (Ty-6: pattern annotation)
    //  28: type Line = ... start: Vec2 ...         (Ty-7: type def RHS)
    //  31: type Point = Vec2;                      (Ty-8: type alias RHS)
    //  36: describe : a -> Vec2;                   (Ty-10: TM def type sig)
    //  39: impl Vec2 : Describable {               (Ty-9: impl decl type)
    //  41: describe : Vec2 -> Vec2;                (Ty-11: TM impl type sig)
    //  52: ... Container::Elem c = Vec2 ...        (Ty-12: equality RHS)
    //  59: type Elem VecArray = Vec2;              (Ty-13: assoc type impl RHS)

    /// Ty-1: refs from type definition LHS
    #[test]
    fn test_refs_ty1_type_definition() {
        let mut ctx = LspTestCtx::setup("refs_ty", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 3, 5, true);
        ctx.assert_refs(&locs, 22, "Lib::Vec2");
        assert_has_ref_in_file(&locs, "main.fix");
        ctx.shutdown();
    }

    /// Ty-2: refs from type in GV type sig
    #[test]
    fn test_refs_ty2_gv_type_sig() {
        let mut ctx = LspTestCtx::setup("refs_ty", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 6, 9, true);
        ctx.assert_refs(&locs, 22, "Lib::Vec2");
        ctx.shutdown();
    }

    /// Ty-3: refs from type in MakeStruct
    #[test]
    fn test_refs_ty3_make_struct() {
        let mut ctx = LspTestCtx::setup("refs_ty", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 7, 9, true);
        ctx.assert_refs(&locs, 22, "Lib::Vec2");
        ctx.shutdown();
    }

    /// Ty-4: refs from type in struct pattern
    #[test]
    fn test_refs_ty4_struct_pattern() {
        let mut ctx = LspTestCtx::setup("refs_ty", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 14, 9, true);
        ctx.assert_refs(&locs, 22, "Lib::Vec2");
        ctx.shutdown();
    }

    /// Ty-5: refs from type in expr annotation
    #[test]
    fn test_refs_ty5_expr_annotation() {
        let mut ctx = LspTestCtx::setup("refs_ty", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 18, 20, true);
        ctx.assert_refs(&locs, 22, "Lib::Vec2");
        ctx.shutdown();
    }

    /// Ty-6: refs from type in pattern annotation
    #[test]
    fn test_refs_ty6_pattern_annotation() {
        let mut ctx = LspTestCtx::setup("refs_ty", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 23, 12, true);
        ctx.assert_refs(&locs, 22, "Lib::Vec2");
        ctx.shutdown();
    }

    /// Ty-7: refs from type in type definition RHS (field type)
    #[test]
    fn test_refs_ty7_type_def_rhs() {
        let mut ctx = LspTestCtx::setup("refs_ty", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 28, 34, true);
        ctx.assert_refs(&locs, 22, "Lib::Vec2");
        ctx.shutdown();
    }

    /// Ty-8: refs from type in type alias RHS
    #[test]
    fn test_refs_ty8_type_alias_rhs() {
        let mut ctx = LspTestCtx::setup("refs_ty", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 31, 13, true);
        ctx.assert_refs(&locs, 22, "Lib::Vec2");
        ctx.shutdown();
    }

    /// Ty-9: refs from type in impl declaration
    #[test]
    fn test_refs_ty9_impl_declaration() {
        let mut ctx = LspTestCtx::setup("refs_ty", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 39, 5, true);
        ctx.assert_refs(&locs, 22, "Lib::Vec2");
        ctx.shutdown();
    }

    /// Ty-10: refs from type in TM definition type sig
    #[test]
    fn test_refs_ty10_tm_def_type_sig() {
        let mut ctx = LspTestCtx::setup("refs_ty", &["lib.fix", "main.fix"]);
        // `    describe : a -> Vec2;`
        //  col: 0         1         2
        //       0123456789012345678901234
        //  Vec2 starts at col 20
        let locs = ctx.find_refs("lib.fix", 36, 20, true);
        ctx.assert_refs(&locs, 22, "Lib::Vec2");
        ctx.shutdown();
    }

    /// Ty-11: refs from type in TM impl type sig
    #[test]
    fn test_refs_ty11_tm_impl_type_sig() {
        let mut ctx = LspTestCtx::setup("refs_ty", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 41, 15, true);
        ctx.assert_refs(&locs, 22, "Lib::Vec2");
        ctx.shutdown();
    }

    /// Ty-12: refs from type in Equality constraint RHS
    #[test]
    fn test_refs_ty12_equality_constraint_rhs() {
        let mut ctx = LspTestCtx::setup("refs_ty", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 52, 48, true);
        ctx.assert_refs(&locs, 22, "Lib::Vec2");
        ctx.shutdown();
    }

    /// Ty-13: refs from type in assoc type impl RHS
    #[test]
    fn test_refs_ty13_assoc_type_impl_rhs() {
        let mut ctx = LspTestCtx::setup("refs_ty", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 59, 25, true);
        ctx.assert_refs(&locs, 22, "Lib::Vec2");
        ctx.shutdown();
    }

    // =======================================================================
    // Tr: Trait tests (project: refs_tr)
    // =======================================================================
    //
    // lib.fix lines (0-indexed):
    //   3: trait a : Describable {             (Tr-1: trait def)
    //   7: impl I64 : Describable {            (Tr-3: impl decl)
    //  12: show : [a : Describable] ...        (Tr-2: GV type sig constraint)
    //  19: trait a : MyEq {                    (for Tr-4, Tr-5)
    //  25: impl [a : MyEq, b : MyEq] ...      (Tr-4: impl constraint)
    //  30: trait a : Processor {               (Tr-5)
    //  31: process : [b : MyEq] a -> b -> Bool;
    //  35: trait Printable = ToString + Describable;  (Tr-6)

    /// Tr-1: refs from trait definition name
    #[test]
    fn test_refs_tr1_trait_definition() {
        let mut ctx = LspTestCtx::setup("refs_tr", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 3, 10, true);
        ctx.assert_refs(&locs, 4, "Lib::Describable");
        assert_has_ref_in_file(&locs, "main.fix");
        ctx.shutdown();
    }

    /// Tr-2: refs from trait in GV type sig constraint
    #[test]
    fn test_refs_tr2_gv_type_sig_constraint() {
        let mut ctx = LspTestCtx::setup("refs_tr", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 12, 12, true);
        ctx.assert_refs(&locs, 4, "Lib::Describable");
        ctx.shutdown();
    }

    /// Tr-3: refs from trait in impl declaration
    #[test]
    fn test_refs_tr3_impl_declaration() {
        let mut ctx = LspTestCtx::setup("refs_tr", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 7, 11, true);
        ctx.assert_refs(&locs, 4, "Lib::Describable");
        ctx.shutdown();
    }

    /// Tr-4: refs from trait in impl constraint
    #[test]
    fn test_refs_tr4_impl_constraint() {
        let mut ctx = LspTestCtx::setup("refs_tr", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 25, 10, true);
        ctx.assert_refs(&locs, 5, "Lib::MyEq");
        ctx.shutdown();
    }

    /// Tr-5: refs from trait in TM definition type sig constraint
    #[test]
    fn test_refs_tr5_tm_def_type_sig_constraint() {
        let mut ctx = LspTestCtx::setup("refs_tr", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 31, 19, true);
        ctx.assert_refs(&locs, 5, "Lib::MyEq");
        ctx.shutdown();
    }

    /// Tr-6: refs from trait in trait alias RHS
    #[test]
    fn test_refs_tr6_trait_alias_rhs() {
        let mut ctx = LspTestCtx::setup("refs_tr", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 35, 29, true);
        ctx.assert_refs(&locs, 4, "Lib::Describable");
        ctx.shutdown();
    }

    // =======================================================================
    // TrA: Trait Alias tests (project: refs_tra)
    // =======================================================================
    //
    // lib.fix lines (0-indexed):
    //  11: trait Printable = ToString + Describable;  (TrA-1: definition)
    //  14: show : [a : Printable] a -> String;        (TrA-2: GV type sig)

    /// TrA-1: refs from trait alias definition LHS
    #[test]
    fn test_refs_tra1_trait_alias_definition() {
        let mut ctx = LspTestCtx::setup("refs_tra", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 11, 6, true);
        ctx.assert_refs(&locs, 3, "Lib::Printable");
        assert_has_ref_in_file(&locs, "main.fix");
        ctx.shutdown();
    }

    /// TrA-2: refs from trait alias in GV type sig constraint
    #[test]
    fn test_refs_tra2_gv_type_sig_constraint() {
        let mut ctx = LspTestCtx::setup("refs_tra", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 14, 12, true);
        ctx.assert_refs(&locs, 3, "Lib::Printable");
        ctx.shutdown();
    }

    // =======================================================================
    // AT: Associated Type tests (project: refs_at)
    // =======================================================================
    //
    // lib.fix lines (0-indexed):
    //   5: type Yielded iter;                                              (AT-1)
    //   6: advance : iter -> Option (iter, MyIterator::Yielded iter);      (AT-4)
    //  13: type Yielded MyIter = I64;                                      (AT-2)
    //  21: ... MyIterator::Yielded iter = I64 ...                          (AT-3)
    //  33: type Sum n m;                                                   (AT-5)

    /// AT-1: refs from assoc type declaration
    #[test]
    fn test_refs_at1_assoc_type_declaration() {
        let mut ctx = LspTestCtx::setup("refs_at", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 5, 9, true);
        ctx.assert_refs(&locs, 5, "Lib::MyIterator::Yielded");
        assert_has_ref_in_file(&locs, "main.fix");
        ctx.shutdown();
    }

    /// AT-2: refs from assoc type impl LHS
    #[test]
    fn test_refs_at2_assoc_type_impl() {
        let mut ctx = LspTestCtx::setup("refs_at", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 13, 9, true);
        ctx.assert_refs(&locs, 5, "Lib::MyIterator::Yielded");
        ctx.shutdown();
    }

    /// AT-3: refs from assoc type in Equality constraint
    #[test]
    fn test_refs_at3_equality_constraint() {
        let mut ctx = LspTestCtx::setup("refs_at", &["lib.fix", "main.fix"]);
        // `sum_iter : [iter : MyIterator, MyIterator::Yielded iter = I64] ...`
        //  col:       0         1         2         3         4
        //             01234567890123456789012345678901234567890123456789
        // `MyIterator::Yielded` starts at col 31, `Yielded` at col 43
        let locs = ctx.find_refs("lib.fix", 21, 43, true);
        ctx.assert_refs(&locs, 5, "Lib::MyIterator::Yielded");
        ctx.shutdown();
    }

    /// AT-4: refs from assoc type usage in type
    #[test]
    fn test_refs_at4_usage_in_type() {
        let mut ctx = LspTestCtx::setup("refs_at", &["lib.fix", "main.fix"]);
        // `    advance : iter -> Option (iter, MyIterator::Yielded iter);`
        //  col: 0         1         2         3         4         5
        //       0123456789012345678901234567890123456789012345678901234567
        // `MyIterator::Yielded` starts at col 36, `Yielded` at col 48
        let locs = ctx.find_refs("lib.fix", 6, 48, true);
        ctx.assert_refs(&locs, 5, "Lib::MyIterator::Yielded");
        ctx.shutdown();
    }

    /// AT-5: refs for higher arity associated type
    #[test]
    fn test_refs_at5_higher_arity() {
        let mut ctx = LspTestCtx::setup("refs_at", &["lib.fix", "main.fix"]);
        let locs = ctx.find_refs("lib.fix", 33, 9, true);
        ctx.assert_refs(&locs, 2, "Lib::Nat::Sum");
        ctx.shutdown();
    }
}
