// Rewriting a program by its lambda sets: the copies of the symbols and the type constructors the
// sets ask for, the lambdas lifted to global functions, and each value of a set of one lambda
// represented by that lambda's capture struct. A lambda of a set represented by closures is lifted
// too, and its closure is the lifted function applied to the capture struct.

use super::{
    concrete::SetId,
    infer::{Analysis, NameRef, NodeType},
    represent::{Repr, Representation},
    slots::{AType, SlotFold, SlotLayout},
    solver::{LamId, SetVar},
    specialize::Specialization,
};
use crate::{
    ast::{
        expr::{
            expr_abs_typed, expr_app_typed, expr_let_typed, expr_var, var_local,
            Expr, ExprNode,
        },
        name::FullName,
        pattern::{Pattern, PatternNode},
        program::{Program, Symbol},
        types::{
            apply_type_args, type_arrow, type_tyapp, type_tycon, TyCon, TyConInfo, Type, TypeNode,
        },
    },
    constants::{LS_CAP_NAME, LS_CAP_PREFIX, LS_LAM_SUFFIX, LS_OPERAND_PREFIX, LS_UNREAD_NAME},
    misc::{Map, Set},
    optimization::{
        capture_struct::CaptureStruct, move_bodies::move_bodies_into_callers,
    },
};
use std::{collections::VecDeque, sync::Arc};

/// What the rewrite makes of a program, to be put in place of what the program holds.
pub struct Rewritten {
    /// Every symbol of the rewritten program.
    symbols: Map<FullName, Symbol>,
    /// The type constructors the rewritten program adds.
    tycons: Map<TyCon, TyConInfo>,
    /// The rewritten entry, where the program has one.
    entry: Option<Arc<ExprNode>>,
    /// The rewritten value of each export statement, in order.
    exports: Vec<Arc<ExprNode>>,
    /// For each symbol, the lifted lambdas it calls, whose bodies go where it calls them.
    lifted_callees: Map<FullName, Vec<FullName>>,
}

impl Rewritten {
    /// Puts the rewritten program in place of `prg`, then puts the body of each lifted lambda one
    /// symbol calls once where it calls it, on up to `num_threads` threads.
    pub fn apply(self, prg: &mut Program, num_threads: usize) {
        prg.symbols = self.symbols;
        prg.type_env.add_tycons(self.tycons);
        if let Some(entry) = self.entry {
            prg.entry_io_value = Some(entry);
        }
        assert_eq!(
            prg.export_statements.len(),
            self.exports.len(),
            "the rewrite gives other than one value to each export statement"
        );
        for (stmt, value) in prg.export_statements.iter_mut().zip(self.exports) {
            stmt.value_expr = Some(value);
        }
        move_bodies_into_callers(&mut prg.symbols, &self.lifted_callees, num_threads);
    }
}

/// The program `prg` rewritten by its lambda sets.
///
/// A copy of each component is made for each way its copies represent the sets of
/// `Component::body_params`; a component with one such way keeps the names of its symbols. A lambda
/// is lifted to a global function taking its capture struct, once for each class of its elements
/// (`Representation::classes`); the lambda that is the body of its symbol is the symbol's copy
/// instead. Where the lambda's set is represented by its capture struct, the struct is the value;
/// where it is represented by closures, the lifted function applied to the struct is. A type
/// constructor whose own slots hold a set represented by a capture struct is copied for each way
/// its own slots are represented.
pub fn rewrite(
    prg: &Program,
    layout: &SlotLayout,
    analysis: &Analysis,
    spec: &Specialization,
    repr: &Representation,
) -> Rewritten {
    let mut rw = Rewriter::new(prg, layout, analysis, spec, repr);
    for copy in 0..rw.copies.len() {
        rw.make_copy(copy);
    }
    while let Some(c) = rw.lift_queue.pop_front() {
        rw.make_lifted(c);
    }
    let (entry, exports) = rw.rewrite_roots();
    rw.declare_capture_structs();
    Rewritten {
        symbols: rw.symbols,
        tycons: rw.new_tycons,
        entry,
        exports,
        lifted_callees: rw.lifted_callees,
    }
}

/// A copy of the symbols of a component.
struct SymbolCopy {
    /// The component.
    component: usize,
    /// The copy of the component, as an index into `Specialization::instances`, whose sets the
    /// bodies are rewritten with.
    instance: usize,
    /// What the copy appends to the names of the symbols; `None` where the component has one copy.
    suffix: Option<String>,
}

