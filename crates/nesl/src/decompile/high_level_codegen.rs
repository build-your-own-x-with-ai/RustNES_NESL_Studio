//! Phase 4: High-Level Statement Generation
//!
//! Converts optimized SSA form to high-level structured code:
//! - Structured control flow (if/while/for)
//! - Type-aware expressions
//! - Function calls with parameters
//! - Clean variable names

use std::collections::{HashMap, HashSet};
use super::register_elimination::{Statement, Variable, Expression, Condition, BinaryOp};
use super::control_flow::{ControlFlow, ControlFlowGraph, BasicBlock};

/// High-level code generator
pub struct HighLevelCodeGen {
    var_names: HashMap<Variable, String>,
    next_var_id: usize,
}

impl HighLevelCodeGen {
    pub fn new() -> Self {
        Self {
            var_names: HashMap::new(),
            next_var_id: 0,
        }
    }

    /// Generate high-level code from SSA statements
    pub fn generate(&mut self, statements: &[Statement]) -> String {
        // First, analyze and restructure control flow
        let structured = self.restructure_control_flow(statements);

        // Then generate clean code
        self.generate_code(&structured, 0)
    }

    /// Restructure control flow from flat statements
    fn restructure_control_flow(&mut self, statements: &[Statement]) -> ControlFlow {
        // Identify basic blocks
        let blocks = self.identify_blocks(statements);

        // Detect patterns
        self.detect_patterns(&blocks)
    }

    fn identify_blocks(&self, statements: &[Statement]) -> Vec<Block> {
        let mut blocks = Vec::new();
        let mut current = Vec::new();
        let mut start_idx = 0;

        for (i, stmt) in statements.iter().enumerate() {
            current.push(stmt.clone());

            // End of block?
            match stmt {
                Statement::Branch { .. } | Statement::Return => {
                    blocks.push(Block {
                        start: start_idx,
                        end: i,
                        statements: current.clone(),
                    });
                    current.clear();
                    start_idx = i + 1;
                }
                _ => {}
            }
        }

        // Last block
        if !current.is_empty() {
            blocks.push(Block {
                start: start_idx,
                end: statements.len() - 1,
                statements: current,
            });
        }

        blocks
    }

    fn detect_patterns(&mut self, blocks: &[Block]) -> ControlFlow {
        if blocks.is_empty() {
            return ControlFlow::Sequence(vec![]);
        }

        // Try to detect common patterns
        if let Some(cf) = self.try_if_else(blocks) {
            return cf;
        }

        if let Some(cf) = self.try_while_loop(blocks) {
            return cf;
        }

        // Fallback: sequential
        let mut seq = Vec::new();
        for block in blocks {
            seq.push(self.block_to_control_flow(block));
        }

        ControlFlow::Sequence(seq)
    }

    fn try_if_else(&mut self, blocks: &[Block]) -> Option<ControlFlow> {
        if blocks.len() < 2 {
            return None;
        }

        // Check if first block ends with conditional branch
        if let Some(Statement::Branch { condition, target }) = blocks[0].statements.last() {
            let then_block = self.block_to_control_flow(&blocks[1]);
            let else_block = if blocks.len() > 2 {
                Some(Box::new(self.block_to_control_flow(&blocks[2])))
            } else {
                None
            };

            return Some(ControlFlow::If {
                condition: self.condition_to_string(condition),
                then_block: Box::new(then_block),
                else_block,
            });
        }

        None
    }

    fn try_while_loop(&mut self, blocks: &[Block]) -> Option<ControlFlow> {
        // Simple while: block ends with branch back to itself
        if blocks.len() == 1 {
            if let Some(Statement::Branch { condition, target }) = blocks[0].statements.last() {
                // Check if target is start of block (simplified)
                let body_stmts: Vec<_> = blocks[0].statements.iter()
                    .filter(|s| !matches!(s, Statement::Branch { .. }))
                    .cloned()
                    .collect();

                return Some(ControlFlow::While {
                    condition: self.condition_to_string(condition),
                    body: Box::new(ControlFlow::Block(
                        body_stmts.iter().map(|s| self.statement_to_string(s)).collect()
                    )),
                });
            }
        }

        None
    }

    fn block_to_control_flow(&mut self, block: &Block) -> ControlFlow {
        let code: Vec<String> = block.statements.iter()
            .filter(|s| !matches!(s, Statement::Branch { .. }))
            .map(|s| self.statement_to_string(s))
            .collect();

        ControlFlow::Block(code)
    }

    fn statement_to_string(&mut self, stmt: &Statement) -> String {
        match stmt {
            Statement::Assign { dest, source } => {
                format!("{} = {};",
                    self.var_to_string(dest),
                    self.expr_to_string(source))
            }

            Statement::Store { addr, value } => {
                format!("mem[0x{:02X}] = {};", addr, self.expr_to_string(value))
            }

            Statement::Call { target } => {
                format!("call_${:04X}();", target)
            }

            Statement::Return => {
                "return;".to_string()
            }

            Statement::Branch { .. } => {
                // Handled by control flow
                String::new()
            }
        }
    }

