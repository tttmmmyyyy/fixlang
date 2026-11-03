// Inferring the lambda sets of a program, one component of its call graph at a time.

use super::{
    flow::BuiltinFlow,
    slots::{AType, SlotLayout},
    solver::{DynamicKind, DynamicReason, LamId, Member, Owner, SetVar, Solver},
};
use crate::{
    ast::{
        expr::{Expr, ExprNode},
        name::FullName,
        pattern::{Pattern, PatternNode},
        program::Program,
    },
    constants::CAP_NAME,
    graph::Graph,
    misc::{Map, Set},
};
use std::{collections::VecDeque, rc::Rc, sync::Arc};

/// A lambda of the program: an `Expr::Lam` in the body of a symbol.
pub struct Lambda {
    /// The symbol whose body holds the lambda.
    pub symbol: FullName,
    /// The position of the lambda among the lambdas of the symbol's body, counted in preorder.
    pub index: u32,
    /// The parameters of the component the lambda depends on, as indices into
    /// `Component::params`: those its body, its parameter and the values it captures are typed
    /// with. A `Member::Lam` naming the lambda carries one variable for each, in this order.
    pub params: Vec<usize>,
}

impl Lambda {
    /// How the lambda reads in a dump.
    pub fn to_string(&self) -> String {
        format!("{}@lam{}", self.symbol.to_string(), self.index)
    }
}

/// A strongly connected component of the call graph, and what inference found in it.
///
/// The symbols of a component are inferred together and are monomorphic in one another: a call
/// between two of them uses the types the component gives them. What the component exposes is
/// generalized over its parameters.
pub struct Component {
    /// The symbols of the component, in the order of their names. The component of the program's
    /// entry and the values it exports holds none.
    pub symbols: Vec<FullName>,
    /// The annotated type of each symbol, in the order of `symbols`.
    pub types: Vec<AType>,
    /// The parameters: the distinct variables of `types`, in the order they first occur there. A
    /// copy of the component is given by a lambda set for each.
    pub params: Vec<SetVar>,
    /// The references from the component to symbols of other components.
    pub sites: Vec<Site>,
    /// What a reference to a symbol of the component gets.
    pub scheme: Scheme,
}

/// A reference from a component to a symbol of another one, which needs a copy of that one.
pub struct Site {
    /// The component referred to.
    pub component: usize,
    /// For each parameter of that component, the variable of the referring component the
    /// reference gives it.
    pub args: Vec<SetVar>,
}

/// What a reference to a symbol of a component gets: new variables for the classes below, holding
/// the members recorded for them. The first classes are the parameters of the component, in order,
/// and the others are the variables their members name.
pub struct Scheme {
    pub classes: Vec<Vec<SchemeMember>>,
    /// For each symbol of the component, the class of each slot of its type.
    pub symbol_slots: Vec<Vec<u32>>,
}

/// A member of a class of a `Scheme`.
#[derive(Clone, PartialEq)]
pub enum SchemeMember {
    /// `Member::Lam`, with the owner's variables given as classes.
    Lam { lam: LamId, args: Vec<u32> },
    /// `Member::Dynamic`.
    Dynamic(Rc<DynamicReason>),
}

/// The lambda sets of a program.
pub struct Analysis {
    /// The components of the call graph, each after every component it refers to. The last one is
    /// that of the program's entry and the values it exports.
    pub components: Vec<Component>,
    /// The component of each symbol.
    pub component_of: Map<FullName, usize>,
    /// The lambdas of the program.
    pub lambdas: Vec<Lambda>,
    pub solver: Solver,
}

impl Analysis {
    /// The component of the program's entry and the values it exports.
    pub fn root(&self) -> usize {
        self.components.len() - 1
    }
}

/// Infers the lambda sets of `prg`.
pub fn infer(prg: &Program, layout: &SlotLayout) -> Analysis {
    let mut analysis = Analysis {
        components: vec![],
        component_of: Map::default(),
        lambdas: vec![],
        solver: Solver::new(),
    };
    for symbols in components_callees_first(prg) {
        let component = infer_component(prg, layout, &mut analysis, symbols, vec![]);
        analysis.components.push(component);
    }
    let mut roots = vec![];
    if let Some(entry) = &prg.entry_io_value {
        roots.push((entry.clone(), DynamicKind::Entry));
    }
    for export in &prg.export_statements {
        roots.push((export.value_expr.clone().unwrap(), DynamicKind::Export));
    }
    let root = infer_component(prg, layout, &mut analysis, vec![], roots);
    analysis.components.push(root);
    analysis
}

