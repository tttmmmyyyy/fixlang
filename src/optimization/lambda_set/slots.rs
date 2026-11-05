// The places in a type where a lambda set is attached, and the annotated types that carry one set
// variable at each.

use super::solver::{DynamicKind, SetVar, Solver};
use crate::{
    ast::{
        name::Name,
        type_env::TypeEnv,
        types::{type_tyapp, TyCon, TyConVariant, Type, TypeNode},
    },
    fixstd::builtin::DESTRUCTOR_TYPE,
    graph::Graph,
    misc::Map,
};
use std::{cell::RefCell, sync::Arc};

/// A Fix type with a lambda set variable at each of its slots.
///
/// The slots of a type are read off it front to back: at each type constructor applied to all its
/// arguments, the constructor's own slots, then the slots of each argument in order. An arrow owns
/// one slot. A struct, a union and an array own the slots `SlotLayout` assigns to the declaration.
/// A type constructor applied to fewer arguments than it takes owns none, and neither do the
/// arguments it is applied to.
#[derive(Clone)]
pub struct AType {
    /// The Fix type.
    pub ty: Arc<TypeNode>,
    /// The variable at each slot of `ty`, in the order the slots are read off it.
    pub slots: Vec<SetVar>,
}

/// The own slots of the type constructors of one group: a type constructor together with every
/// one its fields reach and that reaches it back. Every member of a group has the same own slots,
/// and an occurrence of a member in a member's field stands for those same slots. That is what
/// keeps the slots of a recursive type finite.
struct GroupSlots {
    /// One entry per own slot of the group: the type constructor whose runtime calls the functions
    /// at that slot, if any. A value at such a slot is always a closure built at run time.
    own_slots: Vec<Option<TyCon>>,
    /// For each member and each of its fields, the own slot the field's first own slot is.
    field_offsets: Map<TyCon, Vec<usize>>,
}

/// What an occurrence of a type variable stands for while a declared type is read at an instance.
#[derive(Clone)]
enum Binding {
    /// A type whose slots are the given variables.
    Linked(Arc<TypeNode>, Vec<SetVar>),
    /// A type none of whose slots are tracked: each read of one gets new variables, made Dynamic.
    Untracked(Arc<TypeNode>),
}

impl Binding {
    /// The type this binding stands for.
    fn ty(&self) -> &Arc<TypeNode> {
        match self {
            Binding::Linked(ty, _) | Binding::Untracked(ty) => ty,
        }
    }
}

/// How a walk over a declared type gets the own slots of a type constructor occurring in it.
enum OwnSource<'a> {
    /// The declaration being read is the field of a member of `group`, at an instance whose own
    /// slots are `own`. An occurrence of a member of the group stands for `own` as a whole; any
    /// other one takes the next own slots from `cursor` on.
    Declared {
        group: usize,
        own: &'a [SetVar],
        cursor: usize,
    },
    /// The declaration is that of a newtype expanded at an argument a type constructor was given
    /// without its arguments. Its own slots are new variables, made Dynamic as places inside a
    /// type argument of the newtype.
    Untracked(TyCon),
}

/// The slots of the types of one program, and the annotated types built over them.
pub struct SlotLayout<'e> {
    type_env: &'e TypeEnv,
    /// The group of each type constructor, by its layout constructor (`layout_tycon`).
    group_of: Map<TyCon, usize>,
    /// The members of each group, in the order of their names.
    group_members: Vec<Vec<TyCon>>,
    /// The own slots of each group, worked out the first time they are asked for.
    group_slots: RefCell<Vec<Option<Arc<GroupSlots>>>>,
    /// The number of slots of each type, worked out the first time it is asked for.
    slot_counts: RefCell<Map<Arc<TypeNode>, usize>>,
}

impl<'e> SlotLayout<'e> {
    /// The slots of the types `type_env` declares.
    pub fn new(type_env: &'e TypeEnv) -> Self {
        let mut layout = SlotLayout {
            type_env,
            group_of: Map::default(),
            group_members: vec![],
            group_slots: RefCell::new(vec![]),
            slot_counts: RefCell::new(Map::default()),
        };
        layout.find_groups();
        layout
    }

