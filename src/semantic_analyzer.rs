use crate::ast::*;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    //符号付き整数
    I8,
    I16,
    I32,
    I64,
    I128,
    //符号なし整数
    U8,
    U16,
    U32,
    U64,
    U128,
    //浮動小数点数
    F16,
    F32,
    F64,
    F128,
    //文字
    Char,
    ShortChar,
    //真偽値
    Bool,
    //文字列
    String,
    //何も返さない（戻り値用）
    Void,
    //確定していない場合
    Unknown(usize),
}
impl Type {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "i8" => Some(Type::I8),
            "i16" => Some(Type::I16),
            "i32" => Some(Type::I32),
            "i64" => Some(Type::I64),
            "i128" => Some(Type::I128),
            "u8" => Some(Type::U8),
            "u16" => Some(Type::U16),
            "u32" => Some(Type::U32),
            "u64" => Some(Type::U64),
            "u128" => Some(Type::U128),
            "f16" => Some(Type::F16),
            "f32" => Some(Type::F32),
            "f64" => Some(Type::F64),
            "f128" => Some(Type::F128),
            "char" => Some(Type::Char),
            "short_char" => Some(Type::ShortChar),
            "bool" => Some(Type::Bool),
            "string" => Some(Type::String),
            _ => None,
        }
    }
}
//Union-Find(DSU)
#[derive(Debug, Clone)]
pub struct TypeTable {
    // Union-Find の親ポインタ
    parent: Vec<usize>,
    // 各ノードの現在の型情報
    types: Vec<Type>,
}
impl TypeTable {
    pub fn new() -> Self {
        Self {
            parent: Vec::new(),
            types: Vec::new(),
        }
    }

    /// 新しい型変数（Unknown）を1つ発行してそのIDを返す
    pub fn new_type_var(&mut self) -> usize {
        let id = self.parent.len();
        self.parent.push(id);
        self.types.push(Type::Unknown(id));
        id
    }

    /// 既知の具象型（例: Type::I32）を登録してそのIDを返す
    pub fn new_concrete(&mut self, ty: Type) -> usize {
        let id = self.parent.len();
        self.parent.push(id);
        self.types.push(ty);
        id
    }

    /// 代表元（根）を探索する（経路圧縮付き）
    pub fn find(&mut self, mut x: usize) -> usize {
        let mut root = x;
        while root != self.parent[root] {
            root = self.parent[root];
        }
        // 経路圧縮
        while x != root {
            let next = self.parent[x];
            self.parent[x] = root;
            x = next;
        }
        root
    }

    /// 2つの型IDを統合（単一化 / Unify）する
    pub fn unify(&mut self, a: usize, b: usize) -> Result<(), String> {
        let root_a = self.find(a);
        let root_b = self.find(b);
        if root_a == root_b {
            return Ok(()); // すでに同じグループ
        }

        let type_a = self.types[root_a].clone();
        let type_b = self.types[root_b].clone();

        match (type_a, type_b) {
            // 片方が Unknown なら、もう片方の型に合流させる
            (Type::Unknown(_), concrete) => {
                self.parent[root_a] = root_b;
                self.types[root_b] = concrete;
            }
            (concrete, Type::Unknown(_)) => {
                self.parent[root_b] = root_a;
                self.types[root_a] = concrete;
            }
            // 両方とも具象型の場合：一致していればOK、違えば型不一致エラー
            (t1, t2) if t1 == t2 => {
                self.parent[root_b] = root_a;
            }
            (t1, t2) => {
                return Err(format!("Type mismatch: cannot match {t1:?} with {t2:?}"));
            }
        }
        Ok(())
    }

    /// 最終的に確定した型を取得する
    pub fn get_type(&mut self, id: usize) -> Type {
        let root = self.find(id);
        self.types[root].clone()
    }
}