/// The state of a rewrite of one program.
struct Rewriter<'a, 'e> {
    /// The program before the rewrite.
    prg: &'a Program,
    /// The slots of the types of `prg`.
    layout: &'a SlotLayout<'e>,
    /// The lambda sets inferred for `prg`.
    analysis: &'a Analysis,
    /// The copies `analysis` asks for.
    spec: &'a Specialization,
    /// How the values of each set of `spec` are represented.
    repr: &'a Representation,
    /// The copies of the symbols of the components.
    copies: Vec<SymbolCopy>,
    /// The copy each copy of a component is made into, as an index into `copies`.
    copy_of_instance: Vec<usize>,
    /// The copy of each component made for the representations of its body parameters.
    copy_of_body_reprs: Map<(usize, Vec<Repr>), usize>,
    /// The lambda each symbol whose body is a lambda has as its body.
    body_lambda: Map<FullName, LamId>,
    /// The lambda written at each node, by its symbol and the node's position.
    lambda_at: Map<(FullName, usize), LamId>,
    /// The field each punched struct punches out.
    punched_at: Map<TyCon, usize>,
    /// The punched structs of each struct, each with the field it punches out.
    punched_of: Map<TyCon, Vec<(TyCon, usize)>>,
    /// The suffix of the copies of each group of type constructors made for the representations of
    /// its own slots.
    type_copy_suffixes: Map<(usize, Vec<Repr>), String>,
    /// The type constructors made so far.
    new_tycons: Map<TyCon, TyConInfo>,
    /// The capture structs made so far, by their classes.
    capture_structs: Map<u32, CaptureStruct>,
    /// The classes whose capture struct a type made so far names.
    named_capture_structs: Set<u32>,
    /// The classes whose lifted function has been asked for.
    lift_asked: Set<u32>,
    /// The classes whose lifted function is asked for and not yet made.
    lift_queue: VecDeque<u32>,
    /// The symbols made so far.
    symbols: Map<FullName, Symbol>,
    /// For each symbol made, the lifted lambdas it calls.
    lifted_callees: Map<FullName, Vec<FullName>>,
}

/// A walk rewriting the nodes of one expression with the sets of one copy of its component.
struct Walk<'n> {
    /// The symbol whose body the expression is part of; `None` for a root of the program.
    symbol: Option<FullName>,
    /// The copy of the component, as an index into `Specialization::instances`.
    instance: usize,
    /// The nodes of the body or the root, in the order `NodeType` lists them.
    nodes: &'n [NodeType],
    /// The position in `nodes` of the next node the walk meets.
    cursor: usize,
    /// The symbol the rewritten expression becomes the body of; `None` for a root.
    owner: Option<FullName>,
}

impl<'a, 'e> Rewriter<'a, 'e> {
    /// A rewriter that has decided the copy of each component and made nothing yet. Copies of one
    /// component that represent the sets of `Component::body_params` alike share one copy of the
    /// symbols.
    fn new(
        prg: &'a Program,
        layout: &'a SlotLayout<'e>,
        analysis: &'a Analysis,
        spec: &'a Specialization,
        repr: &'a Representation,
    ) -> Self {
        let mut copies: Vec<SymbolCopy> = vec![];
        let mut copy_of_body_reprs: Map<(usize, Vec<Repr>), usize> = Map::default();
        let mut copy_of_instance = vec![];
        for (k, instance) in spec.instances.iter().enumerate() {
            let component = &analysis.components[instance.component];
            let body_reprs: Vec<Repr> = component
                .body_params
                .iter()
                .map(|p| repr.of(instance.args[*p]))
                .collect();
            let copy = *copy_of_body_reprs
                .entry((instance.component, body_reprs))
                .or_insert_with(|| {
                    copies.push(SymbolCopy {
                        component: instance.component,
                        instance: k,
                        suffix: None,
                    });
                    copies.len() - 1
                });
            copy_of_instance.push(copy);
        }
        let mut copy_counts: Map<usize, usize> = Map::default();
        for (component, _) in copy_of_body_reprs.keys() {
            *copy_counts.entry(*component).or_insert(0) += 1;
        }
        for ((component, body_reprs), copy) in &copy_of_body_reprs {
            if copy_counts[component] > 1 {
                copies[*copy].suffix = Some(repr.copy_suffix(body_reprs));
            }
        }

        let mut body_lambda = Map::default();
        let mut lambda_at = Map::default();
        for (lam, lambda) in analysis.lambdas.iter().enumerate() {
            if lambda.is_body() {
                body_lambda.insert(lambda.symbol.clone(), lam as LamId);
            }
            lambda_at.insert((lambda.symbol.clone(), lambda.nodes.start), lam as LamId);
        }

        let mut punched_at = Map::default();
        let mut punched_of: Map<TyCon, Vec<(TyCon, usize)>> = Map::default();
        for (tc, info) in layout.type_env().tycons().iter() {
            let Some(from) = &info.punched_from else {
                continue;
            };
            let punched: Vec<usize> = (0..info.fields.len())
                .filter(|i| info.fields[*i].is_punched)
                .collect();
            assert_eq!(
                punched.len(),
                1,
                "the punched struct `{}` punches out other than one field",
                tc.to_string()
            );
            punched_at.insert(tc.clone(), punched[0]);
            punched_of
                .entry(from.clone())
                .or_default()
                .push((tc.clone(), punched[0]));
        }

        Rewriter {
            prg,
            layout,
            analysis,
            spec,
            repr,
            copies,
            copy_of_instance,
            copy_of_body_reprs,
            body_lambda,
            lambda_at,
            punched_at,
            punched_of,
            type_copy_suffixes: Map::default(),
            new_tycons: Map::default(),
            capture_structs: Map::default(),
            named_capture_structs: Set::default(),
            lift_asked: Set::default(),
            lift_queue: VecDeque::new(),
            symbols: Map::default(),
            lifted_callees: Map::default(),
        }
    }