/// The symbols of `prg` split into the strongly connected components of the call graph, each
/// component after every one it refers to, and each in the order of the symbols' names.
fn components_callees_first(prg: &Program) -> Vec<Vec<FullName>> {
    let mut names: Vec<FullName> = prg.symbols.keys().cloned().collect();
    names.sort();
    let index: Map<FullName, usize> = names
        .iter()
        .enumerate()
        .map(|(i, n)| (n.clone(), i))
        .collect();
    let mut graph = Graph::new(names.clone());
    for (i, name) in names.iter().enumerate() {
        let expr = prg.symbols[name].expr.as_ref().unwrap();
        let mut callees: Vec<usize> = expr
            .free_vars()
            .iter()
            .filter_map(|n| index.get(n).copied())
            .collect();
        callees.sort();
        for j in callees {
            graph.connect_idx(i, j);
        }
    }
    // A path from i to j gives i an id no larger than j's, so the callees come at the larger ids.
    let scc_ids = graph.compute_sccs();
    let count = scc_ids.iter().max().map_or(0, |m| m + 1);
    let mut components = vec![vec![]; count];
    for (i, id) in scc_ids.iter().enumerate() {
        components[*id].push(names[i].clone());
    }
    components.reverse();
    components
}

/// Infers the component of `symbols`, given the components it refers to in `analysis`. The
/// component of the entry holds no symbol and gives `roots` instead: the expressions the runtime
/// reads, each with what makes its type Dynamic.
fn infer_component(
    prg: &Program,
    layout: &SlotLayout,
    analysis: &mut Analysis,
    symbols: Vec<FullName>,
    roots: Vec<(Arc<ExprNode>, DynamicKind)>,
) -> Component {
    let id = analysis.components.len();
    for s in &symbols {
        analysis.component_of.insert(s.clone(), id);
    }
    let types: Vec<AType> = symbols
        .iter()
        .map(|s| {
            analysis.solver.set_current_symbol(Some(s.clone()));
            layout.fresh(&prg.symbols[s].ty, &mut analysis.solver)
        })
        .collect();
    let mut inferencer = Inferencer {
        prg,
        layout,
        analysis,
        component: id,
        symbols: &symbols,
        types: &types,
        env: Map::default(),
        frames: vec![],
        lambdas: vec![],
        current_members: vec![],
        sites: vec![],
        lam_counter: 0,
    };
    for (i, s) in symbols.iter().enumerate() {
        inferencer.enter_symbol(Some(s.clone()));
        let body = inferencer.infer(prg.symbols[s].expr.as_ref().unwrap());
        inferencer.analysis.solver.unify(&types[i], &body);
    }
    inferencer.enter_symbol(None);
    for (expr, kind) in &roots {
        let ty = inferencer.infer(expr);
        inferencer.analysis.solver.make_all_dynamic(&ty, kind);
    }
    let Inferencer {
        lambdas,
        current_members,
        sites,
        ..
    } = inferencer;
    generalize(analysis, symbols, types, lambdas, current_members, sites)
}

/// What a lambda under inference records about its term, which decides the parameters it depends on.
struct LamTerm {
    lam: LamId,
    /// The annotated type of the lambda expression.
    ty: AType,
    /// The variables of the nodes of its body that no lambda inside it holds, of its parameter,
    /// and of the references to other components made there.
    vars: Vec<SetVar>,
    /// The lambdas written directly inside its body.
    children: Vec<LamId>,
}

