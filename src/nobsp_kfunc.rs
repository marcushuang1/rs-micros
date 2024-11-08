
use core::arch::asm;
use core::mem::size_of;
use core::ptr;
use spin::Mutex;
use riscv::register::{mie, mstatus, mideleg, medeleg, sstatus, sie};

use crate::error::{KError, KErrorType};
use crate::zone::{zone_type, kmalloc_page, kfree_page};
use crate::page;
use crate::vm::{ident_range_map, virt2phys};
use crate::cpu::{SATP_mode, TrapFrame, which_cpu};
use crate::lock::spin_mutex;
use crate::lock::{M_lock, S_lock};
use crate::plic::{plic_controller, plic_ctx, extint_src, extint_name};
use crate::CLINT;


use crate::{kmem,
            vm,
            cpu,
            S_UART,
            M_UART,
            KERNEL_TRAP_FRAME,};

pub fn kinit() -> Result<usize, KError> {
    let current_cpu = which_cpu();

    println!("CPU#{} is running its nobsp_kinit()", current_cpu);

    let pageroot_ptr = kmem::get_page_table();
    let mut pageroot = unsafe{pageroot_ptr.as_mut().unwrap()};

    cpu::satp_write(SATP_mode::Sv39, 0, pageroot_ptr as usize);

    cpu::mepc_write(crate::eh_func_nobsp_kmain as usize);

    // cpu::mstatus_write((1 << 11) | (1 << 5) as usize);

    /*
     * Now we only consider sw interrupt, timer and external
     * interrupt will be enabled in future
     *
     * We will delegate all interrupt into S-mode, enable S-mode
     * interrupt, and then disable M-mode interrupt
     */

    unsafe{
        mstatus::set_sie();
        CLINT.set_mtimecmp(current_cpu, u64::MAX);

        mie::set_msoft();

        mie::set_mtimer();

        mie::set_mext();

        mie::set_sext();
        sstatus::set_spie();
        sie::set_sext();

        mstatus::set_mpp(mstatus::MPP::Supervisor);
    }
    
    cpu::sfence_vma();

    
    Ok(0)
}

pub fn kmain() -> Result<(), KError> {
    let current_cpu = which_cpu();
    println!("CPU#{} Switched to S mode", current_cpu);

    unsafe{
        asm!("ebreak");

        println!("CPU{} Back from trap\n", current_cpu);
        CLINT.set_mtimecmp(current_cpu, CLINT.read_mtime() + 0x500_000);
    }

    loop{
        println!("CPU#{} kmain keep running...", current_cpu);
        let _ = cpu::busy_delay(1);
        unsafe{
            asm!("nop");
        }
    }
}
