use super::*;

// Haskell Core:
//
// data Expr
//   = Var Var
//   | Lit Literal
//   | App Expr Expr
//   | Lam Var Expr -- Both term and type lambda
//   | Let Bind Expr
//   | Case Expr Var Type [(AltCon, [Var], Expr)]
//   | Type Type -- Used for type application

// data Var = Id Name Type -- Term variable
//   | TyVar Name Kind -- Type variable

// data Type = TyVarTy Var
//   | LitTy TyLit
//   | AppTy Type Type
//   | TyConApp TyCon [Type]
//   | FunTy Type Type
//   | ForAllTy Var Type

pub struct ExprInfo {
    pub expr: Arc<Expr>,
    pub free_vars: HashSet<String>,
    code: String,
}

impl ExprInfo {
    fn with_free_vars(self: &Arc<Self>, free_vars: HashSet<String>) -> Arc<ExprInfo> {
        Arc::new(ExprInfo {
            expr: self.expr.clone(),
            free_vars,
            code: self.code.clone(),
        })
    }
}

#[derive(Clone)]
pub enum Expr {
    Var(Arc<Var>),
    Lit(Arc<Literal>),
    App(Arc<ExprInfo>, Arc<ExprInfo>),
    Lam(Arc<Var>, Arc<ExprInfo>),
    Let(Arc<Var>, Arc<ExprInfo>, Arc<ExprInfo>),
    // TODO: Implement case
    If(Arc<ExprInfo>, Arc<ExprInfo>, Arc<ExprInfo>),
    Type(Arc<Type>),
}

impl Expr {
    fn into_expr_info(self: &Arc<Self>) -> Arc<ExprInfo> {
        Arc::new(ExprInfo {
            expr: self.clone(),
            free_vars: Default::default(),
            code: Default::default(),
        })
    }
    pub fn to_string(&self) -> String {
        match self {
            Expr::Var(v) => v.name().clone(),
            Expr::Lit(l) => l.name.clone(),
            Expr::App(f, a) => format!("({}) ({})", f.expr.to_string(), a.expr.to_string()),
            Expr::Lam(x, fx) => format!("\\{}->({})", x.name(), fx.expr.to_string()),
            Expr::Let(x, b, v) => format!(
                "let {}={} in ({})",
                x.name(),
                b.expr.to_string(),
                v.expr.to_string()
            ),
            Expr::If(c, t, e) => format!(
                "if {} then {} else ({})",
                c.expr.to_string(),
                t.expr.to_string(),
                e.expr.to_string()
            ),
            Expr::Type(_) => todo!(),
        }
    }
}

pub type LiteralGenerator =
    dyn Send + Sync + for<'c, 'm, 'b> Fn(&mut GenerationContext<'c, 'm>) -> ExprCode<'c>;

pub struct Literal {
    pub generator: Arc<LiteralGenerator>,
    pub free_vars: Vec<String>, // e.g. "+" literal has two free variables.
    name: String,
}

#[derive(Eq, PartialEq)]
pub enum Var {
    TermVar { name: String },
    TyVar { name: String },
}

impl Var {
    pub fn name(self: &Self) -> &String {
        match self {
            Var::TermVar { name } => name,
            Var::TyVar { name } => name,
        }
    }
}

#[derive(Eq, PartialEq)]
enum Kind {
    Star,
    Arrow(Arc<Kind>, Arc<Kind>),
}

#[derive(Eq, PartialEq)]
struct TyLit {
    value: String,
}

#[derive(Eq, PartialEq)]
pub enum Type {
    TyVar(Arc<Var>),
    LitTy(Arc<TyLit>),
    AppTy(Arc<Type>, Arc<Type>),
    TyConApp(Arc<TyCon>, Vec<Type>),
    FunTy(Arc<Type>, Arc<Type>),
    ForAllTy(Arc<Var>, Arc<Type>),
}

#[derive(Eq, PartialEq)]
enum TyCon {
    Pair,
}

fn star_kind() -> Arc<Kind> {
    Arc::new(Kind::Star)
}

fn lambda_ty(src: Arc<Type>, dst: Arc<Type>) -> Arc<Type> {
    Arc::new(Type::FunTy(src, dst))
}

fn tyvar_var(var_name: &str) -> Arc<Var> {
    Arc::new(Var::TyVar {
        name: String::from(var_name),
    })
}

fn tyvar_ty(var_name: &str) -> Arc<Type> {
    Arc::new(Type::TyVar(tyvar_var(var_name)))
}

fn forall_ty(var_name: &str, ty: Arc<Type>) -> Arc<Type> {
    Arc::new(Type::ForAllTy(tyvar_var(var_name), ty))
}