    fn var_to_string(&mut self, var: &Variable) -> String {
        if let Some(name) = self.var_names.get(var) {
            return name.clone();
        }

        let name = match var {
            Variable::Temp(n) => format!("t{}", n),
            Variable::ZeroPage(addr) => format!("var_{:02X}", addr),
        };

        self.var_names.insert(var.clone(), name.clone());
        name
    }

    fn expr_to_string(&mut self, expr: &Expression) -> String {
        match expr {
            Expression::Constant(val) => format!("0x{:02X}", val),

            Expression::Variable(var) => self.var_to_string(var),

            Expression::BinaryOp { op, left, right } => {
                let op_str = match op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::And => "&",
                    BinaryOp::Or => "|",
                    BinaryOp::Xor => "^",
                };

                format!("({} {} {})",
                    self.expr_to_string(left),
                    op_str,
                    self.expr_to_string(right))
            }

            Expression::Load(addr) => {
                format!("mem[0x{:02X}]", addr)
            }
        }
    }

    fn condition_to_string(&self, cond: &Condition) -> String {
        match cond {
            Condition::Zero => "zero".to_string(),
            Condition::NotZero => "!zero".to_string(),
            Condition::Carry => "carry".to_string(),
            Condition::NotCarry => "!carry".to_string(),
            Condition::Negative => "negative".to_string(),
            Condition::Positive => "!negative".to_string(),
        }
    }

    fn generate_code(&mut self, cf: &ControlFlow, indent: usize) -> String {
        let ind = "    ".repeat(indent);

        match cf {
            ControlFlow::Sequence(seq) => {
                seq.iter()
                    .map(|c| self.generate_code(c, indent))
                    .collect::<Vec<_>>()
                    .join("\n")
            }

            ControlFlow::If { condition, then_block, else_block } => {
                let mut code = format!("{}if ({}) {{\n", ind, condition);
                code += &self.generate_code(then_block, indent + 1);
                code += &format!("\n{}}}", ind);

                if let Some(else_b) = else_block {
                    code += " else {\n";
                    code += &self.generate_code(else_b, indent + 1);
                    code += &format!("\n{}}}", ind);
                }

                code
            }

            ControlFlow::While { condition, body } => {
                let mut code = format!("{}while ({}) {{\n", ind, condition);
                code += &self.generate_code(body, indent + 1);
                code += &format!("\n{}}}", ind);
                code
            }

            ControlFlow::DoWhile { body, condition } => {
                let mut code = format!("{}do {{\n", ind);
                code += &self.generate_code(body, indent + 1);
                code += &format!("\n{}}} while ({});", ind, condition);
                code
            }

            ControlFlow::Block(statements) => {
                statements.iter()
                    .map(|s| format!("{}{}", ind, s))
                    .collect::<Vec<_>>()
                    .join("\n")
            }

            ControlFlow::Goto(target) => {
                format!("{}goto label_{:04X};", ind, target)
            }
        }
    }
}

/// Function parameter detector
pub struct ParameterDetector {
    // Track which zero-page addresses are read before written
    params: Vec<u8>,
}

impl ParameterDetector {
    pub fn new() -> Self {
        Self {
            params: Vec::new(),
        }
    }

    /// Detect function parameters from statements
    pub fn detect(&mut self, statements: &[Statement]) -> Vec<u8> {
        let mut read = HashSet::new();
        let mut written = HashSet::new();

        for stmt in statements {
            match stmt {
                Statement::Assign { source, dest } => {
                    // Check reads in source
                    self.collect_reads(source, &mut read);

                    // Mark as written
                    if let Variable::ZeroPage(addr) = dest {
                        written.insert(*addr);
                    }
                }

                Statement::Store { addr, value } => {
                    self.collect_reads(value, &mut read);
                    written.insert(*addr);
                }

                _ => {}
            }
        }

        // Parameters are variables read before written
        let mut params: Vec<_> = read.difference(&written)
            .cloned()
            .collect();
        params.sort();

        params
    }

    fn collect_reads(&self, expr: &Expression, read: &mut HashSet<u8>) {
        match expr {
            Expression::Variable(Variable::ZeroPage(addr)) => {
                read.insert(*addr);
            }
            Expression::BinaryOp { left, right, .. } => {
                self.collect_reads(left, read);
                self.collect_reads(right, read);
            }
            Expression::Load(addr) => {
                read.insert(*addr);
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
struct Block {
    start: usize,
    end: usize,
    statements: Vec<Statement>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_level_generation() {
        let statements = vec![
            Statement::Assign {
                dest: Variable::ZeroPage(0x50),
                source: Expression::Constant(0x10),
            },
            Statement::Assign {
                dest: Variable::ZeroPage(0x51),
                source: Expression::Constant(0x15),
            },
        ];

        let mut codegen = HighLevelCodeGen::new();
        let code = codegen.generate(&statements);

        assert!(code.contains("0x10"));
        assert!(code.contains("0x15"));
    }

    #[test]
    fn test_parameter_detection() {
        let statements = vec![
            Statement::Assign {
                dest: Variable::Temp(0),
                source: Expression::Load(0x50), // Read before write
            },
            Statement::Store {
                addr: 0x51,
                value: Expression::Variable(Variable::Temp(0)),
            },
        ];

        let mut detector = ParameterDetector::new();
        let params = detector.detect(&statements);

        assert_eq!(params, vec![0x50]);
    }
}
