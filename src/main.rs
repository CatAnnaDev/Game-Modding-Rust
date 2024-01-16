use mach2::kern_return::KERN_SUCCESS;
use mach2::message::mach_msg_type_number_t;
use mach2::port::mach_port_t;
use mach2::traps::{mach_task_self, task_for_pid};
use mach2::vm::mach_vm_region;
use mach2::vm_prot::VM_PROT_EXECUTE;
use mach2::vm_region::{vm_region_basic_info, VM_REGION_BASIC_INFO_64};
use mach2::vm_types::{vm_address_t, vm_size_t};
use process_memory::{DataMember, Memory, Pid, TryIntoProcessHandle};
use sysinfo::System;

fn get_base_address(pid: i32) -> Option<vm_address_t>{

    unsafe {
        let mut task : mach_port_t = 0;
        if task_for_pid(mach_task_self(), pid, &mut task) != KERN_SUCCESS{
            return None
        }

        let mut address : vm_address_t = 1;
        let mut size : vm_size_t = 0;
        let mut info: vm_region_basic_info = std::mem::zeroed();
        let mut info_count = std::mem::size_of_val(&info) as mach_msg_type_number_t;
        let mut object_name: mach_port_t = 0;

        while mach_vm_region(task,
                             &mut address as *mut _ as *mut u64,
                             &mut size as *mut _ as *mut u64,
                             VM_REGION_BASIC_INFO_64,
                             &mut info as *mut _ as *mut i32,
                             &mut info_count,
                             &mut object_name ) == KERN_SUCCESS {

            if info.protection & VM_PROT_EXECUTE != 0 {
                return Some(address)
            }
            address += size;
        }

    }

    None
}

fn patch(offset: Vec<u64>, base_address: usize, pid: i32, new_value: u64){
    if let Ok(handle) = (pid as Pid).try_into_process_handle(){
        let mut current_address = base_address;
        let mut member: DataMember<u64> = DataMember::new(handle);
        for index in 0..offset.len() {
            member = DataMember::new_offset(handle, vec![current_address + offset[index] as usize]);
            unsafe {
                match member.read() {
                    Ok(value) => {println!("Read value address: 0x{:X}", value); current_address = value as usize}
                    Err(e) => { println!("Error: {}", e)}
                }
            }
        }

        match member.write(&new_value) {
            Ok(_) => { println!("New value {new_value} as been written") }
            Err(e) => {println!("Error during write: {}", e)}
        }
    }
}

fn main() {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut pid = 0;
    for x in sys.processes_by_exact_name("my_super_process_pid") {
        pid = x.pid().as_u32() as i32
    }
    println!("Pid is :{pid}");

    let mut base_address = 0;
    match get_base_address(pid) {
        None => println!("Base address not found"),
        Some(value) => base_address = value,
    }
    println!("Base address is : 0x{:X}", base_address);

    let offset = vec![0x1CBD38 ,0x100];
    patch(offset, base_address, pid, 9999);

}