    /// The type constructor whose declaration decides the slots of `tc`: the struct a punched struct
    /// is punched from, and `tc` itself otherwise. A punched struct is laid out as the struct, and a
    /// value moves between the two with its lambda sets unchanged.
    pub(super) fn layout_tycon(&self, tc: &TyCon) -> TyCon {
        let mut tc = tc.clone();
        while let Some(from) = &self.type_env.tycon_info(&tc).punched_from {
            tc = from.clone();
        }
        tc
    }

    /// The number of arguments `tc` takes.
    pub(super) fn arity(&self, tc: &TyCon) -> usize {
        self.type_env.tycon_info(tc).tyvars.len()
    }

    /// The type environment whose types the slots are of.
    pub(super) fn type_env(&self) -> &'e TypeEnv {
        self.type_env
    }

    /// The group of the type constructor `tc`, given by its layout constructor (`layout_tycon`).
    pub(super) fn group_of(&self, tc: &TyCon) -> usize {
        self.group_of[&self.layout_tycon(tc)]
    }

    /// The members of the group `group`, in the order of their names.
    pub(super) fn group_members(&self, group: usize) -> &[TyCon] {
        &self.group_members[group]
    }

    /// The own slot of its group that the first own slot of the field `field_idx` of `member` is.
    pub(super) fn field_offset(&self, member: &TyCon, field_idx: usize) -> usize {
        let group = self.group_of(member);
        self.group_slots(group).field_offsets[member][field_idx]
    }

    /// Splits the type constructors into groups: the strongly connected components of the graph
    /// with an edge from each one to every struct, union or array its fields own slots of.
    fn find_groups(&mut self) {
        let mut tycons: Vec<TyCon> = self
            .type_env
            .tycons()
            .keys()
            .filter(|tc| self.type_env.tycon_info(tc).punched_from.is_none())
            .cloned()
            .collect();
        tycons.sort_by(|a, b| a.name.cmp(&b.name));
        let index: Map<TyCon, usize> = tycons
            .iter()
            .enumerate()
            .map(|(i, tc)| (tc.clone(), i))
            .collect();
        let mut graph = Graph::new(tycons.clone());
        for (i, tc) in tycons.iter().enumerate() {
            if self.type_env.is_unwrapped_newtype(tc) {
                continue;
            }
            let mut reached = vec![];
            for field in &self.type_env.tycon_info(tc).fields {
                self.collect_owning_tycons(&field.ty, &mut reached);
            }
            for to in reached {
                graph.connect_idx(i, index[&to]);
            }
        }
        for (group, component) in graph
            .components_dependencies_first()
            .into_iter()
            .enumerate()
        {
            let members: Vec<TyCon> = component.into_iter().map(|i| tycons[i].clone()).collect();
            for tc in &members {
                self.group_of.insert(tc.clone(), group);
            }
            self.group_members.push(members);
        }
        *self.group_slots.borrow_mut() = vec![None; self.group_members.len()];
    }

    /// Appends to `out` each struct, union or array that `ty` (a declared field type) owns slots of:
    /// the layout constructor of each one occurring applied to all its arguments, outside the
    /// head of an application of a type variable.
    ///
    /// Must stay in sync with `push_own_slots` on which type constructors own slots.
    fn collect_owning_tycons(&self, ty: &Arc<TypeNode>, out: &mut Vec<TyCon>) {
        let spine = ty.flatten_type_application();
        let args = &spine[1..];
        match &spine[0].ty {
            Type::TyVar(_) => {}
            Type::TyCon(tc) => {
                if args.len() < self.arity(tc) {
                    return;
                }
                if !tc.is_arrow() {
                    out.push(self.layout_tycon(tc));
                }
            }
            _ => panic!("the type `{}` has an unexpected head", ty.to_string()),
        }
        for arg in args {
            self.collect_owning_tycons(arg, out);
        }
    }

    /// The own slots of the group `group`.
    fn group_slots(&self, group: usize) -> Arc<GroupSlots> {
        if let Some(slots) = &self.group_slots.borrow()[group] {
            return slots.clone();
        }
        let mut own_slots = vec![];
        let mut field_offsets = Map::default();
        for member in &self.group_members[group] {
            let mut offsets = vec![];
            let info = self.type_env.tycon_info(member);
            for field in &info.fields {
                offsets.push(own_slots.len());
                if !self.type_env.is_unwrapped_newtype(member) {
                    self.push_own_slots(&field.ty, group, &mut own_slots);
                }
            }
            field_offsets.insert(member.clone(), offsets);
        }
        for member in &self.group_members[group] {
            if DESTRUCTOR_TYPE.is(&member.name) {
                for slot in &mut own_slots {
                    *slot = Some(member.clone());
                }
            }
        }
        let slots = Arc::new(GroupSlots {
            own_slots,
            field_offsets,
        });
        self.group_slots.borrow_mut()[group] = Some(slots.clone());
        slots
    }

    /// Appends to `own` an entry for each own slot that the declared field type `ty` of a member of
    /// `group` gives the group, in the order `walk_declared` reads them.
    ///
    /// Must stay in sync with `walk_declared` on the order a declared type is read in, and with
    /// `collect_owning_tycons` on which type constructors own slots.
    fn push_own_slots(&self, ty: &Arc<TypeNode>, group: usize, own: &mut Vec<Option<TyCon>>) {
        let spine = ty.flatten_type_application();
        let args = &spine[1..];
        match &spine[0].ty {
            Type::TyVar(_) => {}
            Type::TyCon(tc) => {
                if args.len() < self.arity(tc) {
                    return;
                }
                assert!(
                    !self.type_env.is_unwrapped_newtype(tc),
                    "a declared field type holds the unwrapped newtype `{}` saturated",
                    tc.to_string()
                );
                if tc.is_arrow() {
                    own.push(None);
                } else {
                    let layout_tc = self.layout_tycon(tc);
                    let tc_group = self.group_of[&layout_tc];
                    if tc_group != group {
                        own.extend(self.group_slots(tc_group).own_slots.iter().cloned());
                    }
                }
            }
            _ => panic!("the type `{}` has an unexpected head", ty.to_string()),
        }
        for arg in args {
            self.push_own_slots(arg, group, own);
        }
    }

    /// The number of own slots of the type constructor `tc` applied to all its arguments.
    pub(super) fn own_slot_count(&self, tc: &TyCon) -> usize {
        if tc.is_arrow() {
            return 1;
        }
        let group = self.group_of[&self.layout_tycon(tc)];
        self.group_slots(group).own_slots.len()
    }

    /// The number of slots of `ty`.
    pub fn slot_count(&self, ty: &Arc<TypeNode>) -> usize {
        if let Some(count) = self.slot_counts.borrow().get(ty) {
            return *count;
        }
        let tc = head_tycon(ty);
        let spine = ty.flatten_type_application();
        let args = &spine[1..];
        let count = if args.len() < self.arity(&tc) {
            0
        } else {
            self.assert_not_unwrapped_newtype(&tc, ty);
            self.own_slot_count(&tc) + args.iter().map(|a| self.slot_count(a)).sum::<usize>()
        };
        self.slot_counts.borrow_mut().insert(ty.clone(), count);
        count
    }

    /// Panics unless `tc`, at the head of the type `ty` of a value, is a type values are built at.
    fn assert_not_unwrapped_newtype(&self, tc: &TyCon, ty: &Arc<TypeNode>) {
        assert!(
            !self.type_env.is_unwrapped_newtype(tc),
            "the type `{}` of a value holds the unwrapped newtype `{}` saturated",
            ty.to_string(),
            tc.to_string()
        );
    }

    /// `ty` annotated with a new variable at each slot. A slot whose functions the runtime calls is
    /// made Dynamic.
    pub fn fresh(&self, ty: &Arc<TypeNode>, solver: &mut Solver) -> AType {
        let mut slots = vec![];
        self.fresh_slots(ty, solver, &mut slots);
        AType {
            ty: ty.clone(),
            slots,
        }
    }

    /// Appends to `out` a new variable for each slot of `ty`, as `fresh` makes them.
    fn fresh_slots(&self, ty: &Arc<TypeNode>, solver: &mut Solver, out: &mut Vec<SetVar>) {
        let tc = head_tycon(ty);
        let spine = ty.flatten_type_application();
        let args = &spine[1..];
        if args.len() < self.arity(&tc) {
            return;
        }
        self.assert_not_unwrapped_newtype(&tc, ty);
        if tc.is_arrow() {
            out.push(solver.fresh());
        } else {
            let group = self.group_of[&self.layout_tycon(&tc)];
            for called_by in &self.group_slots(group).own_slots {
                let v = solver.fresh();
                if let Some(called_by) = called_by {
                    solver.make_dynamic(v, DynamicKind::CalledByRuntime(called_by.clone()));
                }
                out.push(v);
            }
        }
        for arg in args {
            self.fresh_slots(arg, solver, out);
        }
    }

    /// The `i`-th type argument of `ty`, with its slots.
    pub fn type_arg(&self, ty: &AType, i: usize) -> AType {
        let tc = head_tycon(&ty.ty);
        let spine = ty.ty.flatten_type_application();
        let args = &spine[1..];
        let mut start = self.own_slot_count(&tc);
        for arg in &args[..i] {
            start += self.slot_count(arg);
        }
        let len = self.slot_count(&args[i]);
        AType {
            ty: args[i].clone(),
            slots: ty.slots[start..start + len].to_vec(),
        }
    }

    /// `ty` written with the text `show` gives each of its slots: a function as `a -[s]-> b`, and a
    /// type constructor owning slots as `T<s, t>`.
    pub fn annotate(&self, ty: &AType, show: &mut dyn FnMut(SetVar) -> String) -> String {
        let mut pos = 0;
        let text = self.annotate_at(&ty.ty, &ty.slots, &mut pos, show);
        assert_eq!(pos, ty.slots.len());
        text
    }

    /// `annotate` for the part `ty` of a type whose slots are `slots`, the first of them at `pos`.
    fn annotate_at(
        &self,
        ty: &Arc<TypeNode>,
        slots: &[SetVar],
        pos: &mut usize,
        show: &mut dyn FnMut(SetVar) -> String,
    ) -> String {
        let tc = head_tycon(ty);
        let spine = ty.flatten_type_application();
        let args = &spine[1..];
        if args.len() < self.arity(&tc) {
            return ty.to_string();
        }
        if tc.is_arrow() {
            let own = show(slots[*pos]);
            *pos += 1;
            let param = self.annotate_at(&args[0], slots, pos, show);
            let ret = self.annotate_at(&args[1], slots, pos, show);
            return format!("({} -[{}]-> {})", param, own, ret);
        }
        let own_count = self.own_slot_count(&tc);
        let own: Vec<String> = (0..own_count).map(|i| show(slots[*pos + i])).collect();
        *pos += own_count;
        let mut text = tc.to_string();
        if !own.is_empty() {
            text += &format!("<{}>", own.join(", "));
        }
        for arg in args {
            let arg = self.annotate_at(arg, slots, pos, show);
            if arg.contains(' ') && !arg.starts_with('(') {
                text += &format!(" ({})", arg);
            } else {
                text += &format!(" {}", arg);
            }
        }
        text
    }

    /// The own slots of `ty`: the slots its outermost type constructor owns.
    pub fn own_slots<'a>(&self, ty: &'a AType) -> &'a [SetVar] {
        let tc = head_tycon(&ty.ty);
        &ty.slots[..self.own_slot_count(&tc)]
    }

    /// Appends to `out` the own slot of each function a value of `ty` holds in place: the value
    /// itself where it is a function, and otherwise each function reached from it through fields
    /// of unboxed structs and unions alone. A function whose set is represented by a capture struct
    /// is laid out where it is held, so these are the slots whose capture structs a value of `ty`
    /// contains. `solver` gives the new variables of a place the slots of `ty` leave untracked.
    pub fn held_in_place(&self, ty: &AType, solver: &mut Solver, out: &mut Vec<SetVar>) {
        let tc = head_tycon(&ty.ty);
        if ty.ty.flatten_type_application().len() - 1 < self.arity(&tc) {
            return;
        }
        if tc.is_arrow() {
            out.push(ty.slots[0]);
            return;
        }
        let info = self.type_env.tycon_info(&self.layout_tycon(&tc));
        let is_struct_or_union = matches!(info.variant, TyConVariant::Struct | TyConVariant::Union);
        if !is_struct_or_union || !info.is_unbox {
            return;
        }
        for i in 0..info.fields.len() {
            let field = self.field(ty, i, solver);
            self.held_in_place(&field, solver, out);
        }
    }

    /// The field `field_idx` of the struct, the variant `field_idx` of the union, or (with
    /// `field_idx` 0) the element of the array `ty`, with its slots. `solver` gives the new
    /// variables of a place the slots of `ty` leave untracked.
    pub fn field(&self, ty: &AType, field_idx: usize, solver: &mut Solver) -> AType {
        let tc = head_tycon(&ty.ty);
        let layout_tc = self.layout_tycon(&tc);
        let group = self.group_of[&layout_tc];
        let group_slots = self.group_slots(group);
        let info = self.type_env.tycon_info(&layout_tc);
        let own_count = group_slots.own_slots.len();
        let mut env: Map<Name, Binding> = Map::default();
        for (i, tv) in info.tyvars.iter().enumerate() {
            let arg = self.type_arg(ty, i);
            env.insert(tv.name.clone(), Binding::Linked(arg.ty, arg.slots));
        }
        let mut source = OwnSource::Declared {
            group,
            own: &ty.slots[..own_count],
            cursor: group_slots.field_offsets[&layout_tc][field_idx],
        };
        let mut slots = vec![];
        self.walk_declared(
            &info.fields[field_idx].ty,
            &env,
            &mut source,
            solver,
            &mut slots,
        );
        let field_ty = ty.ty.field_types(self.type_env)[field_idx].clone();
        assert_eq!(
            slots.len(),
            self.slot_count(&field_ty),
            "the field {} of `{}` is read with a number of slots its type `{}` does not have",
            field_idx,
            ty.ty.to_string(),
            field_ty.to_string()
        );
        AType {
            ty: field_ty,
            slots,
        }
    }

    /// Appends to `out` the slots of the declared type `ty` read at the instance `env` binds its
    /// type variables to: the slots of the type `TypeNode::field_types` gives for it there.
    ///
    /// Must stay in sync with `push_own_slots` on the order a declared type is read in.
    fn walk_declared(
        &self,
        ty: &Arc<TypeNode>,
        env: &Map<Name, Binding>,
        source: &mut OwnSource,
        solver: &mut Solver,
        out: &mut Vec<SetVar>,
    ) {
        let spine = ty.flatten_type_application();
        let args = &spine[1..];
        match &spine[0].ty {
            Type::TyVar(tv) => {
                let binding = &env[&tv.name];
                if args.is_empty() {
                    match binding {
                        Binding::Linked(_, slots) => out.extend(slots.iter().copied()),
                        Binding::Untracked(arg_ty) => {
                            let reason = untracked_reason(source);
                            push_untracked(self.slot_count(arg_ty), &reason, solver, out);
                        }
                    }
                    return;
                }
                // A type variable applied to arguments: the type it stands for is a type
                // constructor given without its arguments, and the type argument it was given to
                // owns none of its slots. The arguments here are written in the declaration, so
                // they are read as it reads them, each once and in order.
                let mut head_args: Vec<Binding> = vec![];
                let head_tc = head_tycon(binding.ty());
                let head_spine = binding.ty().flatten_type_application();
                for given in &head_spine[1..] {
                    head_args.push(Binding::Untracked(given.clone()));
                }
                for arg in args {
                    let mut slots = vec![];
                    self.walk_declared(arg, env, source, solver, &mut slots);
                    head_args.push(Binding::Linked(instantiate(arg, env), slots));
                }
                self.walk_untracked_application(&head_tc, head_args, solver, out);
            }
            Type::TyCon(tc) => {
                if args.len() < self.arity(tc) {
                    return;
                }
                match source {
                    OwnSource::Declared { group, own, cursor } => {
                        let own_count = self.own_slot_count(tc);
                        let in_group =
                            !tc.is_arrow() && self.group_of[&self.layout_tycon(tc)] == *group;
                        if in_group {
                            out.extend(own.iter().copied());
                        } else {
                            out.extend(own[*cursor..*cursor + own_count].iter().copied());
                            *cursor += own_count;
                        }
                    }
                    OwnSource::Untracked(reason) => {
                        push_untracked(self.own_slot_count(tc), reason, solver, out);
                    }
                }
                for arg in args {
                    self.walk_declared(arg, env, source, solver, out);
                }
            }
            _ => panic!(
                "the declared type `{}` has an unexpected head",
                ty.to_string()
            ),
        }
    }

    /// Appends to `out` the slots of the type constructor `tc` applied to `args`, as a type argument
    /// given without its arguments becomes once a declaration applies it. What `tc` itself
    /// contributes is untracked; what the arguments contribute is as they are bound.
    fn walk_untracked_application(
        &self,
        tc: &TyCon,
        args: Vec<Binding>,
        solver: &mut Solver,
        out: &mut Vec<SetVar>,
    ) {
        if args.len() < self.arity(tc) {
            return;
        }
        if self.type_env.is_unwrapped_newtype(tc) {
            let info = self.type_env.tycon_info(tc);
            if info.fields[0].is_punched {
                return;
            }
            let env: Map<Name, Binding> = info
                .tyvars
                .iter()
                .zip(args)
                .map(|(tv, b)| (tv.name.clone(), b))
                .collect();
            let mut source = OwnSource::Untracked(tc.clone());
            self.walk_declared(&info.fields[0].ty, &env, &mut source, solver, out);
            return;
        }
        push_untracked(self.own_slot_count(tc), tc, solver, out);
        for arg in args {
            match arg {
                Binding::Linked(_, slots) => out.extend(slots),
                Binding::Untracked(ty) => push_untracked(self.slot_count(&ty), tc, solver, out),
            }
        }
    }
}

