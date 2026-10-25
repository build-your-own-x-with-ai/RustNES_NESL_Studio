//! Phase 3: Register Elimination
//!
//! Converts low-level register operations (A, X, Y) to high-level variable operations
//!
//! This is a critical step in decompilation:
//! - Track register values through the program
//! - Replace register operations with variable assignments
//! - Eliminate temporary register usage

use std::collections::{HashMap, HashSet};
use crate::disasm::{Instruction, AddressingMode};

/// Register state tracking
#[derive(Debug, Clone)]
pub struct RegisterState {
    pub a: Option<Value>,
    pub x: Option<Value>,
    pub y: Option<Value>,
    pub flags: FlagState,
}

#[derive(Debug, Clone)]
pub enum Value {
    Constant(u8),
    Variable(u8),       // Zero-page address
    Unknown,
}

#[derive(Debug, Clone)]
pub struct FlagState {
    pub zero: Option<bool>,
    pub carry: Option<bool>,
    pub negative: Option<bool>,
}

impl RegisterState {
    pub fn new() -> Self {
        Self {
            a: None,
            x: None,
            y: None,
            flags: FlagState {
                zero: None,
                carry: None,
                negative: None,
            },
        }
    }

    /// Update state based on instruction
    pub fn execute(&mut self, instr: &Instruction, val: u16) {
        match instr {
            // Load instructions
            Instruction::LDA(mode, _val) => {
                self.a = self.value_from_mode(mode, val);
            }
            Instruction::LDX(mode, _val) => {
                self.x = self.value_from_mode(mode, val);
            }
            Instruction::LDY(mode, _val) => {
                self.y = self.value_from_mode(mode, val);
            }

            // Store instructions - invalidate the destination
            Instruction::STA(_, _) | Instruction::STX(_, _) | Instruction::STY(_, _) => {
                // Memory modified, but registers unchanged
            }

            // Transfer instructions
            Instruction::TAX => {
                self.x = self.a.clone();
            }
            Instruction::TAY => {
                self.y = self.a.clone();
            }
            Instruction::TXA => {
                self.a = self.x.clone();
            }
            Instruction::TYA => {
                self.a = self.y.clone();
            }

            // Arithmetic - result unknown
            Instruction::ADC(_, _) | Instruction::SBC(_, _) => {
                self.a = Some(Value::Unknown);
            }

            // Increment/Decrement
            Instruction::INX => {
                self.x = Some(Value::Unknown);
            }
            Instruction::INY => {
                self.y = Some(Value::Unknown);
            }
            Instruction::DEX => {
                self.x = Some(Value::Unknown);
            }
            Instruction::DEY => {
                self.y = Some(Value::Unknown);
            }

            // Compare instructions - affect flags only
            Instruction::CMP(_, _) | Instruction::CPX(_, _) | Instruction::CPY(_, _) => {
                // Update flags but registers unchanged
            }

            // Logical operations
            Instruction::AND(_, _) | Instruction::ORA(_, _) | Instruction::EOR(_, _) => {
                self.a = Some(Value::Unknown);
            }

            // Shifts
            Instruction::ASL(_, _) | Instruction::LSR(_, _) | Instruction::ROL(_, _) | Instruction::ROR(_, _) => {
                self.a = Some(Value::Unknown);
            }

            // Branch/Jump - no register change
            _ => {}
        }
    }

    fn value_from_mode(&self, mode: &AddressingMode, val: u16) -> Option<Value> {
        match mode {
            AddressingMode::Immediate => Some(Value::Constant(val as u8)),
            AddressingMode::ZeroPage => Some(Value::Variable(val as u8)),
            _ => Some(Value::Unknown),
        }
    }
}

