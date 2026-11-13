use crate::ast::*;
use std::collections::HashMap;
use std::collections::HashSet;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    //符号付き整数
    I8, I16, I32, I64, I128,
    //符号なし整数
    U8, U16, U32, U64, U128,
    //浮動小数点数
    F16, F32, F64, F128,
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
}impl Type {
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

pub fn Program_Analyzer(Program_val :Program){
    let mut Type_Manager=TypeTable::new();
    let mut Variables:HashMap<String,usize>=HashMap::new();
    for line in Program_val{
        Stmt_Amalyzer(line,&mut Type_Manager,&mut Variables);
    }
}pub fn Stmt_Amalyzer(Statement :Stmt,Type_Manager :&mutTypeTable,Variables: &mut HashMap<String,usize>){
    match Statement{
        Stmt::Assign(target,op,value)=>{
            let target_id=match Variables.get(&target){
                Some(&id)=>id,
                None=>expect("{target} is not defined."),
            };//Type_Manager.unify(target_id, value_id).expect...
        }
        Stmt::Break=>//まだ書いてない(Forの中か判定。スタックつかったらよさそう)
        Stmt::Continue=>//まだ書いてない(Forの中か判定)
        Stmt::Expr(Expression)=>//まだ書いてない
        Stmt::For(Var,Iteration,Program_val)=>//これの意味はあとで聞く
        Stmt::If(Expression,Program_val,Program_else_val)=>//まだ
        Stmt::Let(Var,Init)=>{Type_Management.entry(Var).or_insert(Init);}//すでにVarがあった場合はエラーを出します？
        Stmt::Return(Expression)=>//まだ
        Stmt::While(Expression,Program_val)=>//まだ
    }
}pub fn Expr_Analyzer(Expr_val :Expr)->usize{
    match Expr_val{
        Expr::Char=>{
            return Type.from_str("char");
        }
        Expr::Int=>{
            return Type.from_str("i64");
        }
        Expr::Float=>{
            return Type.from_str("f64");
        }
        Expr::Flase=>{
            return Type.from_str("bool");
        }
        Expr::True=>{
            return Type.from_str("bool");
        }Expr::BinaryOp(left,op,right)=>{
            //右側と左側どう分けようかな
        }Expr::Call(calle,args)=>{
            //何だっけこれ
        }Expr::Function=>{
            //めんどくさいから後回しでいいかこれ
        }Expr::Unary(target,op)=>{
            return Expr_Analyzer(*target);
        }Expr::Variable=>{
            let target_id=match Variables.get(&target){
                Some(&id)=>id,
                None=>expect("{target} is not defined."),
            };
            return target_id;
        }
    }
}