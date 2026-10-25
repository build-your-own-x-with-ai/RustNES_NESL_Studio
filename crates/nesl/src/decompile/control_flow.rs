//! Control Flow Reconstruction
//!
//! Converts low-level goto-based code to structured control flow (if/while/for)

use std::collections::{HashMap, HashSet};

/// Basic block - a sequence of instructions without branches
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub start: u16,
    pub end: u16,
    pub instructions: Vec<String>,
    pub successors: Vec<u16>, // Addresses of blocks this can jump to
}

/// Control flow graph
#[derive(Debug)]
pub struct ControlFlowGraph {
    pub blocks: HashMap<u16, BasicBlock>,
    pub entry: u16,
}

/// Structured control flow node
#[derive(Debug, Clone)]
pub enum ControlFlow {
    Sequence(Vec<ControlFlow>),
    If {
        condition: String,
        then_block: Box<ControlFlow>,
        else_block: Option<Box<ControlFlow>>,
    },
    While {
        condition: String,
        body: Box<ControlFlow>,
    },
    DoWhile {
        body: Box<ControlFlow>,
        condition: String,
    },
    Block(Vec<String>), // Raw instructions
    Goto(u16),          // Fallback
}

impl ControlFlowGraph {
    /// Build CFG from disassembled code
    pub fn from_instructions(instructions: &[(u16, String)]) -> Self {
        let mut blocks = HashMap::new();
        let mut leaders = HashSet::new();

        // Identify basic block leaders
        leaders.insert(instructions[0].0); // First instruction

        for i in 0..instructions.len() {
            let (_addr, instr) = &instructions[i];

            // Branch target is a leader
            if let Some(target) = Self::extract_branch_target(instr) {
                leaders.insert(target);

                // Instruction after branch is a leader
                if i + 1 < instructions.len() {
                    leaders.insert(instructions[i + 1].0);
                }
            }
        }

        // Build basic blocks
        let mut current_block = Vec::new();
        let mut block_start = instructions[0].0;

        for (addr, instr) in instructions {
            if leaders.contains(addr) && !current_block.is_empty() {
                // Start new block
                let successors = Self::find_successors(&current_block);
                let block = BasicBlock {
                    start: block_start,
                    end: current_block.last().map(|(a, _)| *a).unwrap_or(block_start),
                    instructions: current_block.iter().map(|(_, i)| i.clone()).collect(),
                    successors,
                };
                blocks.insert(block_start, block);

                current_block.clear();
                block_start = *addr;
            }

            current_block.push((*addr, instr.clone()));
        }

        // Add last block
        if !current_block.is_empty() {
            let successors = Self::find_successors(&current_block);
            let block = BasicBlock {
                start: block_start,
                end: current_block.last().map(|(a, _)| *a).unwrap_or(block_start),
                instructions: current_block.iter().map(|(_, i)| i.clone()).collect(),
                successors,
            };
            blocks.insert(block_start, block);
        }

        ControlFlowGraph {
            blocks,
            entry: instructions[0].0,
        }
    }

    /// Extract branch target from instruction
    fn extract_branch_target(instr: &str) -> Option<u16> {
        // Parse "goto label" or "if (...) goto label"
        if let Some(pos) = instr.rfind("goto ") {
            let target_str = &instr[pos + 5..].trim();
            // Parse hex or decimal
            if target_str.starts_with("0x") || target_str.starts_with("$") {
                u16::from_str_radix(&target_str[2..], 16).ok()
            } else {
                target_str.parse().ok()
            }
        } else {
            None
        }
    }

    /// Find successor blocks
    fn find_successors(block: &[(u16, String)]) -> Vec<u16> {
        let mut successors = Vec::new();

        if let Some((addr, last_instr)) = block.last() {
            // Check for branch
            if let Some(target) = Self::extract_branch_target(last_instr) {
                successors.push(target);
            }

            // Fall-through (unless unconditional jump/return)
            if !last_instr.contains("return") &&
               !last_instr.starts_with("goto ") {
                successors.push(addr + 1); // Simplified
            }
        }

        successors
    }

    /// Reconstruct structured control flow
    pub fn reconstruct(&self) -> ControlFlow {
        self.reconstruct_block(self.entry, &mut HashSet::new())
    }