/// The walk inferring the bodies of one component.
struct Inferencer<'a, 'p, 'e> {
    prg: &'p Program,
    layout: &'a SlotLayout<'e>,
    analysis: &'a mut Analysis,
    component: usize,
    symbols: &'a [FullName],
    types: &'a [AType],
    /// The annotated types of the local names in scope, the innermost binding of a name last.
    env: Map<FullName, Vec<AType>>,
    /// The lambdas enclosing the node under inference, the innermost last, as indices into
    /// `lambdas`.
    frames: Vec<usize>,
    /// The lambdas of the component.
    lambdas: Vec<LamTerm>,
    /// The variables a `Member::Lam` with `Owner::Current` was added to.
    current_members: Vec<SetVar>,
    sites: Vec<Site>,
    /// How many lambdas of the current symbol's body have been met.
    lam_counter: u32,
}

impl<'a, 'p, 'e> Inferencer<'a, 'p, 'e> {
    /// Starts inferring the body of `symbol`, or the roots where `None`.
    fn enter_symbol(&mut self, symbol: Option<FullName>) {
        self.analysis.solver.set_current_symbol(symbol);
        self.lam_counter = 0;
    }

    /// Records the variables of `ty`, met inside the innermost lambda, as part of its term.
    fn record(&mut self, ty: &AType) {
        self.record_vars(&ty.slots);
    }

    /// Records `vars`, met inside the innermost lambda, as part of its term.
    fn record_vars(&mut self, vars: &[SetVar]) {
        if let Some(frame) = self.frames.last() {
            self.lambdas[*frame].vars.extend(vars.iter().copied());
        }
    }

    /// The annotated type of `e`, after recording what `e` makes equal and what it puts where.
    fn infer(&mut self, e: &Arc<ExprNode>) -> AType {
        let ty = e.type_.clone().unwrap();
        let result = match &*e.expr {
            Expr::Var(v) => self.reference(&v.name),
            Expr::Builtin(b) => {
                let operands: Vec<AType> =
                    b.op.free_vars()
                        .iter()
                        .map(|name| self.operand(name))
                        .collect();
                let result = self.layout.fresh(&ty, &mut self.analysis.solver);
                let mut flow = BuiltinFlow::new(
                    self.layout,
                    &mut self.analysis.solver,
                    operands,
                    result.clone(),
                    b.op.name(),
                );
                b.op.lambda_set_flow(&mut flow);
                result
            }
            Expr::App(func, args) => {
                assert_eq!(
                    args.len(),
                    1,
                    "an application at this stage takes one argument"
                );
                let func = self.infer(func);
                let arg = self.infer(&args[0]);
                let param = self.layout.type_arg(&func, 0);
                self.analysis.solver.unify(&param, &arg);
                let ret = self.layout.type_arg(&func, 1);
                AType {
                    ty: ty.clone(),
                    slots: ret.slots,
                }
            }
            Expr::Lam(params, body) => {
                assert_eq!(
                    params.len(),
                    1,
                    "a lambda at this stage takes one parameter"
                );
                let lam_ty = self.layout.fresh(&ty, &mut self.analysis.solver);
                let lam = self.new_lambda();
                let own = lam_ty.slots[0];
                self.analysis.solver.add(
                    own,
                    Member::Lam {
                        lam,
                        owner: Owner::Current,
                    },
                );
                self.current_members.push(own);
                if let Some(parent) = self.frames.last() {
                    self.lambdas[*parent].children.push(lam);
                }
                let frame = self.lambdas.len();
                self.lambdas.push(LamTerm {
                    lam,
                    ty: lam_ty.clone(),
                    vars: vec![],
                    children: vec![],
                });
                self.frames.push(frame);
                let param = self.layout.type_arg(&lam_ty, 0);
                self.record(&param);
                self.bind(&params[0].name, param);
                let body = self.infer(body);
                self.unbind(&params[0].name);
                self.frames.pop();
                let ret = self.layout.type_arg(&lam_ty, 1);
                self.analysis.solver.unify(&ret, &body);
                lam_ty
            }
            Expr::Let(pat, bound, val) => {
                let bound = self.infer(bound);
                let names = self.bind_pattern(pat, &bound);
                let val = self.infer(val);
                for name in names.iter().rev() {
                    self.unbind(name);
                }
                val
            }
            Expr::If(cond, then_expr, else_expr) => {
                self.infer(cond);
                let then_ty = self.infer(then_expr);
                let else_ty = self.infer(else_expr);
                self.analysis.solver.unify(&then_ty, &else_ty);
                then_ty
            }
            Expr::Match(cond, arms) => {
                let cond = self.infer(cond);
                let result = self.layout.fresh(&ty, &mut self.analysis.solver);
                for (pat, val) in arms {
                    let names = self.bind_pattern(pat, &cond);
                    let val = self.infer(val);
                    self.analysis.solver.unify(&result, &val);
                    for name in names.iter().rev() {
                        self.unbind(name);
                    }
                }
                result
            }
            Expr::TyAnno(expr, _) => self.infer(expr),
            Expr::ArrayLit(elems) => {
                let result = self.layout.fresh(&ty, &mut self.analysis.solver);
                let elem_ty = self.layout.type_arg(&result, 0);
                for elem in elems {
                    let elem = self.infer(elem);
                    self.analysis.solver.unify(&elem_ty, &elem);
                }
                result
            }
            Expr::MakeStruct(_, fields) => {
                let result = self.layout.fresh(&ty, &mut self.analysis.solver);
                for (name, _, field_expr) in fields {
                    let index = ty.field_index(&self.prg.type_env, name).unwrap();
                    let field = self.layout.field(&result, index, &mut self.analysis.solver);
                    let value = self.infer(field_expr);
                    self.analysis.solver.unify(&field, &value);
                }
                result
            }
            Expr::FFICall(_, _, _, _, args, _) => {
                for arg in args {
                    let arg = self.infer(arg);
                    assert!(
                        arg.slots.is_empty(),
                        "the C function argument of type `{}` holds a function",
                        arg.ty.to_string()
                    );
                }
                let result = self.layout.fresh(&ty, &mut self.analysis.solver);
                assert!(
                    result.slots.is_empty(),
                    "the C function result of type `{}` holds a function",
                    ty.to_string()
                );
                result
            }
            Expr::Eval(side, main) => {
                self.infer(side);
                self.infer(main)
            }
        };
        self.record(&result);
        result
    }