    /// The set the copy `instance` gives the variable `v` of its component.
    fn set_of(&self, instance: usize, v: SetVar) -> SetId {
        let instance = &self.spec.instances[instance];
        let component = &self.analysis.components[instance.component];
        instance.var_sets[component.var_index[&v]]
    }

    /// The sets the copy `instance` gives the slots of `ty`.
    fn sets_of(&self, instance: usize, ty: &AType) -> Vec<SetId> {
        ty.slots.iter().map(|v| self.set_of(instance, *v)).collect()
    }

    /// The name of the copy `copy` of `symbol`.
    fn copy_name(&self, symbol: &FullName, copy: usize) -> FullName {
        let mut name = symbol.clone();
        if let Some(suffix) = &self.copies[copy].suffix {
            name.name += suffix;
        }
        name
    }

    /// The copy of the component of the class `c`'s lambda that the class stands for, where the
    /// lambda is the body of its symbol: the copy made for the representations the class gives
    /// the body parameters.
    fn body_copy_of_class(&self, c: u32) -> usize {
        let class = &self.repr.classes[c as usize];
        let lambda = &self.analysis.lambdas[class.lam as usize];
        let component = self.analysis.component_of[&lambda.symbol];
        self.copy_of_body_reprs[&(component, class.arg_reprs.clone())]
    }

    /// The name of the global function a value of the class `c` is called through, where the
    /// class's lambda is not the body of its symbol.
    fn lifted_name(&self, c: u32) -> FullName {
        let class = &self.repr.classes[c as usize];
        let lambda = &self.analysis.lambdas[class.lam as usize];
        let mut name = lambda.symbol.clone();
        name.name += &format!("{}{}", LS_LAM_SUFFIX, lambda.index);
        if self.repr.class_count(class.lam) > 1 {
            name.name += "#";
            name.name += &class.hash;
        }
        name
    }

    /// The global function a value of the class `c` is called through: the copy of the symbol
    /// whose body the class's lambda is, or the lifted lambda.
    fn callee_name(&self, c: u32) -> FullName {
        let class = &self.repr.classes[c as usize];
        let lambda = &self.analysis.lambdas[class.lam as usize];
        if lambda.is_body() {
            self.copy_name(&lambda.symbol, self.body_copy_of_class(c))
        } else {
            self.lifted_name(c)
        }
    }

    /// The type constructor of the capture struct of the class `c`.
    fn capture_tycon(&mut self, c: u32) -> Arc<TyCon> {
        self.named_capture_structs.insert(c);
        Arc::new(TyCon::new(CaptureStruct::tycon_name(
            LS_CAP_PREFIX,
            &self.callee_name(c),
        )))
    }

    /// The capture struct of the class `c`, made the first time it is asked for: its fields are the
    /// names the lambda captures, at their types in the copy that stands for the class.
    fn capture_struct(&mut self, c: u32) -> CaptureStruct {
        if let Some(cap) = self.capture_structs.get(&c) {
            return cap.clone();
        }
        let class = &self.repr.classes[c as usize];
        let lambda = &self.analysis.lambdas[class.lam as usize];
        let instance = class.instance;
        let fields: Vec<(FullName, Arc<TypeNode>)> = lambda
            .captures
            .iter()
            .map(|(name, ty)| (name.clone(), self.rewrite_type(instance, ty)))
            .collect();
        let cap = CaptureStruct::new(LS_CAP_PREFIX, &self.callee_name(c), &fields);
        self.new_tycons
            .insert(cap.tycon.as_ref().clone(), cap.tycon_info.clone());
        self.capture_structs.insert(c, cap.clone());
        cap
    }

    /// Declares the capture struct of every class a type made so far names.
    fn declare_capture_structs(&mut self) {
        loop {
            let pending: Vec<u32> = self
                .named_capture_structs
                .iter()
                .filter(|c| !self.capture_structs.contains_key(*c))
                .copied()
                .collect();
            if pending.is_empty() {
                break;
            }
            for c in pending {
                self.capture_struct(c);
            }
        }
    }