    fn reconstruct_block(&self, addr: u16, visited: &mut HashSet<u16>) -> ControlFlow {
        if visited.contains(&addr) {
            return ControlFlow::Goto(addr);
        }
        visited.insert(addr);

        let block = match self.blocks.get(&addr) {
            Some(b) => b,
            None => return ControlFlow::Block(vec![format!("// Unknown block ${:04X}", addr)]),
        };

        // Check for patterns
        if let Some(cf) = self.try_if_pattern(block, visited) {
            return cf;
        }

        if let Some(cf) = self.try_while_pattern(block, visited) {
            return cf;
        }

        // Fallback: sequence
        let mut seq = vec![ControlFlow::Block(block.instructions.clone())];

        // Add successors
        for &succ in &block.successors {
            if !visited.contains(&succ) {
                seq.push(self.reconstruct_block(succ, visited));
            }
        }

        if seq.len() == 1 {
            seq.into_iter().next().unwrap()
        } else {
            ControlFlow::Sequence(seq)
        }
    }

    /// Try to match if-then-else pattern
    fn try_if_pattern(&self, block: &BasicBlock, visited: &mut HashSet<u16>) -> Option<ControlFlow> {
        // Look for: conditional branch at end
        let last_instr = block.instructions.last()?;

        if !last_instr.starts_with("if ") {
            return None;
        }

        // Extract condition
        let condition = if let Some(cond_part) = last_instr.strip_prefix("if (") {
            if let Some(cond) = cond_part.split(") goto").next() {
                cond.to_string()
            } else {
                return None;
            }
        } else {
            return None;
        };

        if block.successors.len() != 2 {
            return None;
        }

        let then_addr = block.successors[0];
        let else_addr = block.successors[1];

        let then_block = self.reconstruct_block(then_addr, visited);
        let else_block = if visited.contains(&else_addr) {
            None
        } else {
            Some(Box::new(self.reconstruct_block(else_addr, visited)))
        };

        Some(ControlFlow::If {
            condition,
            then_block: Box::new(then_block),
            else_block,
        })
    }

    /// Try to match while loop pattern
    fn try_while_pattern(&self, block: &BasicBlock, visited: &mut HashSet<u16>) -> Option<ControlFlow> {
        // Look for: back-edge to self
        if !block.successors.contains(&block.start) {
            return None;
        }

        // Simple while loop
        let last_instr = block.instructions.last()?;

        if last_instr.starts_with("if ") {
            let condition = if let Some(cond_part) = last_instr.strip_prefix("if (") {
                if let Some(cond) = cond_part.split(") goto").next() {
                    cond.to_string()
                } else {
                    return None;
                }
            } else {
                return None;
            };

            let body = ControlFlow::Block(
                block.instructions[..block.instructions.len() - 1].to_vec()
            );

            return Some(ControlFlow::While {
                condition,
                body: Box::new(body),
            });
        }

        None
    }
}

impl ControlFlow {
    /// Generate high-level code from control flow
    pub fn to_code(&self, indent: usize) -> String {
        let ind = "    ".repeat(indent);

        match self {
            ControlFlow::Sequence(seq) => {
                seq.iter()
                    .map(|cf| cf.to_code(indent))
                    .collect::<Vec<_>>()
                    .join("\n")
            }

            ControlFlow::If { condition, then_block, else_block } => {
                let mut code = format!("{}if ({}) {{\n", ind, condition);
                code += &then_block.to_code(indent + 1);
                code += &format!("\n{}}}", ind);

                if let Some(else_b) = else_block {
                    code += &format!(" else {{\n");
                    code += &else_b.to_code(indent + 1);
                    code += &format!("\n{}}}", ind);
                }

                code
            }

            ControlFlow::While { condition, body } => {
                let mut code = format!("{}while ({}) {{\n", ind, condition);
                code += &body.to_code(indent + 1);
                code += &format!("\n{}}}", ind);
                code
            }

            ControlFlow::DoWhile { body, condition } => {
                let mut code = format!("{}do {{\n", ind);
                code += &body.to_code(indent + 1);
                code += &format!("\n{}}} while ({});", ind, condition);
                code
            }

            ControlFlow::Block(instrs) => {
                instrs.iter()
                    .map(|i| format!("{}{}", ind, i))
                    .collect::<Vec<_>>()
                    .join("\n")
            }

            ControlFlow::Goto(addr) => {
                format!("{}goto label_{:04X};", ind, addr)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_if_pattern() {
        let instructions = vec![
            (0x8000, "A = 5".to_string()),
            (0x8001, "if (A > 0) goto 0x8003".to_string()),
            (0x8002, "A = 0".to_string()),
            (0x8003, "return".to_string()),
        ];

        let cfg = ControlFlowGraph::from_instructions(&instructions);
        let cf = cfg.reconstruct();
        let code = cf.to_code(0);

        assert!(code.contains("if ("));
    }
}
