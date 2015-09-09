/// Unittest for parsing module


use std::iter::Iterator;
use super::ast::*;
use super::token:: {TokenSpan, Lit};
use super::lex::Lexer;
use std::mem::swap;
use super::token::Token;
use super::Span;
use super::super::storage::SqlType;
use super::parser;
use std::collections::HashMap;

// ============================================================================
// Result::Ok unittest
// ============================================================================

#[test]
fn test_create_table_empty() {
    let mut p = parser::Parser::create("cReAtE table
        foo");

    assert_eq!(p.parse(), Ok(Query::DefStmt(DefStmt::Create(
        CreateStmt::Table(CreateTableStmt {tid: "foo".to_string(),
            cols: Vec::<ColumnInfo>::new()
        })))));
}

#[test]
fn test_create_table_content() {
    let mut p = parser::Parser::create(
        "create table foo (FirstName char(255), LastName char(255))");

    let vec = vec![ColumnInfo  {
            cid: "FirstName".to_string(),
            datatype: SqlType::Char(255),
            primary: false,
        },  ColumnInfo  {
            cid: "LastName".to_string(),
            datatype: SqlType::Char(255),
            primary: false,
        }
    ];

    assert_eq!(p.parse().unwrap(), Query::DefStmt(DefStmt::Create(
        CreateStmt::Table(CreateTableStmt {
            tid: "foo".to_string(), cols: vec }))))
}

#[test]
fn test_create_table_content_primary() {
    let mut p = parser::Parser::create(
        "create table foo (FirstName char(255), LastName char(255) primary key)");

    let vec = vec![ColumnInfo  {
            cid: "FirstName".to_string(),
            datatype: SqlType::Char(255),
            primary: false,
        },  ColumnInfo  {
            cid: "LastName".to_string(),
            datatype: SqlType::Char(255),
            primary: true,
        }
    ];


    assert_eq!(p.parse().unwrap(), Query::DefStmt(DefStmt::Create(
        CreateStmt::Table(CreateTableStmt {
            tid: "foo".to_string(), cols: vec }))))
}

#[test]
fn test_create_database() {
    let mut p = parser::Parser::create("create database Database");

    assert_eq!(p.parse().unwrap(), Query::DefStmt(DefStmt::Create(
        CreateStmt::Database("Database".to_string()))));
}

#[test]
fn test_alter_table_add_column() {
    let mut p = parser::Parser::create("alter table foo add bar int");

    assert_eq!(p.parse().unwrap(), Query::DefStmt(DefStmt::Alter(
        AltStmt::Table(AlterTableStmt {tid: "foo".to_string(),
        op: AlterOp::Add(ColumnInfo {
            cid: "bar".to_string(),
            datatype: SqlType::Int,
            primary: false,
        })
    }))));
}

#[test]
fn test_alter_table_add_column_primary() {
    let mut p = parser::Parser::create("alter table foo add bar int primary key");

    assert_eq!(p.parse().unwrap(), Query::DefStmt(DefStmt::Alter(
        AltStmt::Table(AlterTableStmt {tid: "foo".to_string(),
        op: AlterOp::Add(ColumnInfo {
            cid: "bar".to_string(),
            datatype: SqlType::Int,
            primary: true,
        })
    }))));
}

#[test]
fn test_alter_table_drop_column() {
    let mut p = parser::Parser::create("alter table foo drop column bar");

    assert_eq!(p.parse().unwrap(), Query::DefStmt(DefStmt::Alter(
        AltStmt::Table(AlterTableStmt {tid: "foo".to_string(),
        op: AlterOp::Drop("bar".to_string())
        })
    )));
}

#[test]
fn test_alter_table_modify() {
    let mut p = parser::Parser::create("alter table foo modify
        column bar bool");

    assert_eq!(p.parse().unwrap(), Query::DefStmt(DefStmt::Alter(
        AltStmt::Table(AlterTableStmt {tid: "foo".to_string(),
        op: AlterOp::Modify(ColumnInfo {
            cid: "bar".to_string(),
            datatype: SqlType::Bool,
            primary: false,
        })
    }))));
}

#[test]
fn test_drop_table() {
    let mut p = parser::Parser::create("drop table foo");

    assert_eq!(p.parse().unwrap(), Query::DefStmt(DefStmt::Drop(
        DropStmt::Table("foo".to_string()))));
}

#[test]
fn test_drop_database() {
    let mut p = parser::Parser::create("drop database foo");

    assert_eq!(p.parse().unwrap(), Query::DefStmt(DefStmt::Drop(
        DropStmt::Database("foo".to_string()))));
}

#[test]
fn test_use_database() {
    let mut p = parser::Parser::create("use database foo");

    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Use(UseStmt::Database("foo".to_string()))));
}

