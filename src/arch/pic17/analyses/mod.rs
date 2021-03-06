use yaxpeax_arch::{Arch, LengthedInstruction};
use yaxpeax_pic17::PIC17;
use analyses::control_flow::Effect;
use arch::pic17::cpu::try_debank;
use arch::pic17::StateUpdate;
use arch::pic17;
use ContextRead;

pub fn all_instruction_analyses(
    instr: &<PIC17 as Arch>::Instruction,
    address: <PIC17 as Arch>::Address,
    effect: &Effect<<PIC17 as Arch>::Address>,
    ctxs: &pic17::MergedContextTable
) -> Vec<(<PIC17 as Arch>::Address, StateUpdate)> {
    let mut results = compute_bit_name(instr, address, effect, ctxs);
    results.extend(collect_function_hint(instr, address, effect, ctxs));
    results.extend(compute_next_state(instr, address, effect, ctxs));
    results
}

pub fn compute_next_state(
    instr: &<PIC17 as Arch>::Instruction,
    address: <PIC17 as Arch>::Address,
    effect: &Effect<<PIC17 as Arch>::Address>,
    ctxs: &pic17::MergedContextTable
) -> Vec<(<PIC17 as Arch>::Address, StateUpdate)> {
    // this exposes a bug. if the current instruction ends a basic block
    // we might compute some state applied to the next instruction eg after the
    // end of this block.
    //
    // this is permitted because blocks are Copy and the version that's updated
    // is in the cfg where we have a potentially out of date one.
/*
    let mut ctx = pic17::compute_state(&instr, &ctx_table.at(&address));

    if address < block.end {
        ctx_table.computed_contexts.insert(address + instr.len(), ctx);
    }
    */
    let ctx = pic17::compute_state(&instr, &ctxs.at(&address));

    if !effect.is_stop() {
        vec![(address + instr.len(), pic17::StateUpdate::FullContext(ctx))]
//                    ctxs.computed_contexts[(address + instr.len()) as usize] = Some(ctx);
    } else {
        vec![]
    }
}

// TODO: unbreak this
pub fn collect_function_hint(
    _instr: &<PIC17 as Arch>::Instruction,
    _address: <PIC17 as Arch>::Address,
    _effect: &Effect<<PIC17 as Arch>::Address>,
    _ctxs: &pic17::MergedContextTable
) -> Vec<(<PIC17 as Arch>::Address, StateUpdate)> {
    vec![]
    /*
    if instr.is_call() {
        match effect.dest {
            Some(Target::Relative(rel)) => {
                // TODO: have a high level program info
                // declare functions on it.
                // program_model.declare_function(
                // instead.....
                let addr = address + rel;
                if !function_table.contains_key(&addr) {
                    vec![(addr, pic17::Function::new(format!("fn_{:04x}", addr), vec![]))]
                }
            },
            Some(Target::Absolute(dest)) => {
                // TODO: have a high level program info
                // declare functions on it.
                // program_model.declare_function(
                // instead.....
                let addr = dest;
                if !function_table.contains_key(&addr) {
                    vec![(addr, pic17::Function::new(format!("fn_{:04x}", addr), vec![]))]
                }
            }
            _ => {
                panic!("What *is* a call to an unknown or multiple dest?");
            }
        }
    }
    */
}

pub fn compute_bit_name(
    instr: &<PIC17 as Arch>::Instruction,
    address: <PIC17 as Arch>::Address,
    _effect: &Effect<<PIC17 as Arch>::Address>,
    ctxs: &pic17::MergedContextTable
) -> Vec<(<PIC17 as Arch>::Address, StateUpdate)> {
    let comment = match instr.opcode {
        yaxpeax_pic17::Opcode::BTG |
        yaxpeax_pic17::Opcode::BSF |
        yaxpeax_pic17::Opcode::BCF |
        yaxpeax_pic17::Opcode::BTFSS |
        yaxpeax_pic17::Opcode::BTFSC => {
            let bit_num = match instr.operands[1] {
                yaxpeax_pic17::Operand::ImmediateU8(bit_num) => bit_num,
                _ => unreachable!()
            };

            let file_value = match instr.operands[0] {
                yaxpeax_pic17::Operand::File(f) => f,
                _ => unreachable!()
            };

//                        match try_debank(file_value, Some(&ctx_table.at(&address))) {
            match try_debank(file_value, Some(&ctxs.at(&address))) {
                Some(file) => {
                    pic17::bit_name(file, bit_num)
                },
                None => None
            }
        },
        _ => {
            None
        }
    };

    match comment {
        Some(comment) => {
            vec![(address + instr.len(), StateUpdate::ComputedComment(comment.to_owned()))]
        },
        None => {
            vec![]
        }
    }
}
