use crate::ast::expr::Var;
use crate::ast::name::{FullName, Name, NameSpace};
use crate::ast::program::EndNode;
use crate::constants::{FORMAT_LINE_LIMIT, STD_NAME};
use crate::parse::parser::{is_token_of, TokenCategory};
use crate::parse::sourcefile::{SourcePos, Span};
use crate::printer::Text;
use std::cmp::Ordering;

/// Whether any statement of `stmts` makes `name` accessible in the module they are written in.
pub fn is_accessible(stmts: &[ImportStatement], name: &FullName) -> bool {
    stmts.iter().any(|stmt| stmt.is_accessible(name))
}

/// An `import` statement: the module it brings in, the items it takes from that module, and the
/// items it keeps out.
#[derive(Clone)]
pub struct ImportStatement {
    /// The module the statement is written in.
    pub importer: Name,
    /// The imported module's name.
    pub module_name: Name,
    /// The source span of the token that named the imported module: the `Mod` of an `import Mod;`
    /// written in source, or the `Mod` of a `::Mod::name` written as an absolute path. The imports
    /// a module gets of itself and of `Std` carry none, since no token names the module there.
    pub module_span: Option<Span>,
    /// The items brought in, each as one path under the imported module.
    pub items: Vec<ImportTreeNode>,
    /// The items the `hiding` clause keeps out, even where `items` covers them.
    pub hiding: Vec<ImportTreeNode>,
    /// The span of the whole `import ...;` statement in source. A statement the compiler added
    /// carries none, since no statement stands for it in source; the span of the token that led to
    /// it is in `module_span`.
    pub source: Option<Span>,
    /// Whether the compiler added the statement rather than the source: every module imports itself
    /// and `Std` this way.
    pub implicit: bool,
}

impl ImportStatement {
    /// Orders `stmts` by the name of the module each imports, and the items of each statement by
    /// their kind and name.
    pub fn sort(stmts: &mut [ImportStatement]) {
        stmts.sort_by(|a, b| a.module_name.cmp(&b.module_name));
        for stmt in stmts {
            ImportTreeNode::sort(&mut stmt.items);
            ImportTreeNode::sort(&mut stmt.hiding);
        }
    }

    /// The entity the statement names at `pos`: the imported module, or the item the path written
    /// there reaches.
    pub fn find_node_at(&self, pos: &SourcePos) -> Option<EndNode> {
        let span = self.module_span.as_ref()?;
        if span.includes_pos_lsp(pos) {
            return Some(EndNode::Module(self.module_name.clone()));
        }
        let namespace = NameSpace::new(vec![self.module_name.clone()]);
        for item in &self.items {
            let node = item.find_node_at(pos, &namespace);
            if node.is_some() {
                return node;
            }
        }
        for item in &self.hiding {
            let node = item.find_node_at(pos, &namespace);
            if node.is_some() {
                return node;
            }
        }
        return None;
    }

    /// Whether this statement makes `name` accessible: `name` lies in the imported module, an item
    /// of the statement covers it, and the `hiding` clause leaves it in.
    pub fn is_accessible(&self, name: &FullName) -> bool {
        if name.module() != self.module_name {
            return false;
        }
        let mut name = name.clone();
        assert!(name.pop_front_namespace());
        if self
            .hiding
            .iter()
            .any(|hidden_item| hidden_item.is_accessible(&name))
        {
            return false;
        }
        self.items.iter().any(|item| item.is_accessible(&name))
    }

    /// The statement by which `module` brings in every item of its own.
    pub fn implicit_self_import(module: Name) -> ImportStatement {
        ImportStatement {
            importer: module.clone(),
            module_name: module,
            module_span: None,
            items: vec![ImportTreeNode::Any(None)],
            hiding: vec![],
            source: None,
            implicit: true,
        }
    }

    /// The statement by which `module` brings in every item of `Std`.
    pub fn implicit_std_import(module: Name) -> ImportStatement {
        ImportStatement {
            importer: module,
            module_name: STD_NAME.to_string(),
            module_span: None,
            items: vec![ImportTreeNode::Any(None)],
            hiding: vec![],
            source: None,
            implicit: true,
        }
    }