#[test]
fn test_describe_column() {
    let mut p = parser::Parser::create("describe foo");

    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Describe("foo".to_string())));
}

#[test]
fn test_insert_1() {
    let mut p = parser::Parser::create("insert into foo values
        ('peter', 'pan', 3) ");

    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Insert(InsertStmt {
            tid: "foo".to_string(),
            col: Vec::<String>::new(),
            val: vec![Lit::Str("peter".to_string()),
                Lit::Str("pan".to_string()),
                Lit::Int(3)],
    })));
}

#[test]
fn test_insert_2() {
    let mut p = parser::Parser::create("insert into foo () values
        ('peter', 'pan', 4)");

    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Insert(InsertStmt {
            tid: "foo".to_string(),
            col: Vec::<String>::new(),
            val: vec![Lit::Str("peter".to_string()),
                Lit::Str("pan".to_string()),
                Lit::Int(4)],
    })));
}

#[test]
fn test_insert_3() {
    let mut p = parser::Parser::create("insert into foo (eins, zwei, drei)
        values ('peter', 'pan', 5)");

    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Insert(InsertStmt {
            tid: "foo".to_string(),
            col: vec!["eins".to_string(), "zwei".to_string(), "drei".to_string()],
            val: vec![Lit::Str("peter".to_string()),
                Lit::Str("pan".to_string()),
                Lit::Int(5)],
    })));
}

#[test]
fn test_delete_row() {
    let mut p = parser::Parser::create("delete from foo where name = 'peter'");

    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Delete(DeleteStmt {
            tid: "foo".to_string(),
            alias: HashMap::new(),
            cond: Some(Conditions::Leaf(Condition {
                aliascol: None,
                col: "name".to_string(),
                op: CompType::Equ,
                aliasrhs: None,
                rhs: CondType::Literal(Lit::Str("peter".to_string())),
            })),
    })));
}
#[test]
fn test_delete_full_with_alias() {
    let mut p = parser::Parser::create("delete from foo bar");
    let mut aliashm = HashMap::new();
    aliashm.insert("bar".to_string(), "foo".to_string());


    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Delete(DeleteStmt {
            tid: "foo".to_string(),
            alias: aliashm,
            cond: None,
    })));
}
#[test]
fn test_mult_where_blocks() {
    let mut p = parser::Parser::create("delete from foo where lname = 'peng' or
        fname = 'peter' and lname = 'pan'");

    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Delete(DeleteStmt {
            tid: "foo".to_string(),
            alias: HashMap::new(),
            cond: Some(
                    Conditions::Or(Box::new(
                        Conditions::Leaf(Condition {
                            aliascol: None,
                            col: "lname".to_string(),
                            op: CompType::Equ,
                            aliasrhs: None,
                            rhs: CondType::Literal(Lit::Str("peng".to_string())),
                            }
                        )
                    ), Box::new(
                        Conditions::And(Box::new(
                            Conditions::Leaf(Condition {
                                aliascol: None,
                                col: "fname".to_string(),
                                op: CompType::Equ,
                                aliasrhs: None,
                                rhs: CondType::Literal(Lit::Str("peter".to_string())),
                                }
                            )), Box::new(Conditions::Leaf(Condition{
                                aliascol: None,
                                col: "lname".to_string(),
                                op: CompType::Equ,
                                aliasrhs: None,
                                rhs: CondType::Literal(Lit::Str("pan".to_string())),
                                }))
                            )
                        )
                    )
                )
            }
        ))
    );
}

// ============================================================================
// Result::Err unittest
// ============================================================================



#[test]
fn test_create_table_error1() {
    let mut p = parser::Parser::create("cReAtE table");
    let sol = parser::ParseError::UnexpectedEoq;
    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn test_create_keyword1() {
    let mut p = parser::Parser::create("   table create");
    let sol = parser::ParseError::WrongKeyword(Span {
        lo : 5,
        hi : 10,
    });

    assert_eq!(p.parse(), Err(sol));
}

//Missing parentheses in front
#[test]
fn test_create_missing_p_front() {
    let mut p = parser::Parser::create("create table Studenten )");
    let sol = parser::ParseError::WrongToken(Span {
        lo : 24,
        hi : 24,
    });

    assert_eq!(p.parse(), Err(sol));
}
