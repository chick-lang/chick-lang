use crate::ast::*;
use std::collecions::HashMap;
use std::collecions::HashSet;
let mut Type_Management=HashMap::new();
let mut Types=HashSet::from(
    [String::from("Int"),
    String::from("Float"),
    String::from("Char"),
    String::from("String")]
);
pub fn Program_Analyzer(Program_val :Program){
    for line in Program_val{
        Stmt_Amalyzer(line);
    }
}pub fn Stmt_Amalyzer(Statement :Stmt){
    match Statement{
        Stmt::Assign(target,op,value)=>{
            let mut targ_ref=Type_Management.get(target).expect("{target} is not defined.");
            if(targ_ref=="null")targ_ref;//=型
            else //型が一致するかの確認
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
}