    /// Asks for the lifted function of the class `c`.
    fn ask_lift(&mut self, c: u32) {
        if self.lift_asked.insert(c) {
            self.lift_queue.push_back(c);
        }
    }

    /// `ty` at the sets the copy `instance` gives its slots.
    fn rewrite_type(&mut self, instance: usize, ty: &AType) -> Arc<TypeNode> {
        let sets = self.sets_of(instance, ty);
        self.rewrite_type_at(&ty.ty, &sets, false)
    }

    /// The function type `ty` at the sets the copy `instance` gives its slots, kept a function
    /// whatever its own set is represented by: the type of a symbol whose body is a lambda.
    fn rewrite_function_type(&mut self, instance: usize, ty: &AType) -> Arc<TypeNode> {
        let sets = self.sets_of(instance, ty);
        self.rewrite_type_at(&ty.ty, &sets, true)
    }

    /// `ty` at the sets `sets` of its slots. With `keep_function`, a function type stays one.
    fn rewrite_type_at(
        &mut self,
        ty: &Arc<TypeNode>,
        sets: &[SetId],
        keep_function: bool,
    ) -> Arc<TypeNode> {
        let layout = self.layout;
        let root = keep_function.then(|| ty.clone());
        layout.fold_slots(ty, sets, &mut TypeRewrite { rw: self, root })
    }

    /// The type constructor standing for `tc` where its own slots are represented by `own`: `tc`
    /// itself where they are all closures, and a copy otherwise.
    fn tycon_copy(&mut self, tc: &Arc<TyCon>, own: &[Repr]) -> Arc<TyCon> {
        if own.iter().all(|r| *r == Repr::Closure) {
            return tc.clone();
        }
        let layout_tc = self.layout.layout_tycon(tc);
        let group = self.layout.group_of(&layout_tc);
        let suffix = self.type_copy_suffix(group, own);
        let mut name = layout_tc.name.clone();
        name.name += &suffix;
        let mut copy = TyCon::new(name);
        if let Some(i) = self.punched_at.get(tc.as_ref()) {
            copy.into_punched_type_name(*i);
        }
        Arc::new(copy)
    }

    /// The suffix of the copies of the group `group` made for the representations `own` of its own
    /// slots, declaring the copies the first time it is asked for.
    fn type_copy_suffix(&mut self, group: usize, own: &[Repr]) -> String {
        let key = (group, own.to_vec());
        if let Some(suffix) = self.type_copy_suffixes.get(&key) {
            return suffix.clone();
        }
        let suffix = self.repr.copy_suffix(own);
        self.type_copy_suffixes.insert(key, suffix.clone());
        self.declare_group_copy(group, own, &suffix);
        suffix
    }

    /// Declares the copy of each member of the group `group`, and of each struct punched out of
    /// one, whose own slots are represented by `own`, each named with `suffix` appended.
    fn declare_group_copy(&mut self, group: usize, own: &[Repr], suffix: &str) {
        let type_env = self.layout.type_env();
        let members = self.layout.group_members(group).to_vec();
        // The fields of the group read the own slots one after another, so the first own slot of
        // each field is where the previous field stopped, as `SlotLayout::field_offset` gives it.
        let mut next_offset = 0;
        for member in &members {
            // An unwrapped newtype is a group of its own and owns no slot, so no representation
            // of the group's own slots asks for a copy of it.
            assert!(
                !type_env.is_unwrapped_newtype(member),
                "a copy of the unwrapped newtype `{}` is asked for",
                member.to_string()
            );
            let info = type_env.tycon_info(member);
            let mut fields = info.fields.clone();
            for (i, field) in fields.iter_mut().enumerate() {
                let mut cursor = self.layout.field_offset(member, i);
                assert_eq!(
                    cursor,
                    next_offset,
                    "the field {} of `{}` is read out of step with its own slots",
                    i,
                    member.to_string()
                );
                field.ty = self.rewrite_declared(&field.ty, group, own, &mut cursor);
                next_offset = cursor;
            }
            let mut name = member.name.clone();
            name.name += suffix;
            let copy = TyCon::new(name);
            self.new_tycons.insert(
                copy.clone(),
                TyConInfo {
                    fields: fields.clone(),
                    ..info.clone()
                },
            );
            for (punched, i) in self.punched_of.get(member).cloned().unwrap_or_default() {
                let punched_info = type_env.tycon_info(&punched);
                let mut punched_fields = fields.clone();
                for (field, punched_field) in punched_fields.iter_mut().zip(&punched_info.fields) {
                    field.is_punched = punched_field.is_punched;
                }
                let mut punched_copy = copy.clone();
                punched_copy.into_punched_type_name(i);
                self.new_tycons.insert(
                    punched_copy,
                    TyConInfo {
                        fields: punched_fields,
                        punched_from: Some(copy.clone()),
                        ..punched_info.clone()
                    },
                );
            }
        }
        assert_eq!(
            next_offset,
            own.len(),
            "the fields of a group read other than its own slots"
        );
    }