/// Statement in Static Single Assignment (SSA) form
#[derive(Debug, Clone)]
pub enum Statement {
    Assign {
        dest: Variable,
        source: Expression,
    },
    Store {
        addr: u8,
        value: Expression,
    },
    Call {
        target: u16,
    },
    Return,
    Branch {
        condition: Condition,
        target: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Variable {
    Temp(usize),      // Temporary (register)
    ZeroPage(u8),     // Zero-page variable
}

#[derive(Debug, Clone)]
pub enum Expression {
    Constant(u8),
    Variable(Variable),
    BinaryOp {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Load(u8),         // Load from zero-page
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    And,
    Or,
    Xor,
}

#[derive(Debug, Clone)]
pub enum Condition {
    Zero,
    NotZero,
    Carry,
    NotCarry,
    Negative,
    Positive,
}

/// Register elimination transformer
pub struct RegisterEliminator {
    next_temp: usize,
    statements: Vec<Statement>,
}

impl RegisterEliminator {
    pub fn new() -> Self {
        Self {
            next_temp: 0,
            statements: Vec::new(),
        }
    }

    /// Convert instruction sequence to SSA form
    pub fn eliminate_registers(&mut self, instructions: &[(Instruction, u16)]) -> Vec<Statement> {
        let mut state = RegisterState::new();
        let mut a_temp = self.new_temp();
        let mut _x_temp = self.new_temp();
        let mut _y_temp = self.new_temp();

        for (instr, val) in instructions {
            match instr {
                Instruction::LDA(mode, _instr_val) => {
                    let expr = self.mode_to_expr(mode, *val);
                    self.statements.push(Statement::Assign {
                        dest: Variable::Temp(a_temp),
                        source: expr,
                    });
                }

                Instruction::STA(mode, _instr_val) => {
                    if let AddressingMode::ZeroPage = mode {
                        self.statements.push(Statement::Store {
                            addr: *val as u8,
                            value: Expression::Variable(Variable::Temp(a_temp)),
                        });
                    }
                }

                Instruction::ADC(mode, _instr_val) => {
                    let right = self.mode_to_expr(mode, *val);
                    let new_temp = self.new_temp();
                    self.statements.push(Statement::Assign {
                        dest: Variable::Temp(new_temp),
                        source: Expression::BinaryOp {
                            op: BinaryOp::Add,
                            left: Box::new(Expression::Variable(Variable::Temp(a_temp))),
                            right: Box::new(right),
                        },
                    });
                    a_temp = new_temp;
                }

                Instruction::TAX => {
                    _x_temp = a_temp; // Just alias
                }

                Instruction::RTS => {
                    self.statements.push(Statement::Return);
                }

                Instruction::JSR(target) => {
                    self.statements.push(Statement::Call { target: *target });
                }

                Instruction::BNE(_offset) => {
                    // Calculate target (simplified)
                    let target = 0; // Would calculate from offset
                    self.statements.push(Statement::Branch {
                        condition: Condition::NotZero,
                        target,
                    });
                }

                _ => {
                    // Handle other instructions
                }
            }

            state.execute(instr, *val);
        }

        self.statements.clone()
    }

    fn new_temp(&mut self) -> usize {
        let t = self.next_temp;
        self.next_temp += 1;
        t
    }

    fn mode_to_expr(&self, mode: &AddressingMode, val: u16) -> Expression {
        match mode {
            AddressingMode::Immediate => Expression::Constant(val as u8),
            AddressingMode::ZeroPage => Expression::Load(val as u8),
            _ => Expression::Constant(0),
        }
    }

    /// Generate high-level code from SSA
    pub fn to_code(&self, var_names: &HashMap<u8, String>) -> String {
        let mut code = String::new();

        for stmt in &self.statements {
            match stmt {
                Statement::Assign { dest, source } => {
                    code.push_str(&format!("    {} = {};\n",
                        self.var_to_string(dest, var_names),
                        self.expr_to_string(source, var_names)));
                }

                Statement::Store { addr, value } => {
                    let var_name = var_names.get(addr)
                        .cloned()
                        .unwrap_or_else(|| format!("var_{:02X}", addr));
                    code.push_str(&format!("    {} = {};\n",
                        var_name,
                        self.expr_to_string(value, var_names)));
                }

                Statement::Call { target } => {
                    code.push_str(&format!("    call_${:04X}();\n", target));
                }

                Statement::Return => {
                    code.push_str("    return;\n");
                }

                Statement::Branch { condition, target } => {
                    code.push_str(&format!("    if ({}) goto label_{:04X};\n",
                        self.condition_to_string(condition),
                        target));
                }
            }
        }

        code
    }

    fn var_to_string(&self, var: &Variable, var_names: &HashMap<u8, String>) -> String {
        match var {
            Variable::Temp(n) => format!("t{}", n),
            Variable::ZeroPage(addr) => {
                var_names.get(addr)
                    .cloned()
                    .unwrap_or_else(|| format!("var_{:02X}", addr))
            }
        }
    }

    fn expr_to_string(&self, expr: &Expression, var_names: &HashMap<u8, String>) -> String {
        match expr {
            Expression::Constant(val) => format!("0x{:02X}", val),
            Expression::Variable(var) => self.var_to_string(var, var_names),
            Expression::BinaryOp { op, left, right } => {
                format!("({} {} {})",
                    self.expr_to_string(left, var_names),
                    match op {
                        BinaryOp::Add => "+",
                        BinaryOp::Sub => "-",
                        BinaryOp::And => "&",
                        BinaryOp::Or => "|",
                        BinaryOp::Xor => "^",
                    },
                    self.expr_to_string(right, var_names))
            }
            Expression::Load(addr) => {
                var_names.get(addr)
                    .cloned()
                    .unwrap_or_else(|| format!("var_{:02X}", addr))
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_elimination() {
        let instructions = vec![
            (Instruction::LDA(AddressingMode::Immediate, 0x10), 0x10),
            (Instruction::STA(AddressingMode::ZeroPage, 0x50), 0x50),
        ];

        let mut eliminator = RegisterEliminator::new();
        let statements = eliminator.eliminate_registers(&instructions);

        assert_eq!(statements.len(), 2);
    }
}
