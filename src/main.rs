use crate::my_malloc::{
    available_memory, my_free, my_malloc, number_of_free_nodes, print_free_list, reset_heap,
};

mod my_malloc;

fn main() {
    reset_heap();

    println!("Available memory: {} bytes", available_memory());
    println!("{} free nodes", number_of_free_nodes());
    print_free_list();
    println!();

    // request 4 blocks of size = 16 bytes
    let p = my_malloc(16);
    let q = my_malloc(16);
    let r = my_malloc(16);
    let s = my_malloc(16);

    println!("Allocated block p at : {:p}", p);
    println!("Allocated block q at : {:p}", q);
    println!("Allocated block s at : {:p}", r);
    println!("Allocated block s at : {:p}", s);

    println!();
    println!("Available memory: {} bytes", available_memory());
    println!("{} free nodes", number_of_free_nodes());
    print_free_list();
    println!();

    // release 1st and 3rd blocks
    my_free(r);
    my_free(p);

    println!("Freed block at: {:p}", r);
    println!("Freed block at: {:p}", p);

    println!();
    println!("Available memory: {} bytes", available_memory());
    println!("{} free nodes", number_of_free_nodes());
    print_free_list();
    println!();

    // release 2nd block and embark in coalescing under case 4
    my_free(q);
    println!("Freed block at: {:p}", q);

    println!();
    println!("Available memory: {} bytes", available_memory());
    println!("{} free nodes", number_of_free_nodes());
    print_free_list();
    println!();
}