    /// A new lambda of the current symbol.
    fn new_lambda(&mut self) -> LamId {
        let lam = self.analysis.lambdas.len() as LamId;
        self.analysis.lambdas.push(Lambda {
            symbol: self
                .analysis
                .solver
                .current_symbol()
                .expect("the roots of the program hold no lambda")
                .clone(),
            index: self.lam_counter,
            params: vec![],
        });
        self.lam_counter += 1;
        lam
    }

    /// The annotated type of the name `name` read by a builtin operation. `CAP_NAME` is the capture
    /// of the innermost lambda, which only a closure object holds, so that lambda's set is Dynamic.
    fn operand(&mut self, name: &FullName) -> AType {
        if name.is_local() && name.name == CAP_NAME {
            let frame = *self
                .frames
                .last()
                .expect("a builtin operation reads the capture of no lambda");
            let ty = self.lambdas[frame].ty.clone();
            let own = ty.slots[0];
            self.analysis
                .solver
                .make_dynamic(own, DynamicKind::ReadsOwnCapture);
            return ty;
        }
        self.reference(name)
    }

    /// The annotated type of a reference to the name `name`.
    fn reference(&mut self, name: &FullName) -> AType {
        if name.is_local() {
            return self
                .env
                .get(name)
                .and_then(|tys| tys.last())
                .unwrap_or_else(|| panic!("the local name `{}` is unbound", name.to_string()))
                .clone();
        }
        let component = *self
            .analysis
            .component_of
            .get(name)
            .unwrap_or_else(|| panic!("the global name `{}` is no symbol", name.to_string()));
        if component == self.component {
            let i = self.symbols.iter().position(|s| s == name).unwrap();
            return self.types[i].clone();
        }
        let callee = &self.analysis.components[component];
        let position = callee.symbols.iter().position(|s| s == name).unwrap();
        let vars = instantiate(&callee.scheme, &mut self.analysis.solver);
        let callee = &self.analysis.components[component];
        let slots: Vec<SetVar> = callee.scheme.symbol_slots[position]
            .iter()
            .map(|c| vars[*c as usize])
            .collect();
        let args = vars[..callee.params.len()].to_vec();
        self.record_vars(&args);
        self.sites.push(Site { component, args });
        AType {
            ty: self.prg.symbols[name].ty.clone(),
            slots,
        }
    }