    /// A statement bringing `name` alone into `importer`.
    ///
    /// # Examples
    /// The statement for `Lib::Ns::f` stringifies to `import Lib::Ns::f;`.
    pub fn import_to_use(importer: Name, name: FullName) -> ImportStatement {
        Self::import_to_use_with_spans(importer, name, &[])
    }

    /// A statement bringing `name` alone into `importer`, carrying the source spans of the tokens
    /// that named it.
    ///
    /// # Arguments
    /// * `path_spans` — the span of each name along `name`'s whole path, `[module, namespaces...,
    ///   name]`. The first becomes the span of the imported module, and each of the rest the span
    ///   of the `ImportTreeNode` that name becomes. Names a shorter `path_spans` leaves uncovered
    ///   carry no span.
    // PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn import_to_use_with_spans(
        importer: Name,
        name: FullName,
        path_spans: &[Span],
    ) -> ImportStatement {
        let module_span = path_spans.first().cloned();
        let item_spans = path_spans.get(1..).unwrap_or(&[]);
        let module = name.module();
        let mut names = name.to_namespace().names.clone();
        let names = names.split_off(1);
        ImportStatement {
            importer,
            module_name: module,
            module_span,
            items: vec![ImportTreeNode::from_names_with_spans(&names, item_spans)],
            hiding: vec![],
            source: None,
            implicit: false,
        }
    }

    /// The items the statement names, each qualified by the imported module. The items of the
    /// `hiding` clause are among them.
    pub fn referred_items(&self) -> Vec<ImportItem> {
        let mut result = vec![];
        for item in &self.items {
            result.append(&mut item.to_items());
        }
        for item in &self.hiding {
            result.append(&mut item.to_items());
        }
        for item in &mut result {
            item.push_front(self.module_name.clone());
        }
        result
    }

    /// The statement as it is written in source, such as `import Lib::{f, g};`.
    pub fn stringify(&self) -> String {
        self.stringify_internal().to_string()
    }

    /// The text of the statement, broken across lines where a long list of items needs it.
    fn stringify_internal(&self) -> Text {
        let text = Text::from_str("import ");
        let text = text.append_to_last_line(&self.module_name);
        let text = if self.items.len() == 0 {
            text.append_to_last_line("::{}")
        } else {
            text
        };
        let text = if self.items.len() == 1 && matches!(self.items[0], ImportTreeNode::Any(_)) {
            // For example, "import Std::*" should be written as "import Std"
            text
        } else {
            if self.items.len() >= 1 {
                let text = text.append_to_last_line("::");
                let items_text = Text::join(
                    self.items
                        .iter()
                        .map(|item| item.stringify())
                        .collect::<Vec<_>>(),
                    ", ",
                    FORMAT_LINE_LIMIT,
                );
                let needs_brace = self.items.len() >= 2;
                let items_text = if needs_brace {
                    items_text.curly_brace()
                } else {
                    items_text
                };
                let text = text.append_nobreak(items_text);
                text
            } else {
                text
            }
        };
        let text = if self.hiding.len() >= 1 {
            let text = text.append_to_last_line(" hiding ");
            let hiding_text = Text::join(
                self.hiding
                    .iter()
                    .map(|item| item.stringify())
                    .collect::<Vec<_>>(),
                ", ",
                FORMAT_LINE_LIMIT,
            );
            let needs_brace = self.hiding.len() >= 2;
            let hiding_text = if needs_brace {
                hiding_text.curly_brace()
            } else {
                hiding_text
            };
            let text = text.append_nobreak(hiding_text);
            text
        } else {
            text
        };
        let text = text.append_to_last_line(";");
        text
    }

    /// Makes `name` accessible from `importer`: it is added to a statement of `imports` that brings
    /// in `name`'s module and carries no `hiding` clause, and becomes a statement of its own where
    /// `imports` holds no such statement. An `imports` that already reaches `name` is left as it
    /// stands.
    // PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn add_import(imports: &mut Vec<ImportStatement>, importer: Name, name: FullName) {
        // If it's already accessible, do nothing.
        if is_accessible(&imports, &name) {
            return;
        }
        // Find an import statement which has the same module name to `name` and has no hiding items.
        let module = name.module();
        let import = imports
            .iter()
            .enumerate()
            .find(|(_i, import)| import.module_name == module && import.hiding.is_empty());
        // If there is no such import, create a new one.
        if import.is_none() {
            let new_import = ImportStatement::import_to_use(importer, name);
            imports.push(new_import);
            return;
        }
        // If found, add the name to the import statement.
        let idx = import.unwrap().0;
        let import = &mut imports[idx];
        import.add_item(&name.to_namespace().names[1..]);
    }

    /// Adds the item at the path `names`, which runs from inside the imported module, to the items
    /// brought in. `names` holds at least one name, and the statement carries no `hiding` clause.
    fn add_item(&mut self, names: &[Name]) {
        assert!(names.len() >= 1);
        assert!(self.hiding.is_empty());
        ImportTreeNode::add_item_internal(&mut self.items, names);
    }
}