pub fn program_analyzer(program_val: Program) {
    let mut type_manager = TypeTable::new();
    let mut variables: HashMap<String, usize> = HashMap::new();
    for line in program_val {
        stmt_analyzer(line, &mut type_manager, &mut variables, 0);
    }
}
pub fn stmt_analyzer(
    statement: Stmt,
    type_manager: &mut TypeTable,
    variables: &mut HashMap<String, usize>,
    loop_depth: usize,
) {
    match statement {
        Stmt::Assign {
            target,
            op: _,
            value,
        } => {
            let target_id = *variables
                .get(&target)
                .unwrap_or_else(|| panic!("{target} is not defined."));
            let value_id = expr_analyzer(value, type_manager, variables);
            type_manager
                .unify(target_id, value_id)
                .expect("Type mismatch in assignment");
        }
        Stmt::Break => {
            if loop_depth == 0 {
                panic!("`break` cannot be used outside of a loop");
            }
        }
        Stmt::Continue => {
            if loop_depth == 0 {
                panic!("`continue` cannot be used outside of a loop");
            }
        }
        Stmt::Expr(expr) => {
            expr_analyzer(expr, type_manager, variables);
        }
        Stmt::For { var, iter, body } => {
            let iter_id = expr_analyzer(iter, type_manager, variables);
            let mut body_vars = variables.clone();
            let var_id = type_manager.new_type_var();
            type_manager
                .unify(var_id, iter_id)
                .expect("Type mismatch in for-loop iterator");
            body_vars.insert(var, var_id);

            for stmt in body {
                stmt_analyzer(stmt, type_manager, &mut body_vars, loop_depth + 1);
            }
        }
        Stmt::If {
            cond,
            body,
            else_branch,
        } => {
            let cond_id = expr_analyzer(cond, type_manager, variables);
            let bool_id = type_manager.new_concrete(Type::Bool);
            type_manager
                .unify(cond_id, bool_id)
                .expect("Condition of if statement must be of type Bool");

            let mut body_vars = variables.clone();
            for stmt in body {
                stmt_analyzer(stmt, type_manager, &mut body_vars, loop_depth);
            }

            if let Some(else_stmts) = else_branch {
                let mut else_vars = variables.clone();
                for stmt in else_stmts {
                    stmt_analyzer(stmt, type_manager, &mut else_vars, loop_depth);
                }
            }
        }
        Stmt::Let {
            variable,
            initiator,
        } => {
            let (name, type_annot) = variable;
            if variables.contains_key(&name) {
                panic!("Variable `{name}` is already declared in this scope.");
            }

            let init_type_id = expr_analyzer(initiator, type_manager, variables);

            let var_type_id = if let Some(type_str) = type_annot {
                let annot_type = Type::from_str(&type_str)
                    .unwrap_or_else(|| panic!("Unknown type `{type_str}`"));
                let annot_type_id = type_manager.new_concrete(annot_type);
                type_manager
                    .unify(annot_type_id, init_type_id)
                    .expect("Type mismatch between type annotation and initializer");
                annot_type_id
            } else {
                init_type_id
            };

            variables.insert(name, var_type_id);
        }
        Stmt::Return(expr) => {
            expr_analyzer(expr, type_manager, variables);
        }
        Stmt::While { cond, body } => {
            let cond_id = expr_analyzer(cond, type_manager, variables);
            let bool_id = type_manager.new_concrete(Type::Bool);
            type_manager
                .unify(cond_id, bool_id)
                .expect("Condition of while loop must be of type Bool");

            let mut body_vars = variables.clone();
            for stmt in body {
                stmt_analyzer(stmt, type_manager, &mut body_vars, loop_depth + 1);
            }
        }
    }
}

