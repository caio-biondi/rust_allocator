const HEAP_SIZE: usize = 4096;
const MAGIC: u32 = 0xDEADBEEF;

pub struct Header {
    size: usize,
    magic: u32
}

pub struct Node {
    size: usize,
    next: *mut Node
}

// A global variable to store the head of the free list.
static mut HEAD: *mut Node = std::ptr::null_mut() ;

// Calculates the amount of free memory available in the heap.
fn available_memory() -> usize {
    let mut n: usize = 0;
    let mut curr= unsafe{ HEAD };

    while !curr.is_null() {
        n += unsafe{ (*curr).size };
        curr = unsafe{ (*curr).next };
    }

    return n;
}

// Prints the free list. Useful for debugging purposes
fn print_free_list() {
    let mut curr= unsafe{ HEAD };

    while !curr.is_null() {
        print!("Free({})", unsafe{ (*curr).size });
        curr = unsafe{ (*curr).next };
        if !curr.is_null() {
            print!("->");
        }
    }
    println!();
}

// Returns the number of nodes on the free list.
fn number_of_free_nodes() -> u32 {
    let mut count: u32 = 0;
    let mut curr= unsafe{ HEAD };

    while !curr.is_null() {
        count += 1;
        curr = unsafe{ (*curr).next };
    }

    return count;
}

fn find_free_worst_fit(size: usize, found: &mut *mut Node, previous: &mut *mut Node) {

    let actual_size = size + std::mem::size_of::<Header>();

    let mut curr: *mut Node = unsafe { HEAD };
    let mut prev: *mut Node = std::ptr::null_mut();

    let mut worst_fit: Option<(*mut Node, usize)> = None; //keeps track of worst fit node
    let mut worst_fit_prev: *mut Node = std::ptr::null_mut(); //keeps track of node previous to worst_fit

    // Iterate over the free list
    while !curr.is_null() {

        // check if node is complete according to worst fit
        if unsafe { (*curr).size } >= actual_size {
            let remaining_size = unsafe { (*curr).size } - actual_size;

            if worst_fit.is_none() || remaining_size > worst_fit.unwrap().1 {
                worst_fit = Some((curr, remaining_size));
                worst_fit_prev = prev;
            }
        }

        prev = curr;
        curr = unsafe { (*curr).next };
    }

    if let Some((worst_fit_node, _)) = worst_fit {
        *found = worst_fit_node; // Return a pointer to node

        if !prev.is_null() {
            *previous = worst_fit_prev;
        }
    }
}

// Finds a node on the free list that has enough available memory to allocate to a calling program.
fn find_free(size: usize, found: & mut *mut Node, previous: & mut *mut Node) {
    // Size of Node must be able to accommodate size in bytes plus the size of Header
    let actual_size = size + std :: mem :: size_of :: <Header>();

    let mut curr: *mut Node = unsafe { HEAD }; // Head of the linked list
    let mut prev: *mut Node = std :: ptr :: null_mut(); // Previous node (initialize to None because head has no previous)

    // Iterate over free list
    while !curr.is_null() {

        // Node on the free list has been found
        if unsafe{(*curr).size} >= actual_size {

            *found = curr; // Return a pointer to node

            if !prev.is_null() {
                *previous = prev;
            }
            return
        }
        prev = curr;
        curr = unsafe{ (*curr).next };
    }
}

// Splits a found free node to accommodate an allocation request.
fn split(size: usize, previous: &mut *mut Node, free_block: &mut *mut Node, allocated: &mut *mut Header) {
    assert!(!free_block.is_null());

    // You will need a pointer to where the free block was originally before to assign allocated properly
    let hptr = *free_block;

    // Also accommodate the size of the Header struct
    let actual_size = size + std :: mem :: size_of :: <Header>();

    // Adjust the free block pointer by the number of bytes we need to allocate
    *free_block = (((*free_block) as usize) + actual_size) as *mut Node;

    // Update free_block ptr size
    unsafe { (**free_block).size = (*hptr).size - actual_size };

    unsafe { (**free_block).next = (*hptr).next };

    // Adjust the previous pointer to point to the new location of the free block
    if !(*previous).is_null() {
        unsafe{ (**previous).next = *free_block };
    } else { // Allocating from the first node in the free list
        unsafe{ HEAD = *free_block };
    }

    // Overlay/embed a header_t to the start of this piece of memory
    *allocated = hptr as *mut Header;

    unsafe{ (**allocated).size = size };
    unsafe{ (**allocated).magic = MAGIC };
}