    /// Brings the local name `name` into scope, at the annotated type `ty`.
    fn bind(&mut self, name: &FullName, ty: AType) {
        self.env.entry(name.clone()).or_default().push(ty);
    }

    /// Takes the innermost binding of the local name `name` out of scope.
    fn unbind(&mut self, name: &FullName) {
        self.env.get_mut(name).unwrap().pop();
    }

    /// Brings the names `pat` binds into scope, where `pat` takes apart a value of the annotated
    /// type `ty`. Returns the names, in the order they were bound.
    fn bind_pattern(&mut self, pat: &Arc<PatternNode>, ty: &AType) -> Vec<FullName> {
        let mut names = vec![];
        self.bind_pattern_into(pat, ty, &mut names);
        names
    }

    /// `bind_pattern`, appending the names to `names`.
    fn bind_pattern_into(&mut self, pat: &Arc<PatternNode>, ty: &AType, names: &mut Vec<FullName>) {
        match &pat.pattern {
            Pattern::Var(v, _) => {
                self.bind(&v.name, ty.clone());
                names.push(v.name.clone());
            }
            Pattern::Struct(_, fields, _) => {
                for (name, _, sub) in fields {
                    let index = ty.ty.field_index(&self.prg.type_env, name).unwrap();
                    let field = self.layout.field(ty, index, &mut self.analysis.solver);
                    self.record(&field);
                    self.bind_pattern_into(sub, &field, names);
                }
            }
            Pattern::Union(variant, _, sub) => {
                let index = ty
                    .ty
                    .field_index(&self.prg.type_env, &variant.name)
                    .unwrap();
                let payload = self.layout.field(ty, index, &mut self.analysis.solver);
                self.record(&payload);
                self.bind_pattern_into(sub, &payload, names);
            }
        }
    }
}

/// New variables for the classes of `scheme`, holding the members recorded for them.
fn instantiate(scheme: &Scheme, solver: &mut Solver) -> Vec<SetVar> {
    let vars: Vec<SetVar> = scheme.classes.iter().map(|_| solver.fresh()).collect();
    for (class, members) in scheme.classes.iter().enumerate() {
        for member in members {
            let member = match member {
                SchemeMember::Lam { lam, args } => Member::Lam {
                    lam: *lam,
                    owner: Owner::Instance(args.iter().map(|a| vars[*a as usize]).collect()),
                },
                SchemeMember::Dynamic(reason) => Member::Dynamic(reason.clone()),
            };
            solver.add(vars[class], member);
        }
    }
    vars
}

