// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2022-2023 SUSE LLC
//
// Author: Joerg Roedel <jroedel@suse.de>

use super::common::{load_idt, Idt, IdtEntry, DF_VECTOR, GLOBAL_IDT, VC_VECTOR};
use crate::address::VirtAddr;
use crate::cpu::control_regs::read_cr2;
use crate::cpu::vc::{stage2_handle_vc_exception, stage2_handle_vc_exception_no_ghcb};
use crate::cpu::X86ExceptionContext;
use core::arch::global_asm;

fn init_idt(idt: &mut Idt, handler_array: *const u8) {
    // Set IDT handlers
    let handlers = VirtAddr::from(handler_array);
    for (i, entry) in idt.iter_mut().enumerate() {
        *entry = IdtEntry::entry(handlers + (32 * i));
    }
}

pub fn early_idt_init_no_ghcb() {
    unsafe {
        init_idt(
            &mut GLOBAL_IDT,
            &stage2_idt_handler_array_no_ghcb as *const u8,
        );
        load_idt(&GLOBAL_IDT);
    }
}

pub fn early_idt_init() {
    unsafe {
        init_idt(&mut GLOBAL_IDT, &stage2_idt_handler_array as *const u8);
        load_idt(&GLOBAL_IDT);
    }
}

#[no_mangle]
pub extern "C" fn stage2_generic_idt_handler(ctx: &mut X86ExceptionContext) {
    match ctx.vector {
        DF_VECTOR => {
            let cr2 = read_cr2();
            let rip = ctx.frame.rip;
            let rsp = ctx.frame.rsp;
            panic!(
                "Double-Fault at RIP {:#018x} RSP: {:#018x} CR2: {:#018x}",
                rip, rsp, cr2
            );
        }
        VC_VECTOR => stage2_handle_vc_exception(ctx),
        _ => {
            let err = ctx.error_code;
            let vec = ctx.vector;
            let rip = ctx.frame.rip;

            panic!(
                "Unhandled exception {} RIP {:#018x} error code: {:#018x}",
                vec, rip, err
            );
        }
    }
}

#[no_mangle]
pub extern "C" fn stage2_generic_idt_handler_no_ghcb(ctx: &mut X86ExceptionContext) {
    match ctx.vector {
        DF_VECTOR => {
            let cr2 = read_cr2();
            let rip = ctx.frame.rip;
            let rsp = ctx.frame.rsp;
            panic!(
                "Double-Fault at RIP {:#018x} RSP: {:#018x} CR2: {:#018x}",
                rip, rsp, cr2
            );
        }
        VC_VECTOR => stage2_handle_vc_exception_no_ghcb(ctx),
        _ => {
            let err = ctx.error_code;
            let vec = ctx.vector;
            let rip = ctx.frame.rip;

            panic!(
                "Unhandled exception {} RIP {:#018x} error code: {:#018x}",
                vec, rip, err
            );
        }
    }
}

extern "C" {
    static stage2_idt_handler_array: u8;
    static stage2_idt_handler_array_no_ghcb: u8;
}

global_asm!(
    r#"
         /* Early tage 2 handler array setup */
        .text
    push_regs_no_ghcb:
        pushq   %rax
        pushq   %rbx
        pushq   %rcx
        pushq   %rdx
        pushq   %rsi
        pushq   %rdi
        pushq   %rbp
        pushq   %r8
        pushq   %r9
        pushq   %r10
        pushq   %r11
        pushq   %r12
        pushq   %r13
        pushq   %r14
        pushq   %r15

        movq    %rsp, %rdi
        call    stage2_generic_idt_handler_no_ghcb

        jmp generic_idt_handler_return
        
        .align 32
        .globl stage2_idt_handler_array_no_ghcb
    stage2_idt_handler_array_no_ghcb:
        i = 0
        .rept 32
        .align 32
        .if ((0x20027d00 >> i) & 1) == 0
        pushq   $0
        .endif
        pushq   $i  /* Vector Number */
        jmp push_regs_no_ghcb
        i = i + 1
        .endr
        
        /* Stage 2 handler array setup */
        .text
    push_regs_stage2:
        pushq   %rax
        pushq   %rbx
        pushq   %rcx
        pushq   %rdx
        pushq   %rsi
        pushq   %rdi
        pushq   %rbp
        pushq   %r8
        pushq   %r9
        pushq   %r10
        pushq   %r11
        pushq   %r12
        pushq   %r13
        pushq   %r14
        pushq   %r15

        movq    %rsp, %rdi
        call    stage2_generic_idt_handler

        jmp generic_idt_handler_return
        
        .align 32
        .globl stage2_idt_handler_array
    stage2_idt_handler_array:
        i = 0
        .rept 32
        .align 32
        .if ((0x20027d00 >> i) & 1) == 0
        pushq   $0
        .endif
        pushq   $i  /* Vector Number */
        jmp push_regs_stage2
        i = i + 1
        .endr
    "#,
    options(att_syntax)
);