    /// The declared field type `ty` of a member of the group `group`, whose own slots are
    /// represented by `own`, the next of them at `cursor`. The type variables of the declaration
    /// stay; the slots are read in the order `SlotLayout::walk_declared` reads them.
    fn rewrite_declared(
        &mut self,
        ty: &Arc<TypeNode>,
        group: usize,
        own: &[Repr],
        cursor: &mut usize,
    ) -> Arc<TypeNode> {
        let spine = ty.flatten_type_application();
        let args = &spine[1..];
        match &spine[0].ty {
            Type::TyVar(_) => {
                if args.is_empty() {
                    return ty.clone();
                }
                args.iter().fold(spine[0].clone(), |applied, a| {
                    type_tyapp(applied, self.rewrite_declared(a, group, own, cursor))
                })
            }
            Type::TyCon(tc) => {
                if !self.layout.is_applied_to_all(tc, args.len()) {
                    return ty.clone();
                }
                let range = self.layout.declared_own_range(tc, group, own.len(), cursor);
                let tc_own = own[range].to_vec();
                let new_args: Vec<Arc<TypeNode>> = args
                    .iter()
                    .map(|a| self.rewrite_declared(a, group, own, cursor))
                    .collect();
                if tc.is_arrow() {
                    return match tc_own[0] {
                        Repr::Single(c) => type_tycon(&self.capture_tycon(c)),
                        Repr::Closure => type_arrow(new_args[0].clone(), new_args[1].clone()),
                    };
                }
                let head = self.tycon_copy(tc, &tc_own);
                apply_type_args(&head, &new_args)
            }
            _ => panic!(
                "the declared type `{}` has an unexpected head",
                ty.to_string()
            ),
        }
    }

    /// Makes the symbols of the copy `copy`.
    fn make_copy(&mut self, copy: usize) {
        let component_index = self.copies[copy].component;
        let instance = self.copies[copy].instance;
        let component = &self.analysis.components[component_index];
        for (pos, symbol) in component.symbols.iter().enumerate() {
            let original = &self.prg.symbols[symbol];
            let name = self.copy_name(symbol, copy);
            let mut walk = Walk {
                symbol: Some(symbol.clone()),
                instance,
                nodes: &component.node_types[pos],
                cursor: 0,
                owner: Some(name.clone()),
            };
            let expr = self.walk_expr(&mut walk, original.expr.as_ref().unwrap(), true);
            assert_eq!(walk.cursor, walk.nodes.len());
            let ty = if self.body_lambda.contains_key(symbol) {
                self.rewrite_function_type(instance, &component.types[pos])
            } else {
                self.rewrite_type(instance, &component.types[pos])
            };
            self.insert_symbol(Symbol {
                name: name.clone(),
                generic_name: original.generic_name.clone(),
                ty,
                expr: Some(expr),
                inline_into_callers: false,
            });
        }
    }

    /// Makes the lifted function of the class `c`: it takes the capture struct, takes it apart into
    /// the names the lambda captures, and is then the lambda, rewritten with the sets of the copy
    /// standing for the class.
    fn make_lifted(&mut self, c: u32) {
        let class = &self.repr.classes[c as usize];
        let lambda = &self.analysis.lambdas[class.lam as usize];
        let instance = class.instance;
        let component = &self.analysis.components[self.analysis.component_of[&lambda.symbol]];
        let pos = component
            .symbols
            .iter()
            .position(|s| *s == lambda.symbol)
            .unwrap();
        let nodes = &component.node_types[pos];
        let name = self.lifted_name(c);
        let mut walk = Walk {
            symbol: Some(lambda.symbol.clone()),
            instance,
            nodes,
            cursor: lambda.nodes.start + 1,
            owner: Some(name.clone()),
        };
        let (params, body) = lambda.expr.destructure_lam();
        let body = self.walk_expr(&mut walk, &body, false);
        assert_eq!(walk.cursor, lambda.nodes.end);
        let param_ty = self.rewrite_type(
            instance,
            &self.layout.type_arg(&nodes[lambda.nodes.start].ty, 0),
        );
        let lam = expr_abs_typed(params[0].clone(), param_ty, body)
            .set_source(lambda.expr.source.clone());
        let func = self
            .capture_struct(c)
            .lifted_function(&FullName::local(LS_CAP_NAME), lam);
        self.insert_symbol(Symbol {
            name: name.clone(),
            generic_name: name,
            ty: func.type_.clone().unwrap(),
            expr: Some(func),
            inline_into_callers: false,
        });
    }

