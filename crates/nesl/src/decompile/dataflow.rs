//! Data Flow Analysis and Variable Propagation
//!
//! This module implements classic compiler optimization techniques:
//! - Reaching definitions
//! - Use-def chains
//! - Constant propagation
//! - Copy propagation
//! - Dead code elimination

use std::collections::{HashMap, HashSet};
use super::register_elimination::{Statement, Variable, Expression, BinaryOp};

/// Data flow analyzer
pub struct DataFlowAnalyzer {
    statements: Vec<Statement>,
    // For each statement index, track reaching definitions
    reaching_defs: HashMap<usize, HashSet<usize>>,
    // Use-def chains: for each use, which statement defined it
    use_def: HashMap<(usize, Variable), usize>,
}

impl DataFlowAnalyzer {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self {
            statements,
            reaching_defs: HashMap::new(),
            use_def: HashMap::new(),
        }
    }

    /// Run data flow analysis
    pub fn analyze(&mut self) {
        self.compute_reaching_definitions();
        self.build_use_def_chains();
    }

    /// Compute reaching definitions using iterative data flow analysis
    fn compute_reaching_definitions(&mut self) {
        let n = self.statements.len();

        // Gen and Kill sets for each statement
        let mut gen: Vec<HashSet<usize>> = vec![HashSet::new(); n];
        let mut kill: Vec<HashSet<Variable>> = vec![HashSet::new(); n];

        // Compute Gen and Kill sets
        for (i, stmt) in self.statements.iter().enumerate() {
            match stmt {
                Statement::Assign { dest, .. } => {
                    gen[i].insert(i);
                    kill[i].insert(dest.clone());
                }
                Statement::Store { .. } => {
                    gen[i].insert(i);
                }
                _ => {}
            }
        }

        // Iterative fixed-point computation
        let mut in_sets: Vec<HashSet<usize>> = vec![HashSet::new(); n];
        let mut out_sets: Vec<HashSet<usize>> = vec![HashSet::new(); n];

        let mut changed = true;
        while changed {
            changed = false;

            for i in 0..n {
                // IN[i] = Union of OUT[predecessors]
                let mut new_in = HashSet::new();
                if i > 0 {
                    new_in = out_sets[i - 1].clone();
                }

                // OUT[i] = Gen[i] U (IN[i] - Kill[i])
                let mut new_out = gen[i].clone();
                for &def_idx in &in_sets[i] {
                    let killed = self.is_killed(def_idx, &kill[i]);
                    if !killed {
                        new_out.insert(def_idx);
                    }
                }

                if new_in != in_sets[i] || new_out != out_sets[i] {
                    changed = true;
                    in_sets[i] = new_in;
                    out_sets[i] = new_out;
                }
            }
        }

        // Store results
        for i in 0..n {
            self.reaching_defs.insert(i, in_sets[i].clone());
        }
    }

    fn is_killed(&self, def_idx: usize, kill_set: &HashSet<Variable>) -> bool {
        if let Some(Statement::Assign { dest, .. }) = self.statements.get(def_idx) {
            kill_set.contains(dest)
        } else {
            false
        }
    }

    /// Build use-def chains
    fn build_use_def_chains(&mut self) {
        for (i, stmt) in self.statements.iter().enumerate() {
            let uses = self.get_uses(stmt);

            for used_var in uses {
                // Find the definition that reaches this use
                if let Some(defs) = self.reaching_defs.get(&i) {
                    for &def_idx in defs {
                        if let Some(Statement::Assign { dest, .. }) = self.statements.get(def_idx) {
                            if dest == &used_var {
                                self.use_def.insert((i, used_var.clone()), def_idx);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    fn get_uses(&self, stmt: &Statement) -> Vec<Variable> {
        let mut uses = Vec::new();

        match stmt {
            Statement::Assign { source, .. } => {
                self.collect_expr_vars(source, &mut uses);
            }
            Statement::Store { value, .. } => {
                self.collect_expr_vars(value, &mut uses);
            }
            _ => {}
        }

        uses
    }

    fn collect_expr_vars(&self, expr: &Expression, vars: &mut Vec<Variable>) {
        match expr {
            Expression::Variable(var) => vars.push(var.clone()),
            Expression::BinaryOp { left, right, .. } => {
                self.collect_expr_vars(left, vars);
                self.collect_expr_vars(right, vars);
            }
            _ => {}
        }
    }

    /// Perform constant propagation
    pub fn constant_propagation(&mut self) -> Vec<Statement> {
        let mut constants: HashMap<Variable, u8> = HashMap::new();
        let mut new_statements = Vec::new();

        for stmt in &self.statements {
            match stmt {
                Statement::Assign { dest, source } => {
                    let new_source = self.propagate_in_expr(source, &constants);

                    // Check if result is constant
                    if let Expression::Constant(val) = new_source {
                        constants.insert(dest.clone(), val);
                    } else {
                        constants.remove(dest);
                    }

                    new_statements.push(Statement::Assign {
                        dest: dest.clone(),
                        source: new_source,
                    });
                }

                Statement::Store { addr, value } => {
                    let new_value = self.propagate_in_expr(value, &constants);
                    new_statements.push(Statement::Store {
                        addr: *addr,
                        value: new_value,
                    });
                }

                _ => {
                    new_statements.push(stmt.clone());
                }
            }
        }

        new_statements
    }

    fn propagate_in_expr(&self, expr: &Expression, constants: &HashMap<Variable, u8>) -> Expression {
        match expr {
            Expression::Variable(var) => {
                if let Some(&val) = constants.get(var) {
                    Expression::Constant(val)
                } else {
                    expr.clone()
                }
            }

            Expression::BinaryOp { op, left, right } => {
                let new_left = self.propagate_in_expr(left, constants);
                let new_right = self.propagate_in_expr(right, constants);

                // Constant folding
                if let (Expression::Constant(l), Expression::Constant(r)) = (&new_left, &new_right) {
                    let result = match op {
                        BinaryOp::Add => l.wrapping_add(*r),
                        BinaryOp::Sub => l.wrapping_sub(*r),
                        BinaryOp::And => l & r,
                        BinaryOp::Or => l | r,
                        BinaryOp::Xor => l ^ r,
                    };
                    Expression::Constant(result)
                } else {
                    Expression::BinaryOp {
                        op: op.clone(),
                        left: Box::new(new_left),
                        right: Box::new(new_right),
                    }
                }
            }

            _ => expr.clone(),
        }
    }

    /// Eliminate dead code
    pub fn eliminate_dead_code(&mut self) -> Vec<Statement> {
        let mut live: HashSet<usize> = HashSet::new();

        // Mark essential statements as live
        for (i, stmt) in self.statements.iter().enumerate() {
            match stmt {
                Statement::Store { .. } | Statement::Call { .. } |
                Statement::Return | Statement::Branch { .. } => {
                    live.insert(i);
                }
                _ => {}
            }
        }

        // Backward propagation of liveness
        let mut changed = true;
        while changed {
            changed = false;

            for i in (0..self.statements.len()).rev() {
                if live.contains(&i) {
                    // Mark definitions that reach this use as live
                    let uses = self.get_uses(&self.statements[i]);

                    for used_var in uses {
                        if let Some(&def_idx) = self.use_def.get(&(i, used_var)) {
                            if !live.contains(&def_idx) {
                                live.insert(def_idx);
                                changed = true;
                            }
                        }
                    }
                }
            }
        }

        // Keep only live statements
        self.statements.iter()
            .enumerate()
            .filter(|(i, _)| live.contains(i))
            .map(|(_, stmt)| stmt.clone())
            .collect()
    }
}

/// Variable propagation optimizer
pub struct VariablePropagator {
    copies: HashMap<Variable, Variable>,
}

impl VariablePropagator {
    pub fn new() -> Self {
        Self {
            copies: HashMap::new(),
        }
    }

    /// Perform copy propagation
    pub fn propagate_copies(&mut self, statements: &[Statement]) -> Vec<Statement> {
        let mut new_statements = Vec::new();

        for stmt in statements {
            match stmt {
                Statement::Assign { dest, source } => {
                    // Check for copy: x = y
                    if let Expression::Variable(src_var) = source {
                        self.copies.insert(dest.clone(), src_var.clone());
                        // Keep the assignment for now
                        new_statements.push(stmt.clone());
                    } else {
                        // Propagate copies in the expression
                        let new_source = self.propagate_in_expr(source);
                        new_statements.push(Statement::Assign {
                            dest: dest.clone(),
                            source: new_source,
                        });
                        self.copies.remove(dest);
                    }
                }

                Statement::Store { addr, value } => {
                    let new_value = self.propagate_in_expr(value);
                    new_statements.push(Statement::Store {
                        addr: *addr,
                        value: new_value,
                    });
                }

                _ => {
                    new_statements.push(stmt.clone());
                }
            }
        }

        new_statements
    }

    fn propagate_in_expr(&self, expr: &Expression) -> Expression {
        match expr {
            Expression::Variable(var) => {
                if let Some(replacement) = self.copies.get(var) {
                    Expression::Variable(replacement.clone())
                } else {
                    expr.clone()
                }
            }

            Expression::BinaryOp { op, left, right } => {
                Expression::BinaryOp {
                    op: op.clone(),
                    left: Box::new(self.propagate_in_expr(left)),
                    right: Box::new(self.propagate_in_expr(right)),
                }
            }

            _ => expr.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_propagation() {
        // t0 = 10
        // t1 = t0
        // t2 = t1 + 5
        // Should become: t2 = 15

        let statements = vec![
            Statement::Assign {
                dest: Variable::Temp(0),
                source: Expression::Constant(10),
            },
            Statement::Assign {
                dest: Variable::Temp(1),
                source: Expression::Variable(Variable::Temp(0)),
            },
            Statement::Assign {
                dest: Variable::Temp(2),
                source: Expression::BinaryOp {
                    op: BinaryOp::Add,
                    left: Box::new(Expression::Variable(Variable::Temp(1))),
                    right: Box::new(Expression::Constant(5)),
                },
            },
        ];

        let mut analyzer = DataFlowAnalyzer::new(statements);
        let optimized = analyzer.constant_propagation();

        // t2 should be assigned 15
        if let Statement::Assign { source, .. } = &optimized[2] {
            assert!(matches!(source, Expression::Constant(15)));
        }
    }
}
