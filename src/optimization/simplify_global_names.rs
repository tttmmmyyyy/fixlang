/*
Simplify global names.

This pass simplifies global names such as "Std::func#{something_added_by_compiler}#{another_thing_added_by_compiler}" to "Std::func#{a-number}".

Used to make the symbol file more readable.
*/

use std::{mem, sync::Arc};

use crate::{
    ast::{
        name::FullName,
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
    },
    misc::{insert_to_map_vec, Map},
    ExprNode, Program, INSTANCIATED_NAME_SEPARATOR,
};

pub fn run(prg: &mut Program) {
    // Get all names and unique them.
    let mut all_names = vec![];
    for (name, _sym) in &mut prg.instantiated_symbols {
        all_names.push(name.clone());
    }
    all_names.sort();
    all_names.dedup();

    // Construct map from the base name (e.g., `Std::func`) to the list of full names (e.g., `Std::func#{something}#{another}`).
    let mut base_to_full: Map<FullName, Vec<FullName>> = Map::default();
    for name in all_names {
        let base_name = get_base_name(&name);
        insert_to_map_vec(&mut base_to_full, &base_name, name);
    }

    // Determine new names.
    let mut old_to_new_names: Map<FullName, FullName> = Map::default();
    for (base_name, full_names) in base_to_full {
        for (name_no, full_name) in full_names.into_iter().enumerate() {
            let new_name = format!(
                "{}{}{}",
                base_name.name, INSTANCIATED_NAME_SEPARATOR, name_no
            );
            let new_name = FullName::new(&base_name.namespace, &new_name);
            old_to_new_names.insert(full_name, new_name);
        }
    }

    // Perform the renaming.
    let mut visitor = SimplifyName {
        old_to_new_names: old_to_new_names.clone(),
    };
    let old_symbols = mem::take(&mut prg.instantiated_symbols);
    let mut new_symbols = Map::default();
    for (old_name, mut sym) in old_symbols {
        let res = visitor.traverse(sym.expr.as_ref().unwrap());
        if res.changed {
            sym.expr = Some(res.expr.calculate_free_vars());
        }
        let new_name = old_to_new_names.get(&old_name).unwrap();
        new_symbols.insert(new_name.clone(), sym);
    }
    prg.instantiated_symbols = new_symbols;
}

fn get_base_name(full_name: &FullName) -> FullName {
    let name = &full_name.name;
    let new_name = if name.starts_with('#') {
        // To avoid the name becomes empty, remove after second '#' if exists.
        let second_sharp = name[1..].find("#");
        match second_sharp {
            Some(pos) => name[0..pos].to_string(),
            None => name.clone(),
        }
    } else {
        // Remove after first '#' if exists.
        let first_sharp = name.find("#");
        match first_sharp {
            Some(pos) => name[0..pos].to_string(),
            None => name.clone(),
        }
    };
    FullName::new(&full_name.namespace, &new_name)
}

#[test]
fn test_get_base_name() {
    assert_eq!(
        get_base_name(&FullName::from_strs(&["Std"], "func")),
        FullName::from_strs(&["Std"], "func")
    );
    assert_eq!(
        get_base_name(&FullName::from_strs(&["Std"], "func#1")),
        FullName::from_strs(&["Std"], "func")
    );
    assert_eq!(
        get_base_name(&FullName::from_strs(&["Std"], "func#1#2")),
        FullName::from_strs(&["Std"], "func")
    );
    assert_eq!(
        get_base_name(&FullName::from_strs(&["Std"], "#func")),
        FullName::from_strs(&["Std"], "#func")
    );
    assert_eq!(
        get_base_name(&FullName::from_strs(&["Std"], "#func#1")),
        FullName::from_strs(&["Std"], "#func")
    );
    assert_eq!(
        get_base_name(&FullName::from_strs(&["Std"], "#func#1#2")),
        FullName::from_strs(&["Std"], "#func")
    );
}

struct SimplifyName {
    old_to_new_names: Map<FullName, FullName>,
}

impl ExprVisitor for SimplifyName {
    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let var = expr.get_var();
        let name = &var.clone().name;
        if let Some(new_name) = self.old_to_new_names.get(name) {
            let new_var = var.set_name(new_name.clone());
            let new_expr = expr.set_var_var(new_var);
            return EndVisitResult::changed(new_expr);
        }
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_var(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_app(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_app(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_lam(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_let(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_let(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_if(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_if(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_match(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_match(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_tyanno(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_tyanno(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_make_struct(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_make_struct(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_array_lit(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_array_lit(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_ffi_call(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_ffi_call(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
}
