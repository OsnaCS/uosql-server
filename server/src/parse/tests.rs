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
    let mut p = parser::Parser::create("cReAtE table \n foo");

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
            auto_increment: false,
            not_null: false,
            comment: None,
        }, ColumnInfo {
            cid: "LastName".to_string(),
            datatype: SqlType::Char(255),
            primary: false,
            auto_increment: false,
            not_null: false,
            comment: None,
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
            auto_increment: false,
            not_null: false,
            comment: None,
        }, ColumnInfo {
            cid: "LastName".to_string(),
            datatype: SqlType::Char(255),
            primary: true,
            auto_increment: false,
            not_null: false,
            comment: None,
        }
    ];


    assert_eq!(p.parse().unwrap(), Query::DefStmt(DefStmt::Create(
        CreateStmt::Table(CreateTableStmt {
            tid: "foo".to_string(), cols: vec }))))
}

#[test]
fn test_create_table_full() {
    let mut p = parser::Parser::create(
        "create table foo (FirstName char(255) not null
            auto_increment comment 'TEST' primary key)");

    let vec = vec![ColumnInfo {
            cid: "FirstName".to_string(),
            datatype: SqlType::Char(255),
            primary: true,
            auto_increment: true,
            not_null: true,
            comment: Some("TEST".to_string()),
        }
    ];

    assert_eq!(p.parse().unwrap(), Query::DefStmt(DefStmt::Create(
        CreateStmt::Table(CreateTableStmt {
            tid: "foo".to_string(), cols: vec }))))
}

#[test]
fn test_create_database() {
    let mut p = parser::Parser::create("create database foo");

    assert_eq!(p.parse().unwrap(), Query::DefStmt(DefStmt::Create(
        CreateStmt::Database("foo".to_string()))));
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
            auto_increment: false,
            not_null: false,
            comment: None,
        })
    }))));
}

#[test]
fn test_alter_table_add_column_primary() {
    let mut p = parser::Parser::create("alter table foo add bar inT primary key");

    assert_eq!(p.parse().unwrap(), Query::DefStmt(DefStmt::Alter(
        AltStmt::Table(AlterTableStmt {tid: "foo".to_string(),
        op: AlterOp::Add(ColumnInfo {
            cid: "bar".to_string(),
            datatype: SqlType::Int,
            primary: true,
            auto_increment: false,
            not_null: false,
            comment: None,
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
            auto_increment: false,
            not_null: false,
            comment: None,
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
fn test_drop_view() {
    let mut p = parser::Parser::create("drop view foo");

    assert_eq!(p.parse().unwrap(), Query::DefStmt(DefStmt::Drop(
        DropStmt::View("foo".to_string()))));
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
            val: vec![Lit::String("peter".to_string()),
                Lit::String("pan".to_string()),
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
            val: vec![Lit::String("peter".to_string()),
                Lit::String("pan".to_string()),
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
            val: vec![Lit::String("peter".to_string()),
                Lit::String("pan".to_string()),
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
                rhs: CondType::Literal(Lit::String("peter".to_string())),
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
            order: Vec::new(),
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
            order: Vec::new(),
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
            order: Vec::new(),
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
            order: Vec::new(),
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
            order: Vec::new(),
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
                        rhs: CondType::Literal(Lit::String("Eugene".to_string())),
                    })),
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "lname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::String("peng".to_string())),
                    })))),
                Box::new(Conditions::And(
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "fname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::String("peter".to_string())),
                    })),
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "lname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::String("pan".to_string())),
                    }))
                ))
            )),
            spec_op: None,
            order: Vec::new(),
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
            order: Vec::new(),
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
                        rhs: CondType::Literal(Lit::String("Eugene".to_string())),
                    })),
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "lname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::String("peng".to_string())),
                    })))),
                Box::new(Conditions::And(
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "fname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::String("peter".to_string())),
                    })),
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "lname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::String("pan".to_string())),
                    }))
                ))
            )),
            spec_op: None,
            order: Vec::new(),
            limit: Some(Limit {
                count: Some(3),
                offset: Some(30),
            }),
    })));
}