// Returns a pointer to a region of memory having at least the request `size` bytes.
fn my_malloc(size: usize) -> *const std :: ffi :: c_void {
    // Define some local pointers to pass to these functions
    let mut found: *mut Node = std :: ptr :: null_mut();
    let mut previous: *mut Node = std :: ptr :: null_mut();

    let mut allocated: *mut Header = std :: ptr :: null_mut();

    find_free(size, &mut found, &mut previous);

    if found.is_null() {
       return std :: ptr :: null();
    }

    // Call split passing in the appropriate arguments
    split(size, &mut previous, &mut found, &mut allocated);

    // Adjust this pointer to be just after the allocated block's header (Header struct)
    let allocated_block: *const std :: ffi :: c_void = ((allocated as usize) + std :: mem :: size_of :: <Header>()) as *const std :: ffi :: c_void;

    return allocated_block;
}

// Perform coalescing between adjacent nodes on the free list
fn coalesce(free_block: *mut Node) {
    let mut curr: *mut Node = free_block;

    // Repeat the coalescing process on the newly coalesced node
    while number_of_free_nodes() != 1 {

        let curr_address: *mut Node = curr;
        let next_address: *mut Node = unsafe{ (*curr_address).next };
        let block_size: usize = unsafe{ (*free_block).size } + std :: mem :: size_of :: <Node>();

        // Compare the value of the current free block's next pointer to the address of the next adjacent block
        if !( ((curr_address as usize) + block_size) == (next_address as usize) ) {
            break;
        }

        // Adjusting pointers to merge two adjacent free blocks into one
        unsafe{ (*curr_address).next = (*next_address).next };

        // Make sure you update the size of the merged node to reflect the merged number of bytes and the size of the node_t at the start of the free block
        unsafe{ (*curr_address).size = (*curr_address).size + (*next_address).size + std :: mem :: size_of :: <Node>() };
        curr = curr_address;
    }
}

// Frees a given region of memory back to the free list.
fn my_free(allocated: *const std :: ffi :: c_void) {
    // Cast the allocated parameter to a *const Header and adjust the pointer that is given by size of Header to point to the actual start of the allocated block
    let allocated_block = ((allocated as usize) - std :: mem :: size_of :: <Header>()) as *mut Header;

    // Assert to ensure that the magic field is indeed equal to MAGIC
    assert_eq!(unsafe { (*allocated_block).magic }, MAGIC);

    // Cast Header struct to a Node struct and set the size to the size of bytes we are freeing
    let freed_node = allocated_block as *mut Node;
    unsafe{ (*freed_node).size = (*allocated_block).size };

    // Link in the freed node into the free list by making this newly freed node the start of the heap
    unsafe { (*freed_node).next = HEAD };
    unsafe{ HEAD = freed_node };
    coalesce(freed_node);
}

extern crate mmap;

fn main() {
    // Open a new memory map with HEAP_SIZE
    let memory_map = mmap :: MemoryMap::new(HEAP_SIZE, &[ mmap :: MapOption::MapReadable, mmap :: MapOption::MapWritable]).unwrap();

    // Get a raw pointer to the start of the allocated block
    unsafe { HEAD = memory_map.data() as *mut Node };

    unsafe{ (*HEAD).size = HEAP_SIZE - std :: mem :: size_of :: <Node>() };
    unsafe{ (*HEAD).next = std :: ptr :: null_mut() };

    let mut allocated: [*const std :: ffi :: c_void; 10] = [std :: ptr :: null_mut(); 10];

    println!();
    println!("Available memory before: {}", available_memory());
    print_free_list();

    for element in allocated.iter_mut() {
        *element = my_malloc(128);
    }

    println!();
    println!("Available memory after: {}", available_memory());
    print_free_list();

    println!();
    println!("Available memory before: {}", available_memory());
    print_free_list();

    for element in allocated.iter().rev() {
        my_free(*element);
    }

    println!();
    println!("Available memory after: {}", available_memory());
    print_free_list();
}