/// A node of the tree an import path spells out under the imported module, as `Lib::{f, Ns::g}`
/// does. Each node carries the source span of the token that named it; the nodes of a statement the
/// compiler added carry none.
#[derive(Clone)]
pub enum ImportTreeNode {
    /// The `*` of `import Lib::*`, standing for every item of the namespace the node sits in.
    Any(Option<Span>),
    /// A value's name.
    Symbol(Name, Option<Span>),
    /// A type's or a trait's name.
    TypeOrTrait(Name, Option<Span>),
    /// A namespace's name, and the items taken from within it.
    NameSpace(Name, Vec<ImportTreeNode>, Option<Span>),
}

impl ImportTreeNode {
    /// The entity this node names at `pos`.
    ///
    /// # Arguments
    /// * `namespace` — the path the walk down the tree has reached, which qualifies the name found
    ///   here.
    fn find_node_at(&self, pos: &SourcePos, namespace: &NameSpace) -> Option<EndNode> {
        match self {
            ImportTreeNode::Any(_span) => {}
            ImportTreeNode::Symbol(name, span) => {
                if span.is_none() {
                    return None;
                }
                let span = span.as_ref().unwrap();
                if !span.includes_pos_lsp(pos) {
                    return None;
                }
                let name = FullName::new(namespace, name);
                let var = Var::create(name);
                return Some(EndNode::Expr(var, None));
            }
            ImportTreeNode::TypeOrTrait(name, span) => {
                if span.is_none() {
                    return None;
                }
                let span = span.as_ref().unwrap();
                if !span.includes_pos_lsp(pos) {
                    return None;
                }
                let name = FullName::new(namespace, name);
                return Some(EndNode::TypeOrTrait(name));
            }
            ImportTreeNode::NameSpace(name, items, span) => {
                if span.is_none() {
                    return None;
                }
                let span = span.as_ref().unwrap();
                if !span.includes_pos_lsp(pos) {
                    return None;
                }
                let mut namespace = namespace.clone();
                namespace.push_back(name.clone());
                for item in items {
                    let node = item.find_node_at(pos, &namespace);
                    if node.is_some() {
                        return node;
                    }
                }
            }
        }
        None
    }

    /// Orders `nodes` by kind and by name within a kind, and orders the items of each namespace the
    /// same way.
    fn sort(nodes: &mut [ImportTreeNode]) {
        nodes.sort_by(|a, b| {
            // Any < Symbol (cmp by name) < TypeOrTrait (cmp by name) < Namespace (cmp by name)
            match (a, b) {
                (ImportTreeNode::Any(_), ImportTreeNode::Any(_)) => Ordering::Equal,
                (ImportTreeNode::Any(_), _) => Ordering::Less,
                (_, ImportTreeNode::Any(_)) => Ordering::Greater,
                (ImportTreeNode::Symbol(name_a, _), ImportTreeNode::Symbol(name_b, _)) => {
                    name_a.cmp(name_b)
                }
                (ImportTreeNode::Symbol(_, _), _) => Ordering::Less,
                (_, ImportTreeNode::Symbol(_, _)) => Ordering::Greater,
                (
                    ImportTreeNode::TypeOrTrait(name_a, _),
                    ImportTreeNode::TypeOrTrait(name_b, _),
                ) => name_a.cmp(name_b),
                (ImportTreeNode::TypeOrTrait(_, _), _) => Ordering::Less,
                (_, ImportTreeNode::TypeOrTrait(_, _)) => Ordering::Greater,
                (
                    ImportTreeNode::NameSpace(name_a, _, _),
                    ImportTreeNode::NameSpace(name_b, _, _),
                ) => name_a.cmp(name_b),
            }
        });
        for node in nodes {
            if let ImportTreeNode::NameSpace(_, items, _) = node {
                Self::sort(items);
            }
        }
    }