#[test]
fn test_select_complete_1() {
    let mut p = parser::Parser::create("
        select bar_1.column_1 as X, bar_2.column_2 as Y from foo bar_1, foo_2 bar_2
        where bar_1.fname = 'Eugene' and bar_1.lname = 'peng'
        or bar_2.fname = bar_1.fname and bar_2.lname = bar_1.lname limit 30,3");
    let mut aliashm = HashMap::new();
    aliashm.insert("bar_2".to_string(), "foo_2".to_string());
    aliashm.insert("bar_1".to_string(), "foo".to_string());
    let selected_tables = vec!["foo".to_string(), "foo_2".to_string()];

    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Select(SelectStmt {
            target: vec![Target {
                alias: Some("bar_1".to_string()),
                col: Col::Specified("column_1".to_string()),
                rename: Some("X".to_string()),
            }, Target {
                alias: Some("bar_2".to_string()),
                col: Col::Specified("column_2".to_string()),
                rename: Some("Y".to_string()),
            }],
            tid: selected_tables,
            alias: aliashm,
            cond: Some(Conditions::Or(
                Box::new(Conditions::And(
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: Some("bar_1".to_string()),
                        col: "fname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::String("Eugene".to_string())),
                    })),
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: Some("bar_1".to_string()),
                        col: "lname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::String("peng".to_string())),
                    })))),
                Box::new(Conditions::And(
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: Some("bar_2".to_string()),
                        col: "fname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: Some("bar_1".to_string()),
                        rhs: CondType::Word("fname".to_string()),
                    })),
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: Some("bar_2".to_string()),
                        col: "lname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: Some("bar_1".to_string()),
                        rhs: CondType::Word("lname".to_string()),
                    }))
                ))
            )),
            spec_op: None,
            order: Vec::new(),
            limit: Some(Limit {
                count: Some(3),
                offset: Some(30),
            }),
    })));
}

#[test]
fn test_select_complete_2_with_order_by() {
    let mut p = parser::Parser::create("
        select bar_1.column_1 as X, bar_2.column_2 as Y from foo bar_1, foo_2 bar_2
        where bar_1.fname = 'Eugene' and bar_1.lname = 'peng'
        or bar_2.fname = bar_1.fname and bar_2.lname = bar_1.lname
        order by bar_1.X limit 30,3");
    let mut aliashm = HashMap::new();
    aliashm.insert("bar_2".to_string(), "foo_2".to_string());
    aliashm.insert("bar_1".to_string(), "foo".to_string());
    let selected_tables = vec!["foo".to_string(), "foo_2".to_string()];

    assert_eq!(p.parse().unwrap(), Query::ManipulationStmt(
        ManipulationStmt::Select(SelectStmt {
            target: vec![Target {
                alias: Some("bar_1".to_string()),
                col: Col::Specified("column_1".to_string()),
                rename: Some("X".to_string()),
            }, Target {
                alias: Some("bar_2".to_string()),
                col: Col::Specified("column_2".to_string()),
                rename: Some("Y".to_string()),
            }],
            tid: selected_tables,
            alias: aliashm,
            cond: Some(Conditions::Or(
                Box::new(Conditions::And(
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: Some("bar_1".to_string()),
                        col: "fname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::String("Eugene".to_string())),
                    })),
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: Some("bar_1".to_string()),
                        col: "lname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::String("peng".to_string())),
                    })))),
                Box::new(Conditions::And(
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: Some("bar_2".to_string()),
                        col: "fname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: Some("bar_1".to_string()),
                        rhs: CondType::Word("fname".to_string()),
                    })),
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: Some("bar_2".to_string()),
                        col: "lname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: Some("bar_1".to_string()),
                        rhs: CondType::Word("lname".to_string()),
                    }))
                ))
            )),
            spec_op: None,
            order: vec![Sort {
                alias: Some("bar_1".to_string()),
                col: "X".to_string(),
                order: Some(Order::Asc),
            }],
            limit: Some(Limit {
                count: Some(3),
                offset: Some(30),
            }),
    })));
}