pub fn var_var(var_name: &str) -> Arc<Var> {
    Arc::new(Var::TermVar {
        name: String::from(var_name),
    })
}

fn lit(generator: Arc<LiteralGenerator>, free_vars: Vec<String>, name: String) -> Arc<ExprInfo> {
    Arc::new(Expr::Lit(Arc::new(Literal {
        generator,
        free_vars,
        name,
    })))
    .into_expr_info()
}

pub fn let_in(var: Arc<Var>, bound: Arc<ExprInfo>, expr: Arc<ExprInfo>) -> Arc<ExprInfo> {
    Arc::new(Expr::Let(var, bound, expr)).into_expr_info()
}

pub fn lam(var: Arc<Var>, val: Arc<ExprInfo>) -> Arc<ExprInfo> {
    Arc::new(Expr::Lam(var, val)).into_expr_info()
}

pub fn app(lam: Arc<ExprInfo>, arg: Arc<ExprInfo>) -> Arc<ExprInfo> {
    Arc::new(Expr::App(lam, arg)).into_expr_info()
}

pub fn var(var_name: &str) -> Arc<ExprInfo> {
    Arc::new(Expr::Var(var_var(var_name))).into_expr_info()
}

pub fn conditional(
    cond: Arc<ExprInfo>,
    then_expr: Arc<ExprInfo>,
    else_expr: Arc<ExprInfo>,
) -> Arc<ExprInfo> {
    Arc::new(Expr::If(cond, then_expr, else_expr)).into_expr_info()
}

pub fn int(val: i64) -> Arc<ExprInfo> {
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let ptr_to_int_obj = ObjectType::int_obj_type()
            .build_allocate_shared_obj(gc, Some(val.to_string().as_str()));
        let value = gc.context.i64_type().const_int(val as u64, false);
        gc.store_obj_field(
            ptr_to_int_obj,
            ObjectType::int_obj_type().to_struct_type(gc.context),
            1,
            value,
        );
        ExprCode {
            ptr: ptr_to_int_obj,
        }
    });
    lit(generator, vec![], val.to_string())
}

pub fn bool(val: bool) -> Arc<ExprInfo> {
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let ptr_to_obj = ObjectType::bool_obj_type()
            .build_allocate_shared_obj(gc, Some(val.to_string().as_str()));
        let value = gc.context.i8_type().const_int(val as u64, false);
        gc.store_obj_field(
            ptr_to_obj,
            ObjectType::bool_obj_type().to_struct_type(gc.context),
            1,
            value,
        );
        ExprCode { ptr: ptr_to_obj }
    });
    lit(generator, vec![], val.to_string())
}

fn add_lit(lhs: &str, rhs: &str) -> Arc<ExprInfo> {
    let lhs_str = String::from(lhs);
    let rhs_str = String::from(rhs);
    let free_vars = vec![lhs_str.clone(), rhs_str.clone()];
    let name = format!("add {} {}", lhs, rhs);
    let name_cloned = name.clone();
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let lhs_val = gc
            .scope_get_field(
                &lhs_str,
                1,
                ObjectType::int_obj_type().to_struct_type(gc.context),
            )
            .into_int_value();
        let rhs_val = gc
            .scope_get_field(
                &rhs_str,
                1,
                ObjectType::int_obj_type().to_struct_type(gc.context),
            )
            .into_int_value();
        let value = gc.builder().build_int_add(lhs_val, rhs_val, "add");
        let ptr_to_int_obj =
            ObjectType::int_obj_type().build_allocate_shared_obj(gc, Some(name_cloned.as_str()));
        gc.store_obj_field(
            ptr_to_int_obj,
            ObjectType::int_obj_type().to_struct_type(gc.context),
            1,
            value,
        );
        gc.release(gc.scope_get(&lhs_str).code.ptr);
        gc.release(gc.scope_get(&rhs_str).code.ptr);
        ExprCode {
            ptr: ptr_to_int_obj,
        }
    });
    lit(generator, free_vars, name)
}

pub fn add() -> Arc<ExprInfo> {
    lam(var_var("lhs"), lam(var_var("rhs"), add_lit("lhs", "rhs")))
}