    /// The tree taking `names` as one path: every name but the last becomes a namespace holding the
    /// next, and the last becomes the item its head character tells it to be. An empty `names`
    /// becomes `Any`, the `*` of `import Lib::*`.
    ///
    /// # Examples
    /// `from_names(&["A", "B", "f"])` is `NameSpace("A", [NameSpace("B", [Symbol("f")])])`.
    fn from_names(names: &[Name]) -> ImportTreeNode {
        Self::from_names_with_spans(names, &[])
    }

    /// The tree taking `names` as one path, with `spans[i]` as the source span of the node
    /// `names[i]` becomes. Names a shorter `spans` leaves uncovered carry no span, and an empty
    /// `names` becomes `Any`, the `*` of `import Lib::*`.
    ///
    /// # Examples
    /// `from_names_with_spans(&["A", "f"], &[])` is `NameSpace("A", [Symbol("f")])`.
    fn from_names_with_spans(names: &[Name], spans: &[Span]) -> ImportTreeNode {
        let head_span = spans.first().cloned();
        let tail_spans = spans.get(1..).unwrap_or(&[]);
        if names.is_empty() {
            return ImportTreeNode::Any(head_span);
        }
        if names.len() == 1 {
            let name = &names[0];
            // The shape of the head character is what tells a type or trait from a value, so a name
            // with no character at all has no node to become.
            assert!(
                !name.is_empty(),
                "An import item is named by an empty name."
            );
            if is_token_of(name, TokenCategory::CapitalName) {
                return ImportTreeNode::TypeOrTrait(name.clone(), head_span);
            }
            return ImportTreeNode::Symbol(name.clone(), head_span);
        }
        let namespace = &names[0];
        ImportTreeNode::NameSpace(
            namespace.clone(),
            vec![ImportTreeNode::from_names_with_spans(
                &names[1..],
                tail_spans,
            )],
            head_span,
        )
    }

    /// Adds the path `names`, which runs from inside this namespace, to the items taken from it.
    /// `names` holds at least one name, and this node is a `NameSpace`.
    fn add_item(&mut self, names: &[Name]) {
        assert!(names.len() >= 1);
        assert!(matches!(self, ImportTreeNode::NameSpace(_, _, _)));
        if let ImportTreeNode::NameSpace(_, items, _) = self {
            ImportTreeNode::add_item_internal(items, names);
            return;
        } else {
            unreachable!()
        }
    }

    /// Adds the path `names` to `items`, walking into the namespace node that already carries the
    /// head name where `items` holds one. An item `items` already names is left as it stands.
    /// `names` holds at least one name.
    fn add_item_internal(items: &mut Vec<ImportTreeNode>, names: &[Name]) {
        assert!(names.len() >= 1);
        if names.len() >= 2 {
            let namespace = &names[0];
            // If `items` already includes the `namespace`, then add the `names` to it.
            if let Some(item) = items.iter_mut().find(|item| match item {
                ImportTreeNode::NameSpace(name, _, _) => name == namespace,
                _ => false,
            }) {
                item.add_item(&names[1..]);
                return;
            }
            // If `items` does not include the `namespace`, then add the `names`.
            items.push(ImportTreeNode::from_names(names));
            return;
        }
        // `names` has no namespace.
        let name = &names[0];
        // If `name` is already included in `items`, do nothing.
        if items.iter().any(|item| match item {
            ImportTreeNode::Symbol(item_name, _) => item_name == name,
            ImportTreeNode::TypeOrTrait(item_name, _) => item_name == name,
            _ => false,
        }) {
            return;
        }
        // If `name` is not already included in `items`, then add it.
        items.push(ImportTreeNode::from_names(names));
    }