#[test]
fn test_create_view_1() {
    let mut p = parser::Parser::create("create view foo as select * from bar");

    assert_eq!(p.parse().unwrap(), Query::DefStmt(DefStmt::Create(
        CreateStmt::View(CreateViewStmt {
            name: "foo".to_string(),
            opt: false,
            sel: SelectStmt {
                    target: vec![Target {
                        alias: None,
                        col: Col::Every,
                        rename: None,
                    }],
                    tid: vec!["bar".to_string()],
                    alias: HashMap::new(),
                    cond: None,
                    spec_op: None,
                    order: Vec::new(),
                    limit: None,
                },
            }
        )
    )));
}

#[test]
fn test_create_view_2() {
   let mut p = parser::Parser::create("create or replace view foo as select * from bar");

    assert_eq!(p.parse().unwrap(), Query::DefStmt(DefStmt::Create(
        CreateStmt::View(CreateViewStmt {
            name: "foo".to_string(),
            opt: true,
            sel: SelectStmt {
                    target: vec![Target {
                        alias: None,
                        col: Col::Every,
                        rename: None,
                    }],
                    tid: vec!["bar".to_string()],
                    alias: HashMap::new(),
                    cond: None,
                    spec_op: None,
                    order: Vec::new(),
                    limit: None,
                },
            }
        )
    )));
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
                rhs: CondType::Literal(Lit::String("pleb".to_string())),
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
                    rhs: CondType::Literal(Lit::String("peng".to_string())),
                    }
                )
            ), Box::new(Conditions::And(Box::new(
                    Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "fname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::String("peter".to_string())),
                        }
                    )), Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "lname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::String("pan".to_string())),
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
                        rhs: CondType::Literal(Lit::String("Eugene".to_string())),
                    })),
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "lname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::String("peng".to_string())),
                    })))),
                Box::new(Conditions::And(
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "fname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::String("peter".to_string())),
                    })),
                    Box::new(Conditions::Leaf(Condition {
                        aliascol: None,
                        col: "lname".to_string(),
                        op: CompType::Equ,
                        aliasrhs: None,
                        rhs: CondType::Literal(Lit::String("pan".to_string())),
                    }))
                ))
            ))
        }))
    );
}

/*#[test]
fn to_do() {
    let mut p = parser::Parser::create("
        select bar_1.column_1 as X, bar_2.column_2 as Y from foo bar_1, foo_2 bar_2
        where bar_1.fname = 'Eugene' and bar_1.lname = 'peng'
        or bar_2.fname = bar_1.fname and bar_2.lname = bar_1.lname limit 30,3");

    assert!(p.parse().is_ok());
}*/

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
        lo: 5,
        hi: 10,
    });

    assert_eq!(p.parse(), Err(sol));
}