fn eq_lit(lhs: &str, rhs: &str) -> Arc<ExprInfo> {
    let lhs_str = String::from(lhs);
    let rhs_str = String::from(rhs);
    let name = format!("eq {} {}", lhs, rhs);
    let name_cloned = name.clone();
    let free_vars = vec![lhs_str.clone(), rhs_str.clone()];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let lhs_val = gc
            .scope_get_field(
                &lhs_str,
                1,
                ObjectType::int_obj_type().to_struct_type(gc.context),
            )
            .into_int_value();
        let rhs_val = gc
            .scope_get_field(
                &rhs_str,
                1,
                ObjectType::int_obj_type().to_struct_type(gc.context),
            )
            .into_int_value();
        let value = gc
            .builder()
            .build_int_compare(IntPredicate::EQ, lhs_val, rhs_val, "eq");
        let value = gc.builder().build_int_cast(
            value,
            ObjectFieldType::Bool
                .to_basic_type(gc.context)
                .into_int_type(),
            "eq_bool",
        );
        let ptr_to_obj =
            ObjectType::bool_obj_type().build_allocate_shared_obj(gc, Some(name_cloned.as_str()));
        gc.store_obj_field(
            ptr_to_obj,
            ObjectType::bool_obj_type().to_struct_type(gc.context),
            1,
            value,
        );
        gc.release(gc.scope_get(&lhs_str).code.ptr);
        gc.release(gc.scope_get(&rhs_str).code.ptr);
        ExprCode { ptr: ptr_to_obj }
    });
    lit(generator, free_vars, name)
}

pub fn eq() -> Arc<ExprInfo> {
    lam(var_var("lhs"), lam(var_var("rhs"), eq_lit("lhs", "rhs")))
}

fn fix_lit(f: &str, x: &str) -> Arc<ExprInfo> {
    let f_str = String::from(f);
    let x_str = String::from(x);
    let name = format!("fix {} {}", f_str, x_str);
    let free_vars = vec![String::from(SELF_NAME), f_str.clone(), x_str.clone()];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let fixf = gc.scope_get(SELF_NAME).code.ptr;
        let x = gc.scope_get(&x_str).code.ptr;
        let f = gc.scope_get(&f_str).code.ptr;
        let f_fixf = gc.apply_lambda(f, fixf).ptr;
        let f_fixf_x = gc.apply_lambda(f_fixf, x).ptr;
        ExprCode { ptr: f_fixf_x }
    });
    lit(generator, free_vars, name)
}

pub fn fix() -> Arc<ExprInfo> {
    lam(var_var("f"), lam(var_var("x"), fix_lit("f", "x")))
}

// TODO: use persistent binary search tree as ExprAuxInfo to avoid O(n^2) complexity of calculate_aux_info.
pub fn calculate_aux_info(ei: Arc<ExprInfo>) -> Arc<ExprInfo> {
    match &*ei.expr {
        Expr::Var(var) => {
            let free_vars = vec![var.name().clone()].into_iter().collect();
            ei.with_free_vars(free_vars)
        }
        Expr::Lit(lit) => {
            let free_vars = lit.free_vars.clone().into_iter().collect();
            ei.with_free_vars(free_vars)
        }
        Expr::App(func, arg) => {
            let func = calculate_aux_info(func.clone());
            let arg = calculate_aux_info(arg.clone());
            let mut free_vars = func.free_vars.clone();
            free_vars.extend(arg.free_vars.clone());
            app(func, arg).with_free_vars(free_vars)
        }
        Expr::Lam(arg, val) => {
            let val = calculate_aux_info(val.clone());
            let mut free_vars = val.free_vars.clone();
            free_vars.remove(arg.name());
            free_vars.remove(SELF_NAME);
            lam(arg.clone(), val).with_free_vars(free_vars)
        }
        Expr::Let(var, bound, val) => {
            // NOTE: Our Let is non-recursive let, i.e.,
            // "let x = f x in g x" is equal to "let y = f x in g y",
            // and x ∈ FreeVars("let x = f x in g x") = (FreeVars(g x) - {x}) + FreeVars(f x) != (FreeVars(g x) + FreeVars(f x)) - {x}.
            let bound = calculate_aux_info(bound.clone());
            let val = calculate_aux_info(val.clone());
            let mut free_vars = val.free_vars.clone();
            free_vars.remove(var.name());
            free_vars.extend(bound.free_vars.clone());
            let_in(var.clone(), bound, val).with_free_vars(free_vars)
        }
        Expr::If(cond, then, else_expr) => {
            let cond = calculate_aux_info(cond.clone());
            let then = calculate_aux_info(then.clone());
            let else_expr = calculate_aux_info(else_expr.clone());
            let mut free_vars = cond.free_vars.clone();
            free_vars.extend(then.free_vars.clone());
            free_vars.extend(else_expr.free_vars.clone());
            conditional(cond, then, else_expr).with_free_vars(free_vars)
        }
        Expr::Type(_) => ei.clone(),
    }
}