    /// Adds `symbol` to the symbols made.
    fn insert_symbol(&mut self, symbol: Symbol) {
        let name = symbol.name.clone();
        assert!(
            self.symbols.insert(name.clone(), symbol).is_none(),
            "two symbols of the rewritten program are named `{}`",
            name.to_string()
        );
    }

    /// The entry and the values of the export statements, rewritten with the one copy of the
    /// component of the entry.
    fn rewrite_roots(&mut self) -> (Option<Arc<ExprNode>>, Vec<Arc<ExprNode>>) {
        let component = &self.analysis.components[self.analysis.entry_component()];
        assert_eq!(
            self.spec.instances[0].component,
            self.analysis.entry_component(),
            "the first copy is not that of the entry"
        );
        let prg = self.prg;
        let roots = prg.root_value_exprs();
        let mut rewritten: Vec<Arc<ExprNode>> = roots
            .iter()
            .enumerate()
            .map(|(i, root)| {
                let mut walk = Walk {
                    symbol: None,
                    instance: 0,
                    nodes: &component.node_types[i],
                    cursor: 0,
                    owner: None,
                };
                let expr = self.walk_expr(&mut walk, root, false);
                assert_eq!(walk.cursor, walk.nodes.len());
                expr
            })
            .collect();
        let entry = prg.entry_io_value.as_ref().map(|_| rewritten.remove(0));
        (entry, rewritten)
    }

    /// The expression `e` rewritten by `walk`, whose next node is that of `e`. With `as_body`, `e` is
    /// the body of a symbol, which stays a lambda where it is one.
    fn walk_expr(&mut self, walk: &mut Walk, e: &Arc<ExprNode>, as_body: bool) -> Arc<ExprNode> {
        let index = walk.cursor;
        walk.cursor += 1;
        let node = &walk.nodes[index];
        let ty = if as_body && e.is_lam() {
            self.rewrite_function_type(walk.instance, &node.ty)
        } else {
            self.rewrite_type(walk.instance, &node.ty)
        };
        match &*e.expr {
            Expr::Var(v) => {
                if v.name.is_local() {
                    return e.set_type(ty);
                }
                self.global_value(walk, &v.name, &node.names[0])
                    .set_source(e.source.clone())
            }
            Expr::Builtin(b) => {
                let mut builtin = b.as_ref().clone();
                let mut bindings: Vec<(FullName, Arc<ExprNode>)> = vec![];
                let operands = builtin.op.free_vars_mut();
                for (i, (operand, name_ref)) in operands.into_iter().zip(&node.names).enumerate() {
                    if let NameRef::Local = name_ref {
                        continue;
                    }
                    let value = self.global_value(walk, operand, name_ref);
                    if value.is_var() {
                        *operand = value.get_var().name.clone();
                    } else {
                        let local = FullName::local(&format!("{}{}", LS_OPERAND_PREFIX, i));
                        *operand = local.clone();
                        bindings.push((local, value));
                    }
                }
                let mut expr = e.set_builtin(builtin).set_type(ty);
                for (local, value) in bindings.into_iter().rev() {
                    let pat = PatternNode::make_var(var_local(&local.name), None)
                        .set_type(value.type_.clone().unwrap());
                    expr = expr_let_typed(pat, value, expr);
                }
                expr
            }
            Expr::App(func, args) => {
                let func_node = &walk.nodes[walk.cursor];
                let own = self
                    .repr
                    .of(self.set_of(walk.instance, func_node.ty.slots[0]));
                let func = self.walk_expr(walk, func, false);
                let arg = self.walk_expr(walk, &args[0], false);
                match own {
                    Repr::Single(c) => self.call_class(walk, c, e, func, arg, ty),
                    Repr::Closure => e.set_app_func(func).set_app_args(vec![arg]).set_type(ty),
                }
            }
            Expr::Lam(_, body) => {
                let symbol = walk.symbol.as_ref().expect("a root holds no lambda");
                let lam = self.lambda_at[&(symbol.clone(), index)];
                let lambda = &self.analysis.lambdas[lam as usize];
                assert_eq!(lambda.is_body(), as_body);
                if !as_body {
                    let own = self.repr.of(self.set_of(walk.instance, node.ty.slots[0]));
                    let value = match own {
                        Repr::Single(c) => {
                            assert_eq!(self.repr.classes[c as usize].lam, lam);
                            self.ask_lift(c);
                            Some(self.capture_struct_value(c))
                        }
                        Repr::Closure if lambda.is_lifted_from_closures() => {
                            let instance = &self.spec.instances[walk.instance];
                            let args: Vec<SetId> =
                                lambda.params.iter().map(|p| instance.args[*p]).collect();
                            let c = self.repr.class_of(lam, &args);
                            self.ask_lift(c);
                            Some(self.closure_value(c, ty.clone()))
                        }
                        Repr::Closure => None,
                    };
                    if let Some(value) = value {
                        walk.cursor = lambda.nodes.end;
                        return value.set_source(e.source.clone());
                    }
                }
                let body = self.walk_expr(walk, body, false);
                e.set_lam_body(body).set_type(ty)
            }
            Expr::Let(_, bound, val) => {
                let bound = self.walk_expr(walk, bound, false);
                let pat = self.walk_pattern(walk, &e.get_let_pat());
                let val = self.walk_expr(walk, val, false);
                e.set_let_bound(bound)
                    .set_let_pat(pat)
                    .set_let_value(val)
                    .set_type(ty)
            }
            Expr::If(cond, then_expr, else_expr) => {
                let cond = self.walk_expr(walk, cond, false);
                let then_expr = self.walk_expr(walk, then_expr, false);
                let else_expr = self.walk_expr(walk, else_expr, false);
                e.set_if_cond(cond)
                    .set_if_then(then_expr)
                    .set_if_else(else_expr)
                    .set_type(ty)
            }
            Expr::Match(cond, arms) => {
                let cond = self.walk_expr(walk, cond, false);
                let arms = arms
                    .iter()
                    .map(|(pat, val)| {
                        let pat = self.walk_pattern(walk, pat);
                        let val = self.walk_expr(walk, val, false);
                        (pat, val)
                    })
                    .collect();
                e.set_match_cond(cond).set_match_pat_vals(arms).set_type(ty)
            }
            Expr::TyAnno(expr, _) => {
                let expr = self.walk_expr(walk, expr, false);
                e.set_tyanno_expr(expr)
                    .set_tyanno_ty(ty.clone())
                    .set_type(ty)
            }
            Expr::ArrayLit(elems) => {
                let elems = elems
                    .iter()
                    .map(|elem| self.walk_expr(walk, elem, false))
                    .collect();
                e.set_array_lit_elems(elems).set_type(ty)
            }
            Expr::MakeStruct(_, fields) => {
                let fields = fields
                    .iter()
                    .map(|(name, src, value)| {
                        (
                            name.clone(),
                            src.clone(),
                            self.walk_expr(walk, value, false),
                        )
                    })
                    .collect();
                e.set_make_struct_fields(fields)
                    .set_make_struct_tycon(ty.head_tycon())
                    .set_type(ty)
            }
            Expr::FFICall(_, _, _, _, args, _) => {
                let args = args
                    .iter()
                    .map(|arg| self.walk_expr(walk, arg, false))
                    .collect();
                e.set_ffi_call_args(args).set_type(ty)
            }
            Expr::Eval(side, main) => {
                let side = self.walk_expr(walk, side, false);
                let main = self.walk_expr(walk, main, false);
                e.set_eval_side(side).set_eval_main(main).set_type(ty)
            }
        }
    }