//Missing parentheses in front
#[test]
fn err_create_wrong_token_1() {
    let mut p = parser::Parser::create("create table Studenten )");
    let sol = parser::ParseError::WrongToken(Span {
        lo: 24,
        hi: 24,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_create_wrong_token_2() {
    let mut p = parser::Parser::create("create table studenten (asd int(");
    let sol = parser::ParseError::WrongToken(Span {
        lo: 32,
        hi: 32,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_create_wrong_token_3() {
    let mut p = parser::Parser::create("create table studenten (asd asd)");
    let sol = parser::ParseError::NotADatatype(Span {
        lo: 30,
        hi: 32,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_create_missing_parenthesis() {
    let mut p = parser::Parser::create("create table studenten asd int)");
    let sol = parser::ParseError::WrongToken(Span {
        lo: 25,
        hi: 28,
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

#[test]
fn err_create_invalid_eoq_1() {
    let mut p = parser::Parser::create("create table studenten (asd int))");
    let sol = parser::ParseError::InvalidEoq;

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_create_invalid_eoq_2() {
    let mut p = parser::Parser::create("create database studenten(asd int,)");
    let sol = parser::ParseError::InvalidEoq;

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_describe() {
    let mut p = parser::Parser::create("describe ,");
    let sol = parser::ParseError::NotAWord(Span {
        lo: 10,
        hi: 10,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_describe_2() {
    let mut p = parser::Parser::create("describe table");
    let sol = parser::ParseError::ReservedKeyword(Span {
        lo: 11,
        hi: 14,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_alter_1() {
    let mut p = parser::Parser::create("alter table table add bar int");
    let sol = parser::ParseError::ReservedKeyword(Span {
        lo: 14,
        hi: 19,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_alter_2() {
    let mut p = parser::Parser::create("alter table foo add bar foo");
    let sol = parser::ParseError::NotADatatype(Span {
        lo: 26,
        hi: 27,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_alter_3() {
    let mut p = parser::Parser::create("alter table foo drop bar_1");
    let sol = parser::ParseError::NotAKeyword(Span {
        lo: 23,
        hi: 26,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_alter_5() {
    let mut p = parser::Parser::create("alter table foo add (bar int");
    let sol = parser::ParseError::NotAWord(Span {
        lo: 22,
        hi: 23,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_alter_6() {
    let mut p = parser::Parser::create("alter table foo drop column (");
    let sol = parser::ParseError::NotAWord(Span {
        lo: 29,
        hi: 29,
    });

    assert_eq!(p.parse(), Err(sol));
}


#[test]
fn err_alter_8() {
    let mut p = parser::Parser::create("alter table foo modify asd");
    let sol = parser::ParseError::NotAKeyword(Span {
        lo: 25,
        hi: 26,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_alter_9() {
    let mut p = parser::Parser::create("alter table foo modify column bar asd");
    let sol = parser::ParseError::NotADatatype(Span {
        lo: 36,
        hi: 37,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_alter_10() {
    let mut p = parser::Parser::create("alter table foo modify column bar bool )");
    let sol = parser::ParseError::InvalidEoq;

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_use_1() {
    let mut p = parser::Parser::create("use table foo");
    let sol = parser::ParseError::WrongKeyword(Span {
        lo: 6,
        hi: 11,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_use_2() {
    let mut p = parser::Parser::create("use database use");
    let sol = parser::ParseError::ReservedKeyword(Span {
        lo: 15,
        hi: 16,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_use_3() {
    let mut p = parser::Parser::create("use database 1");
    let sol = parser::ParseError::NotAWord(Span {
        lo: 14,
        hi: 14,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_drop_1() {
    let mut p = parser::Parser::create("drop foo");
    let sol = parser::ParseError::NotAKeyword(Span {
        lo: 7,
        hi: 8,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_drop_2() {
    let mut p = parser::Parser::create("drop table table");
    let sol = parser::ParseError::ReservedKeyword(Span {
        lo: 13,
        hi: 16,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_drop_3() {
    let mut p = parser::Parser::create("drop table ]");
    let sol = parser::ParseError::NotAWord(Span {
        lo: 12,
        hi: 12,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_insert_1() {
    let mut p = parser::Parser::create("insert a");
    let sol = parser::ParseError::NotAKeyword(Span {
        lo: 8,
        hi: 8,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_insert_2() {
    let mut p = parser::Parser::create("insert into into");
    let sol = parser::ParseError::ReservedKeyword(Span {
        lo: 14,
        hi: 16,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_insert_3() {
    let mut p = parser::Parser::create("insert into foo bar ('⊂(▀¯▀⊂)', 420, 'lel'");
    let sol = parser::ParseError::NotAKeyword(Span {
        lo: 18,
        hi: 21,
    });

    assert_eq!(p.parse(), Err(sol));
}

#[test]
fn err_insert_4() {
    let mut p = parser::Parser::create("insert into foo values ('⊂(▀¯▀⊂)', 420, 'lel'");
    let sol = parser::ParseError::UnexpectedEoq;

    assert_eq!(p.parse(), Err(sol));
}