/// The component inferred from the annotated `types` of its `symbols`, its `lambdas`, the
/// variables `current_members` holding a lambda of it, and its references `sites`: the parameters
/// are found, each lambda is given the ones it depends on, and the scheme is built.
fn generalize(
    analysis: &mut Analysis,
    symbols: Vec<FullName>,
    types: Vec<AType>,
    lambdas: Vec<LamTerm>,
    current_members: Vec<SetVar>,
    sites: Vec<Site>,
) -> Component {
    let solver = &mut analysis.solver;
    let mut params: Vec<SetVar> = vec![];
    let mut param_index: Map<SetVar, usize> = Map::default();
    for ty in &types {
        for v in &ty.slots {
            let root = solver.find(*v);
            if !param_index.contains_key(&root) {
                param_index.insert(root, params.len());
                params.push(root);
            }
        }
    }

    // The parameters each lambda depends on.
    let term_index: Map<LamId, usize> = lambdas
        .iter()
        .enumerate()
        .map(|(i, t)| (t.lam, i))
        .collect();
    for term in &lambdas {
        let depends_on = lambda_params(solver, &lambdas, &term_index, term.lam, &param_index);
        analysis.lambdas[term.lam as usize].params = depends_on;
    }
    for v in current_members {
        let members = std::mem::take(solver.members_mut(v));
        let members = members
            .into_iter()
            .map(|m| match m {
                Member::Lam {
                    lam,
                    owner: Owner::Current,
                } => Member::Lam {
                    lam,
                    owner: Owner::Instance(
                        analysis.lambdas[lam as usize]
                            .params
                            .iter()
                            .map(|p| params[*p])
                            .collect(),
                    ),
                },
                m => m,
            })
            .collect();
        *solver.members_mut(v) = members;
    }

    // The scheme: the parameters, then every variable their members name.
    let mut class_of: Map<SetVar, u32> = Map::default();
    let mut queue: VecDeque<SetVar> = VecDeque::new();
    for (i, p) in params.iter().enumerate() {
        class_of.insert(*p, i as u32);
        queue.push_back(*p);
    }
    let mut classes: Vec<Vec<SchemeMember>> = vec![vec![]; params.len()];
    while let Some(v) = queue.pop_front() {
        let members = solver.members(v).to_vec();
        let mut scheme_members: Vec<SchemeMember> = vec![];
        for member in members {
            let scheme_member = match member {
                Member::Lam {
                    lam,
                    owner: Owner::Instance(args),
                } => {
                    let args = args
                        .iter()
                        .map(|a| {
                            let root = solver.find(*a);
                            *class_of.entry(root).or_insert_with(|| {
                                queue.push_back(root);
                                classes.push(vec![]);
                                (classes.len() - 1) as u32
                            })
                        })
                        .collect();
                    SchemeMember::Lam { lam, args }
                }
                Member::Lam {
                    owner: Owner::Current,
                    ..
                } => unreachable!("every lambda of the component has its owner given by now"),
                Member::Dynamic(reason) => SchemeMember::Dynamic(reason),
            };
            if !scheme_members.contains(&scheme_member) {
                scheme_members.push(scheme_member);
            }
        }
        classes[class_of[&v] as usize] = scheme_members;
    }
    let symbol_slots = types
        .iter()
        .map(|ty| {
            ty.slots
                .iter()
                .map(|v| class_of[&solver.find(*v)])
                .collect()
        })
        .collect();

    let mut site_keys: Set<(usize, Vec<SetVar>)> = Set::default();
    let mut deduped_sites = vec![];
    for site in sites {
        let args: Vec<SetVar> = site.args.iter().map(|a| solver.find(*a)).collect();
        if site_keys.insert((site.component, args.clone())) {
            deduped_sites.push(Site {
                component: site.component,
                args,
            });
        }
    }

    Component {
        symbols,
        types,
        params,
        sites: deduped_sites,
        scheme: Scheme {
            classes,
            symbol_slots,
        },
    }
}

/// The parameters of the component the lambda `lam` depends on, in order: the ones its term
/// reaches. The term is that of the lambda and of the lambdas inside it; it reaches a variable it
/// holds, and whatever the members of a variable it reaches name. A parameter's members are not
/// followed: a copy of the component gives the parameter its value.
fn lambda_params(
    solver: &mut Solver,
    terms: &[LamTerm],
    term_index: &Map<LamId, usize>,
    lam: LamId,
    param_index: &Map<SetVar, usize>,
) -> Vec<usize> {
    let mut found: Vec<usize> = vec![];
    let mut seen_vars: Set<SetVar> = Set::default();
    let mut seen_lams: Set<LamId> = Set::default();
    let mut lam_stack: Vec<LamId> = vec![lam];
    let mut var_stack: Vec<SetVar> = vec![];
    seen_lams.insert(lam);
    loop {
        if let Some(v) = var_stack.pop() {
            let root = solver.find(v);
            if !seen_vars.insert(root) {
                continue;
            }
            if let Some(p) = param_index.get(&root) {
                found.push(*p);
                continue;
            }
            for member in solver.members(root).to_vec() {
                match member {
                    Member::Lam {
                        owner: Owner::Instance(args),
                        ..
                    } => var_stack.extend(args),
                    Member::Lam {
                        lam: other,
                        owner: Owner::Current,
                    } => {
                        if seen_lams.insert(other) {
                            lam_stack.push(other);
                        }
                    }
                    Member::Dynamic(_) => {}
                }
            }
            continue;
        }
        let Some(l) = lam_stack.pop() else {
            break;
        };
        let term = &terms[term_index[&l]];
        var_stack.extend(term.vars.iter().copied());
        for child in &term.children {
            if seen_lams.insert(*child) {
                lam_stack.push(*child);
            }
        }
    }
    found.sort();
    found
}