    /// The pattern `pat` rewritten by `walk`, whose next node is that of `pat`.
    fn walk_pattern(&mut self, walk: &mut Walk, pat: &Arc<PatternNode>) -> Arc<PatternNode> {
        let index = walk.cursor;
        walk.cursor += 1;
        let ty = self.rewrite_type(walk.instance, &walk.nodes[index].ty);
        let pat = match &pat.pattern {
            Pattern::Var(_, anno) => match anno {
                Some(_) => pat.set_var_tyanno(Some(ty.clone())),
                None => pat.clone(),
            },
            Pattern::Struct(_, fields, _) => {
                let fields = fields
                    .iter()
                    .map(|(name, src, sub)| {
                        (name.clone(), src.clone(), self.walk_pattern(walk, sub))
                    })
                    .collect();
                pat.set_struct_field_to_pat(fields)
                    .set_struct_tycon(ty.head_tycon())
            }
            Pattern::Union(variant, variant_src, sub) => {
                let sub = self.walk_pattern(walk, sub);
                let union = ty.head_tycon();
                let mut node = pat.as_ref().clone();
                node.pattern = Pattern::Union(
                    FullName::new(&union.name.to_namespace(), &variant.name),
                    variant_src.clone(),
                    sub,
                );
                Arc::new(node)
            }
        };
        pat.set_type(ty)
    }

