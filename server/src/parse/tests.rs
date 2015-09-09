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

    let vec = vec![ColumnInfo {
            cid: "FirstName".to_string(),
            datatype: SqlType::Char(255),
            primary: false,
        }, ColumnInfo {
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

    let vec = vec![ColumnInfo {
            cid: "FirstName".to_string(),
            datatype: SqlType::Char(255),
            primary: false,
        }, ColumnInfo {
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
fn test_delete_full_with_table_alias() {
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
fn test_select_full_with_table_alias() {
    let mut p = parser::Parser::create("select * from foo bar");
    let mut aliashm = HashMap::new();
    aliashm.insert("bar".to_string(), "foo".to_string());
    let selected_tables = vec!["foo".to_string()];

    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Select(SelectStmt {
            target: vec![Target {
                alias: None,
                col: Col::Every,
                rename: None,
            }],
            tid: selected_tables,
            alias: aliashm,
            cond: None,
            spec_op: None,
            limit: None,
    })));
}

#[test]
fn test_select_specific_column() {
    let mut p = parser::Parser::create("select bar from foo bar");
    let mut aliashm = HashMap::new();
    aliashm.insert("bar".to_string(), "foo".to_string());
    let selected_tables = vec!["foo".to_string()];

    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Select(SelectStmt {
            target: vec![Target {
                alias: None,
                col: Col::Specified("bar".to_string()),
                rename: None,
            }],
            tid: selected_tables,
            alias: aliashm,
            cond: None,
            spec_op: None,
            limit: None,
    })));
}

#[test]
fn test_select_specific_columns() {
    let mut p = parser::Parser::create("select bar_1, bar_2 from foo bar");
    let mut aliashm = HashMap::new();
    aliashm.insert("bar".to_string(), "foo".to_string());
    let selected_tables = vec!["foo".to_string()];

    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Select(SelectStmt {
            target: vec![Target {
                alias: None,
                col: Col::Specified("bar_1".to_string()),
                rename: None,
            }, Target {
                alias:None,
                col:Col::Specified("bar_2".to_string()),
                rename: None,
            }],
            tid: selected_tables,
            alias: aliashm,
            cond: None,
            spec_op: None,
            limit: None,
    })));
}

#[test]
fn test_select_specific_columns_alias() {
    let mut p = parser::Parser::create("select bar_1 as a2, bar_2 as a1 from foo bar");
    let mut aliashm = HashMap::new();
    aliashm.insert("bar".to_string(), "foo".to_string());
    let selected_tables = vec!["foo".to_string()];

    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Select(SelectStmt {
            target: vec![Target {
                alias: None,
                col: Col::Specified("bar_1".to_string()),
                rename: Some("a2".to_string()),
            }, Target {
                alias:None,
                col:Col::Specified("bar_2".to_string()),
                rename: Some("a1".to_string()),
            }],
            tid: selected_tables,
            alias: aliashm,
            cond: None,
            spec_op: None,
            limit: None,
    })));
}

#[test]
fn test_select_specific_columns_alias_dot() {
    let mut p = parser::Parser::create("select a.bar_1, b.bar_2 from foo bar");
    let mut aliashm = HashMap::new();
    aliashm.insert("bar".to_string(), "foo".to_string());
    let selected_tables = vec!["foo".to_string()];

    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Select(SelectStmt {
            target: vec![Target {
                alias: Some("a".to_string()),
                col: Col::Specified("bar_1".to_string()),
                rename: None,
            }, Target {
                alias: Some("b".to_string()),
                col:Col::Specified("bar_2".to_string()),
                rename: None,
            }],
            tid: selected_tables,
            alias: aliashm,
            cond: None,
            spec_op: None,
            limit: None,
    })));
}

#[test]
fn test_select_full_where_clause() {
    let mut p = parser::Parser::create("select * from foo bar where
        fname = 'Eugene' and lname = 'peng' or
        fname = 'peter' and lname = 'pan'");
    let mut aliashm = HashMap::new();
    aliashm.insert("bar".to_string(), "foo".to_string());
    let selected_tables = vec!["foo".to_string()];

    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Select(SelectStmt {
            target: vec![Target {
                alias: None,
                col: Col::Every,
                rename: None,
            }],
            tid: selected_tables,
            alias: aliashm,
            cond: Some(Conditions::Or(
                Box::new(Conditions::And(
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "fname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::Str("Eugene".to_string())),
                    })),
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "lname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::Str("peng".to_string())),
                    })))),
                Box::new(Conditions::And(
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "fname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::Str("peter".to_string())),
                    })),
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "lname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::Str("pan".to_string())),
                    }))
                ))
            )),
            spec_op: None,
            limit: None,
    })));
}

#[test] //to do: fix keyword limit as to prevent usage as alias for foo
fn test_select_full_no_where_limit() {
    let mut p = parser::Parser::create("select * from foo limit 30,3");

    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Select(SelectStmt {
            target: vec![Target {
                alias: None,
                col: Col::Every,
                rename: None,
            }],
            tid: vec!["foo".to_string()],
            alias: HashMap::new(),
            cond: None,
            spec_op: None,
            limit: Some(Limit {
                count: Some(3),
                offset: Some(30),
            }),
    })));
}