    /// Whether this node covers `name`, a name written relative to the namespace the node sits in.
    pub fn is_accessible(&self, name: &FullName) -> bool {
        match self {
            ImportTreeNode::Any(_) => true,
            ImportTreeNode::Symbol(item_name, _) => name.is_local() && name.name == *item_name,
            ImportTreeNode::TypeOrTrait(item_name, _) => name.is_local() && name.name == *item_name,
            ImportTreeNode::NameSpace(namespace, items, _) => {
                if name.is_local() {
                    return false;
                }
                if name.namespace.names[0] != *namespace {
                    return false;
                }
                let mut name = name.clone();
                assert!(name.pop_front_namespace());
                items.iter().any(|item| item.is_accessible(&name))
            }
        }
    }

    /// The items this tree names, each carrying the path from here down to it.
    ///
    /// # Examples
    /// The items of `NameSpace("A", [Symbol("f"), TypeOrTrait("T")])` are the value `A::f` and the
    /// type or trait `A::T`.
    fn to_items(&self) -> Vec<ImportItem> {
        match self {
            ImportTreeNode::Any(src) => {
                vec![ImportItem::NameSpace(NameSpace::new(vec![]), src.clone())]
            }
            ImportTreeNode::Symbol(name, src) => {
                vec![ImportItem::Symbol(FullName::local(name), src.clone())]
            }
            ImportTreeNode::TypeOrTrait(name, src) => {
                vec![ImportItem::TypeOrTrait(FullName::local(name), src.clone())]
            }
            ImportTreeNode::NameSpace(name, items, _src) => {
                let mut result = vec![];
                for item in items {
                    let mut children = item.to_items();
                    for child in &mut children {
                        child.push_front(name.clone());
                    }
                    result.append(&mut children);
                }
                result
            }
        }
    }

    /// The node as it is written in an import path, such as `Ns::{f, g}`, broken across lines where
    /// a long list of items needs it.
    fn stringify(&self) -> Text {
        match self {
            ImportTreeNode::Any(_) => Text::from_str("*"),
            ImportTreeNode::Symbol(name, _) => Text::from_str(name),
            ImportTreeNode::TypeOrTrait(name, _) => Text::from_str(name),
            ImportTreeNode::NameSpace(name, items, _) => {
                let text = Text::from_str(name);
                let text = if items.len() >= 1 {
                    let text = text.append_to_last_line("::");
                    let items_text = Text::join(
                        items
                            .iter()
                            .map(|item| item.stringify())
                            .collect::<Vec<_>>(),
                        ", ",
                        FORMAT_LINE_LIMIT,
                    );
                    let needs_brace = items.len() >= 2;
                    let items_text = if needs_brace {
                        items_text.curly_brace()
                    } else {
                        items_text
                    };
                    let text = text.append_nobreak(items_text);
                    text
                } else {
                    text
                };
                text
            }
        }
    }
}

/// An item an import statement names — a value, a type or trait, or a namespace — with the source
/// span of the token that named it.
pub enum ImportItem {
    /// A value's name.
    Symbol(FullName, Option<Span>),
    /// A type's or a trait's name.
    TypeOrTrait(FullName, Option<Span>),
    /// A namespace, every item of which the statement names.
    NameSpace(NameSpace, Option<Span>),
}

impl ImportItem {
    /// Puts `name` before the item's path, qualifying it by one more namespace.
    pub fn push_front(&mut self, name: Name) {
        match self {
            ImportItem::Symbol(fullname, _) => {
                fullname.push_front(name);
            }
            ImportItem::TypeOrTrait(fullname, _) => {
                fullname.push_front(name);
            }
            ImportItem::NameSpace(namespace, _) => {
                namespace.push_front(name);
            }
        }
    }
}