    /// The value a reference to the global `name` stands for in the walk's copy: the copy of the
    /// symbol the reference names, or, where the symbol's body is a lambda whose set at the
    /// reference is represented by its capture struct, that struct.
    fn global_value(&mut self, walk: &Walk, name: &FullName, name_ref: &NameRef) -> Arc<ExprNode> {
        let NameRef::Global { site, ty } = name_ref else {
            panic!(
                "the global `{}` is referred to as a local",
                name.to_string()
            );
        };
        let target = match site {
            None => walk.instance,
            Some(site) => {
                let args: Vec<SetId> = site
                    .args
                    .iter()
                    .map(|v| self.set_of(walk.instance, *v))
                    .collect();
                self.spec.instance_of[&(site.component, args)]
            }
        };
        let copy = self.copy_of_instance[target];
        let sets = self.sets_of(walk.instance, ty);
        if let Some(lam) = self.body_lambda.get(name) {
            if let Repr::Single(c) = self.repr.of(sets[0]) {
                assert_eq!(self.repr.classes[c as usize].lam, *lam);
                assert_eq!(self.body_copy_of_class(c), copy);
                return self.capture_struct_value(c);
            }
        }
        let ty = self.rewrite_type_at(&ty.ty, &sets, false);
        expr_var(self.copy_name(name, copy), None).set_type(ty)
    }

    /// The construction of a capture struct of the class `c` out of the names its lambda captures.
    fn capture_struct_value(&mut self, c: u32) -> Arc<ExprNode> {
        self.capture_struct(c).struct_expr()
    }

    /// The closure of type `ty` that a lambda of the class `c` builds: the lifted function applied to
    /// the capture struct.
    fn closure_value(&mut self, c: u32, ty: Arc<TypeNode>) -> Arc<ExprNode> {
        let cap = self.capture_struct_value(c);
        let cap_ty = cap.type_.clone().unwrap();
        let callee = expr_var(self.lifted_name(c), None).set_type(type_arrow(cap_ty, ty));
        expr_app_typed(callee, vec![cap])
    }

    /// The application `e` of `func`, a value of the class `c`, to `arg`, of type `ty`: a direct call
    /// of the function the class's values are called through.
    fn call_class(
        &mut self,
        walk: &Walk,
        c: u32,
        e: &Arc<ExprNode>,
        func: Arc<ExprNode>,
        arg: Arc<ExprNode>,
        ty: Arc<TypeNode>,
    ) -> Arc<ExprNode> {
        let lambda = &self.analysis.lambdas[self.repr.classes[c as usize].lam as usize];
        let arg_ty = arg.type_.clone().unwrap();
        let callee_name = self.callee_name(c);
        if lambda.is_body() {
            // The symbol's copy captures nothing, so the call leaves the capture struct unread.
            let callee = expr_var(callee_name, None).set_type(type_arrow(arg_ty, ty.clone()));
            let call = e.set_app_func(callee).set_app_args(vec![arg]).set_type(ty);
            if func.is_var() {
                return call;
            }
            let pat = PatternNode::make_var(var_local(LS_UNREAD_NAME), None)
                .set_type(func.type_.clone().unwrap());
            return expr_let_typed(pat, func, call);
        }
        self.ask_lift(c);
        if let Some(owner) = &walk.owner {
            self.lifted_callees
                .entry(owner.clone())
                .or_default()
                .push(callee_name.clone());
        }
        let cap_ty = func.type_.clone().unwrap();
        let callee = expr_var(callee_name, None)
            .set_type(type_arrow(cap_ty, type_arrow(arg_ty, ty.clone())));
        let partial = expr_app_typed(callee, vec![func]);
        e.set_app_func(partial).set_app_args(vec![arg]).set_type(ty)
    }
}

/// The fold rewriting a type at the sets of its slots (`Rewriter::rewrite_type_at`).
struct TypeRewrite<'w, 'a, 'e> {
    /// The rewrite the types are made in.
    rw: &'w mut Rewriter<'a, 'e>,
    /// The type being rewritten, where it stays a function whatever its own set is represented by.
    root: Option<Arc<TypeNode>>,
}

impl SlotFold<SetId> for TypeRewrite<'_, '_, '_> {
    type Out = Arc<TypeNode>;

    fn short(&mut self, ty: &Arc<TypeNode>) -> Arc<TypeNode> {
        ty.clone()
    }

    fn applied(
        &mut self,
        ty: &Arc<TypeNode>,
        tc: &Arc<TyCon>,
        own: &[SetId],
        args: &[Arc<TypeNode>],
        folded: Vec<Arc<TypeNode>>,
    ) -> Arc<TypeNode> {
        let own: Vec<Repr> = own.iter().map(|s| self.rw.repr.of(*s)).collect();
        let unchanged = folded.iter().zip(args).all(|(n, o)| Arc::ptr_eq(n, o));
        if tc.is_arrow() {
            let keeps_function = self.root.as_ref().is_some_and(|root| Arc::ptr_eq(root, ty));
            if let (Repr::Single(c), false) = (own[0], keeps_function) {
                return type_tycon(&self.rw.capture_tycon(c));
            }
            if unchanged {
                return ty.clone();
            }
            return type_arrow(folded[0].clone(), folded[1].clone());
        }
        let head = self.rw.tycon_copy(tc, &own);
        if Arc::ptr_eq(&head, tc) && unchanged {
            return ty.clone();
        }
        apply_type_args(&head, &folded)
    }
}
