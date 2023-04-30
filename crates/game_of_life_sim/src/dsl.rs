#![allow(dead_code)]

/**
 * An expression in the domain specific language we use to describe cellular automata. Expressions
 * can perform arbitrary arithmetic and comparisons between constants, a boolean that indicates
 * whether the cell is currently alive, and the number of neighbors that a cell currently has.
 */
#[derive(Debug, Clone)]
pub enum Expr {
    U32(u32),
    Alive,
    Neighbors,
    Gt(Box<Expr>, Box<Expr>),
    Gte(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Lte(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Equal(Box<Expr>, Box<Expr>),
}

impl Expr {
    /**
     * This method converts an Expr to an equivalent wgsl code fragment. This is not a valid wgsl
     * program, just an expression in wgsl. When used by statements it can form a complete wgsl
     * program.
     */
    pub fn to_shader(&self) -> String {
        use Expr::*;

        match self {
            U32(val) => format!("{}u", val),
            Alive => "is_alive".to_string(),
            Neighbors => "num_neighbors".to_string(),
            Gt(lhs, rhs) => format!(
                "u32(({}) > ({}))",
                Self::to_shader(lhs),
                Self::to_shader(rhs)
            ),
            Gte(lhs, rhs) => format!(
                "u32(({}) >= ({}))",
                Self::to_shader(lhs),
                Self::to_shader(rhs)
            ),
            Lt(lhs, rhs) => format!(
                "u32(({}) < ({}))",
                Self::to_shader(lhs),
                Self::to_shader(rhs)
            ),
            Lte(lhs, rhs) => format!(
                "u32(({}) <= ({}))",
                Self::to_shader(lhs),
                Self::to_shader(rhs)
            ),
            And(lhs, rhs) => format!("(({}) & ({}))", Self::to_shader(lhs), Self::to_shader(rhs)),
            Or(lhs, rhs) => format!("(({}) | ({}))", Self::to_shader(lhs), Self::to_shader(rhs)),
            Equal(lhs, rhs) => format!(
                "u32(({}) == ({}))",
                Self::to_shader(lhs),
                Self::to_shader(rhs)
            ),
        }
    }
}

/**
 * A statement in the domain specific language we use to describe cellular automata. Statements can
 * conditionally branch on expressions or set whether the current cell is alive or dead to the
 * result of an expression. Through statements we can describe complex rules to form cellular automata.
 */
#[derive(Debug, Clone)]
pub enum Statement {
    Void,
    SetResult(Expr),
    IfThenElse {
        condition: Expr,
        if_true_then: Box<Statement>,
        if_false_then: Box<Statement>,
    },
}

impl Statement {
    /**
     * Turn a statement into a valid wgsl statement that can be injected into our placeholder
     * compute shader and executed on the GPU.
     */
    pub fn to_shader(&self) -> String {
        use Statement::*;

        match self {
            Void => String::new(),
            SetResult(expr) => format!("result = {};", expr.to_shader()),
            IfThenElse {
                condition,
                if_true_then,
                if_false_then,
            } => format!(
                "if ({}) {{ {} }} else {{ {} }}",
                condition.to_shader(),
                if_true_then.to_shader(),
                if_false_then.to_shader()
            ),
        }
    }
}

pub mod exprs {
    use super::Expr;
    use super::Expr::*;

    pub fn const_u32(value: u32) -> Expr {
        U32(value)
    }

    pub fn alive() -> Expr {
        Alive
    }

    pub fn neighbors() -> Expr {
        Neighbors
    }

    pub fn gt(lhs: Expr, rhs: Expr) -> Expr {
        Gt(Box::new(lhs), Box::new(rhs))
    }

    pub fn gte(lhs: Expr, rhs: Expr) -> Expr {
        Gte(Box::new(lhs), Box::new(rhs))
    }

    pub fn lt(lhs: Expr, rhs: Expr) -> Expr {
        Lt(Box::new(lhs), Box::new(rhs))
    }

    pub fn lte(lhs: Expr, rhs: Expr) -> Expr {
        Lte(Box::new(lhs), Box::new(rhs))
    }

    pub fn and(lhs: Expr, rhs: Expr) -> Expr {
        And(Box::new(lhs), Box::new(rhs))
    }

    pub fn or(lhs: Expr, rhs: Expr) -> Expr {
        Or(Box::new(lhs), Box::new(rhs))
    }

    pub fn equal(lhs: Expr, rhs: Expr) -> Expr {
        Equal(Box::new(lhs), Box::new(rhs))
    }
}

pub mod statements {
    use super::Statement::*;
    use super::{Expr, Statement};

    pub fn void() -> Statement {
        Void
    }

    pub fn set_result(expr: Expr) -> Statement {
        SetResult(expr)
    }

    pub fn if_then_else(
        condition: Expr,
        if_true_then: Statement,
        if_false_then: Statement,
    ) -> Statement {
        IfThenElse {
            condition,
            if_true_then: Box::new(if_true_then),
            if_false_then: Box::new(if_false_then),
        }
    }
}

pub mod rulesets {
    use super::{exprs::*, statements::*, Statement};

    /**
     * An implementation of conways game of life in
     * our domain specific language.
     */
    pub fn conways_game_of_life() -> Statement {
        if_then_else(
            alive(),
            set_result(or(
                equal(neighbors(), const_u32(2)),
                equal(neighbors(), const_u32(3)),
            )),
            set_result(equal(neighbors(), const_u32(3))),
        )
    }
}