/// The type constructor at the head of `ty`, the type of a value or a type argument given to a
/// declaration.
fn head_tycon(ty: &Arc<TypeNode>) -> Arc<TyCon> {
    ty.toplevel_tycon().unwrap_or_else(|| {
        panic!(
            "the type `{}` is headed by no type constructor",
            ty.to_string()
        )
    })
}

/// Appends to `out` `count` new variables, each made Dynamic as a place inside a type argument of
/// `reason`.
fn push_untracked(count: usize, reason: &TyCon, solver: &mut Solver, out: &mut Vec<SetVar>) {
    for _ in 0..count {
        let v = solver.fresh();
        solver.make_dynamic(v, DynamicKind::TypeConstructorArgument(reason.clone()));
        out.push(v);
    }
}

/// The type constructor an untracked place read under `source` is attributed to.
fn untracked_reason(source: &OwnSource) -> TyCon {
    match source {
        OwnSource::Untracked(tc) => tc.clone(),
        OwnSource::Declared { .. } => {
            unreachable!(
                "a declaration read at its own instance binds every variable to a type it tracks"
            )
        }
    }
}

/// The declared type `ty` with each of its type variables replaced by the type `env` binds it to.
fn instantiate(ty: &Arc<TypeNode>, env: &Map<Name, Binding>) -> Arc<TypeNode> {
    match &ty.ty {
        Type::TyVar(tv) => env[&tv.name].ty().clone(),
        Type::TyCon(_) => ty.clone(),
        Type::TyApp(fun, arg) => type_tyapp(instantiate(fun, env), instantiate(arg, env)),
        Type::AssocTy(_, _) => panic!(
            "the declared type `{}` has an associated type",
            ty.to_string()
        ),
    }
}