#[test]
fn test_select_full_where_clause_limit() {
    let mut p = parser::Parser::create("select * from foo bar where
        fname = 'Eugene' and lname = 'peng' or
        fname = 'peter' and lname = 'pan' limit 30,3");
    let mut aliashm = HashMap::new();
    aliashm.insert("bar".to_string(), "foo".to_string());
    let selected_tables = vec!["foo".to_string()];

    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Select(SelectStmt {
            target: vec![Target {
                alias: None,
                col: Col::Every,
                rename: None,
            }],
            tid: selected_tables,
            alias: aliashm,
            cond: Some(Conditions::Or(
                Box::new(Conditions::And(
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "fname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::Str("Eugene".to_string())),
                    })),
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "lname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::Str("peng".to_string())),
                    })))),
                Box::new(Conditions::And(
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "fname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::Str("peter".to_string())),
                    })),
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "lname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::Str("pan".to_string())),
                    }))
                ))
            )),
            spec_op: None,
            limit: Some(Limit {
                count: Some(3),
                offset: Some(30),
            }),
    })));
}

#[test]
fn test_update_full_with_table_alias() {
    let mut p = parser::Parser::create("update foo bar set bar_1 = 1 where bar.bar_2 > 'pleb'");
    let mut aliashm = HashMap::new();
    aliashm.insert("bar".to_string(), "foo".to_string());
    let set_vec = vec![Condition {
        aliascol: None,
        col: "bar_1".to_string(),
        op: CompType::Equ,
        aliasrhs: None,
        rhs: CondType::Literal(Lit::Int(1)),
    }];

    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Update(UpdateStmt {
            tid: "foo".to_string(),
            alias: aliashm,
            set: set_vec,
            conds: Some(Conditions::Leaf(Condition {
                aliascol: Some("bar".to_string()),
                col: "bar_2".to_string(),
                op: CompType::GThan,
                aliasrhs: None,
                rhs: CondType::Literal(Lit::Str("pleb".to_string())),
                })
            )
    })));
}

#[test]
fn test_mult_where_blocks_3_param() {
    let mut p = parser::Parser::create("delete from foo where lname = 'peng' or
        fname = 'peter' and lname = 'pan'");

    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Delete(DeleteStmt {
            tid: "foo".to_string(),
            alias: HashMap::new(),
            cond: Some(Conditions::Or(Box::new(
                Conditions::Leaf(Condition {
                    aliascol: None,
                    col: "lname".to_string(),
                    op: CompType::Equ,
                    aliasrhs: None,
                    rhs: CondType::Literal(Lit::Str("peng".to_string())),
                    }
                )
            ), Box::new(Conditions::And(Box::new(
                    Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "fname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::Str("peter".to_string())),
                        }
                    )), Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "lname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::Str("pan".to_string())),
                        }))
                    )
                )
            ))
        }))
    );
}

#[test]
fn test_mult_where_blocks_priority_4_param() {
    let mut p = parser::Parser::create("delete from foo where
        fname = 'Eugene' and lname = 'peng' or
        fname = 'peter' and lname = 'pan'");

    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Delete(DeleteStmt {
            tid: "foo".to_string(),
            alias: HashMap::new(),
            cond: Some(Conditions::Or(
                Box::new(Conditions::And(
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "fname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::Str("Eugene".to_string())),
                    })),
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "lname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::Str("peng".to_string())),
                    })))),
                Box::new(Conditions::And(
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "fname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::Str("peter".to_string())),
                    })),
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "lname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::Str("pan".to_string())),
                    }))
                ))
            ))
        }))
    );
}

#[test]
fn to_do() {
    let mut p = parser::Parser::create("select * from foo limit 30,3");

    assert!(p.parse().is_ok());
}

// ============================================================================
// Result::Err unittest
// ============================================================================

#[test]
fn err_create_table_error1() {
    let mut p = parser::Parser::create("cReAtE table");
    let sol = parser::ParseError::UnexpectedEoq;
    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_create_keyword1() {
    let mut p = parser::Parser::create("   table create");
    let sol = parser::ParseError::WrongKeyword(Span {
        lo : 5,
        hi : 10,
    });

    assert_eq!(p.parse(), Err(sol));
}

//Missing parentheses in front
#[test]
fn err_create_missing_p_front() {
    let mut p = parser::Parser::create("create table Studenten )");
    let sol = parser::ParseError::WrongToken(Span {
        lo : 24,
        hi : 24,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_create_not_a_keyword_1() {
    let mut p = parser::Parser::create("hallo table studenten");
    let sol = parser::ParseError::NotAKeyword(Span {
        lo: 2,
        hi: 7,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_create_not_a_keyword_2() {
    let mut p = parser::Parser::create("create asd Studenten");
    let sol = parser::ParseError::NotAKeyword(Span {
        lo: 9,
        hi: 12,
    });

    assert_eq!(p.parse(), Err(sol));
}