pub fn expr_analyzer(
    expr_val: Expr,
    type_manager: &mut TypeTable,
    variables: &HashMap<String, usize>,
) -> usize {
    match expr_val {
        Expr::Char(_) => type_manager.new_concrete(Type::Char),
        Expr::Int(_) => type_manager.new_concrete(Type::I64),
        Expr::Float(_) => type_manager.new_concrete(Type::F64),
        Expr::String(_) => type_manager.new_concrete(Type::String),
        Expr::True | Expr::False => type_manager.new_concrete(Type::Bool),
        Expr::Variable(name) => *variables
            .get(&name)
            .unwrap_or_else(|| panic!("Variable `{name}` is not defined.")),
        Expr::BinaryOp { left, op, right } => {
            let left_id = expr_analyzer(*left, type_manager, variables);
            let right_id = expr_analyzer(*right, type_manager, variables);

            match op {
                // 比較演算子: 左右の型が同じで、結果は Bool
                BinaryOpKind::Equal
                | BinaryOpKind::NotEqual
                | BinaryOpKind::Greater
                | BinaryOpKind::Less
                | BinaryOpKind::GreaterEqual
                | BinaryOpKind::LessEqual => {
                    type_manager
                        .unify(left_id, right_id)
                        .expect("Type mismatch in comparison");
                    type_manager.new_concrete(Type::Bool)
                }
                // 論理演算子: 左右とも Bool で、結果も Bool
                BinaryOpKind::LogAnd | BinaryOpKind::LogOr => {
                    let bool_id = type_manager.new_concrete(Type::Bool);
                    type_manager
                        .unify(left_id, bool_id)
                        .expect("Left operand of logical op must be Bool");
                    type_manager
                        .unify(right_id, bool_id)
                        .expect("Right operand of logical op must be Bool");
                    bool_id
                }
                // 範囲演算子 (1..10)
                BinaryOpKind::Range => {
                    type_manager
                        .unify(left_id, right_id)
                        .expect("Range operands must have same type");
                    left_id
                }
                // 算術・ビット演算: 左右の型が同じで、結果もその型
                _ => {
                    type_manager
                        .unify(left_id, right_id)
                        .expect("Arithmetic operands must have same type");
                    left_id
                }
            }
        }
        Expr::UnaryOp { target, op } => {
            let target_id = expr_analyzer(*target, type_manager, variables);
            match op {
                UnaryOpKind::Not => {
                    let bool_id = type_manager.new_concrete(Type::Bool);
                    type_manager
                        .unify(target_id, bool_id)
                        .expect("Operand of logical NOT must be Bool");
                    bool_id
                }
                _ => target_id,
            }
        }
        Expr::Call { calee: _, args } => {
            for arg in args {
                expr_analyzer(arg, type_manager, variables);
            }
            type_manager.new_type_var()
        }
        Expr::Function {
            name: _,
            args: _,
            body: _,
        } => type_manager.new_type_var(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_let_and_assign_success() {
        // let x: i64 = 10; x = 20;
        let program = vec![
            Stmt::Let {
                variable: ("x".into(), Some("i64".into())),
                initiator: Expr::Int("10".into()),
            },
            Stmt::Assign {
                target: "x".into(),
                op: None,
                value: Expr::Int("20".into()),
            },
        ];
        program_analyzer(program);
    }

    #[test]
    #[should_panic(expected = "already declared")]
    fn test_let_duplicate_panic() {
        // let x = 10; let x = 20;
        let program = vec![
            Stmt::Let {
                variable: ("x".into(), None),
                initiator: Expr::Int("10".into()),
            },
            Stmt::Let {
                variable: ("x".into(), None),
                initiator: Expr::Int("20".into()),
            },
        ];
        program_analyzer(program);
    }

    #[test]
    #[should_panic(expected = "Type mismatch")]
    fn test_let_type_mismatch_panic() {
        // let x: bool = 10;
        let program = vec![Stmt::Let {
            variable: ("x".into(), Some("bool".into())),
            initiator: Expr::Int("10".into()),
        }];
        program_analyzer(program);
    }

    #[test]
    fn test_if_and_while_success() {
        // if (true) { let x = 10; } while (false) { break; continue; }
        let program = vec![
            Stmt::If {
                cond: Expr::True,
                body: vec![Stmt::Let {
                    variable: ("x".into(), None),
                    initiator: Expr::Int("10".into()),
                }],
                else_branch: None,
            },
            Stmt::While {
                cond: Expr::False,
                body: vec![Stmt::Break, Stmt::Continue],
            },
        ];
        program_analyzer(program);
    }

    #[test]
    #[should_panic(expected = "outside of a loop")]
    fn test_break_outside_loop_panic() {
        let program = vec![Stmt::Break];
        program_analyzer(program);
    }

    #[test]
    fn test_for_loop_success() {
        // for i in 1..10 { let x = i; }
        let program = vec![Stmt::For {
            var: "i".into(),
            iter: Expr::BinaryOp {
                left: Box::new(Expr::Int("1".into())),
                op: BinaryOpKind::Range,
                right: Box::new(Expr::Int("10".into())),
            },
            body: vec![Stmt::Let {
                variable: ("x".into(), None),
                initiator: Expr::Variable("i".into()),
            }],
        }];
        program_analyzer(program);
    }
}
