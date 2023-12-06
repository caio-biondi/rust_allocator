extern crate libc;

use std::ffi::c_void;
use std::mem::size_of;

const HEAP_SIZE: usize = 4096;

// header and boundary tag struct
pub struct Mast {
    size: usize,
    a: i8,
}
// track blocks in free list
pub struct Node {
    next: *mut Node,
    prev: *mut Node,
}

// global variable. head of free list
static mut HEAD: *mut Node = std::ptr::null_mut();

pub fn heap() -> *mut Node {
    if unsafe { HEAD.is_null() } {
        // allocate heap
        let memory_map = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                HEAP_SIZE,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1,
                0,
            )
        };

        if memory_map == libc::MAP_FAILED {
            panic!("Memory allocation failed");
        }

        // ptr to the header and boundary tag of HEAD
        let h_hptr = memory_map as *mut Mast;
        let h_fptr = (memory_map as usize + HEAP_SIZE - size_of::<Mast>()) as *mut Mast;

        // adjust HEAD size and flag. accommodate size of header, boundary tag and next / prev ptrs
        unsafe {
            (*h_hptr).size = HEAP_SIZE - (2 * size_of::<Mast>()) - size_of::<Node>();
            (*h_hptr).a = 0;
            (*h_fptr).size = (*h_hptr).size;
            (*h_fptr).a = 0;
        };

        let nnode = (memory_map as usize + size_of::<Mast>()) as *mut Node;
        unsafe {
            (*nnode).next = std::ptr::null_mut();
            (*nnode).prev = std::ptr::null_mut();
            HEAD = nnode;
        };
        println!();
        println!("Allocated heap at: {:p}", unsafe { HEAD });
        println!();
    }

    unsafe {
        return HEAD;
    };
}

pub fn reset_heap() {
    if unsafe { !HEAD.is_null() } {
        unsafe {
            libc::munmap(
                (HEAD as usize - size_of::<Mast>()) as *mut c_void,
                HEAP_SIZE,
            )
        };
        unsafe { HEAD = std::ptr::null_mut() };
        heap();
    }
}

// calculates amount of free memory available heap
pub fn available_memory() -> usize {
    let mut n: usize = 0;
    let mut curr = heap();

    while !curr.is_null() {
        n += unsafe { (*((curr as usize - size_of::<Mast>()) as *mut Mast)).size };
        curr = unsafe { (*curr).next };
    }

    return n;
}

// prints free list. useful for debugging purposes
pub fn print_free_list() {
    let mut curr = heap();

    while !curr.is_null() {
        print!("Free({})", unsafe {
            (*((curr as usize - size_of::<Mast>()) as *mut Mast)).size
        });
        curr = unsafe { (*curr).next };
        if !curr.is_null() {
            print!("->");
        }
    }
    println!();
}

// returns # of nodes on free list
pub fn number_of_free_nodes() -> u32 {
    let mut count: u32 = 0;
    let mut curr = heap();

    while !curr.is_null() {
        count += 1;
        curr = unsafe { (*curr).next };
    }

    return count;
}

// insert node at the head of this free list
fn push(p: *mut Node) {
    // make next of new node as head and previous as NULL
    unsafe { (*p).next = HEAD }
    unsafe { (*p).prev = std::ptr::null_mut() }

    // change prev of head node to new node
    if unsafe { !HEAD.is_null() } {
        unsafe { (*HEAD).prev = p }
    }

    // move the head to point to the new node
    unsafe { HEAD = p }
}

fn delete_node(p: *mut Node) {
    // Base case
    assert!(!unsafe { HEAD.is_null() });
    assert!(!p.is_null());

    // if node to be deleted is HEAD
    if unsafe { HEAD == p } {
        unsafe { HEAD = (*p).next }
    }

    // change next only if node to be deleted is NOT the last node
    if unsafe { !(*p).next.is_null() } {
        unsafe { (*((*p).next)).prev = (*p).prev }
    }

    // change prev only if node to be deleted is NOT the first node
    if unsafe { !(*p).prev.is_null() } {
        unsafe { (*((*p).prev)).next = (*p).next }
    }
}

// finds node on free list that has enough memory to allocate to calling program. best-fit allocation scheme
// fn find_free(size: usize, found: &mut *mut Node) {
//     // accommodate header and boundary tag
//     let tsize = size + (2 * size_of::<Mast>());
//     let mut curr = heap();
//
//     // iterate free list
//     while !curr.is_null() {
//         let curr_size = unsafe { (*((curr as usize - size_of::<Mast>()) as *mut Mast)).size };
//
//         // node on free list found
//         if curr_size >= tsize {
//             if (*found).is_null() {
//                 *found = curr // return ptr to node
//             } else if unsafe { (*((*found as usize - size_of::<Mast>()) as *mut Mast)).size }
//                 > curr_size
//             {
//                 // worse node found
//                 *found = curr; // return ptr to node
//             }
//         }
//         curr = unsafe { (*curr).next };
//     }
// }

// finds node on free list that has enough memory to allocate to calling program. worst-fit allocation scheme
// fn find_free(size: usize, found: &mut *mut Node) {
//     // accommodate header and boundary tag
//     let tsize = size + (2 * size_of::<Mast>());
//     let mut curr = heap();
//
//     // iterate free list
//     while !curr.is_null() {
//         let curr_size = unsafe { (*((curr as usize - size_of::<Mast>()) as *mut Mast)).size };
//
//         // node on free list found
//         if curr_size >= tsize {
//             if (*found).is_null() {
//                 *found = curr // return ptr to node
//             } else if unsafe { (*((*found as usize - size_of::<Mast>()) as *mut Mast)).size }
//                 < curr_size
//             {
//                 // worse node found
//                 *found = curr; // return ptr to node
//             }
//         }
//         curr = unsafe { (*curr).next };
//     }
// }

// finds node on free list that has enough memory to allocate to calling program. first-fit allocation scheme
fn find_free(size: usize, found: &mut *mut Node) {
    // accommodate header and boundary tag
    let tsize = size + (2 * size_of::<Mast>());
    let mut curr = heap();

    // iterate free list
    while !curr.is_null() {
        // node on free list found
        if unsafe { (*((curr as usize - size_of::<Mast>()) as *mut Mast)).size } >= tsize {
            *found = curr; // return ptr to node
            return;
        }
        curr = unsafe { (*curr).next };
    }
}

// split fb to allocate requested bytes
unsafe fn split(ab_bs: usize, fb: &mut *mut Node, ab_hptr: &mut *mut Mast) {
    assert!(!fb.is_null());
    let fb_bs = (*((*fb as usize - size_of::<Mast>()) as *mut Mast)).size;
    let ab = *fb; // ptr to where fb was originally
    let tsize = ab_bs + (2 * size_of::<Mast>()); // accommodate size, header and boundary tag
    *fb = (*fb as usize + tsize) as *mut Node; // shift fb ptr by # of bytes to allocate
    let fb_hptr = (*fb as usize - size_of::<Mast>()) as *mut Mast; // ptr to header of fb
    let fb_fptr = (*fb as usize + size_of::<Node>() + fb_bs - tsize) as *mut Mast; // ptr to boundary tag of fb

    // update fb ptr size and flag
    (*fb_hptr).size = fb_bs - tsize;
    (*fb_fptr).size = (*fb_hptr).size;
    (*fb_hptr).a = 0;
    (*fb_fptr).a = 0;

    // adjust next and prev ptr to point to new location
    let prev = (*ab).prev;
    let next = (*ab).next;
    (**fb).next = next;
    (**fb).prev = prev;
    if !next.is_null() {
        (*next).prev = *fb;
    }
    if !prev.is_null() {
        (*prev).next = *fb;
    } else {
        // allocating from head
        HEAD = *fb;
    }
    *ab_hptr = (ab as usize - size_of::<Mast>()) as *mut Mast;

    let ab_fptr = (ab as usize + ab_bs) as *mut Mast; // ptr to boundary tag of ab

    (**ab_hptr).size = ab_bs;
    (**ab_hptr).a = 1;
    (*ab_fptr).size = ab_bs;
    (*ab_fptr).a = 1;
}

// returns ptr to region of memory having at lest request `size` bytes
pub fn my_malloc(size: usize) -> *const c_void {
    // local ptrs to pass to functions
    let mut found: *mut Node = std::ptr::null_mut();
    let mut a: *mut Mast = std::ptr::null_mut();

    find_free(size, &mut found);

    if found.is_null() {
        return std::ptr::null();
    }

    // split w/ appropriate arguments
    unsafe { split(size, &mut found, &mut a) };

    // adjust ptr to be after ab's header
    let ab = (a as usize + size_of::<Mast>()) as *const c_void;
    return ab;
}

// perform coalescing on adjacent nodes
fn coalesce(free_block: *const c_void) {
    // ptr to header and boundary tag of fb
    let fb_hptr = (free_block as usize - size_of::<Mast>()) as *mut Mast;
    let fb_bs = unsafe { (*fb_hptr).size };
    let fb_fptr = (free_block as usize + size_of::<Node>() + fb_bs) as *mut Mast;

    // ptr to prev addr boundary tag (possibly non-existent)
    let p_fptr = (fb_hptr as usize - size_of::<Mast>()) as *mut Mast;

    // ptr to the next addr header (possibly non-existent)
    let s_hptr = (fb_fptr as usize + size_of::<Mast>()) as *mut Mast;

    // heap boundaries
    let min = unsafe { HEAD as usize - size_of::<Mast>() };
    let max = min + HEAP_SIZE;

    // case 4
    if unsafe { (*p_fptr).a == 0 }
        && (p_fptr as usize) >= min
        && (p_fptr as usize) < max
        && unsafe { (*s_hptr).a == 0 }
        && (s_hptr as usize) >= min
        && (s_hptr as usize) < max
    {
        let p_bs = unsafe { (*p_fptr).size };
        let s_bs = unsafe { (*s_hptr).size };

        // ptr new fb header and boundary tag
        let nfb_hptr =
            (p_fptr as usize - p_bs - size_of::<Node>() - size_of::<Mast>()) as *mut Mast;
        let nfb_fptr =
            (s_hptr as usize + s_bs + size_of::<Node>() + size_of::<Mast>()) as *mut Mast;

        // splice succ and pred out of list
        let pred = (p_fptr as usize - p_bs - size_of::<Node>()) as *mut Node;
        let succ = (s_hptr as usize + size_of::<Mast>()) as *mut Node;
        delete_node(pred);
        delete_node(succ);

        // coalesce all three memory blocks
        unsafe {
            (*nfb_hptr).size = (*nfb_hptr).size
                + (*fb_hptr).size
                + (*nfb_fptr).size
                + (4 * size_of::<Mast>())
                + (2 * size_of::<Node>());
            (*nfb_fptr).size = (*nfb_hptr).size;
        };

        // insert new block at HEAD of list
        let nnode = (nfb_hptr as usize + size_of::<Mast>()) as *mut Node;
        push(nnode);
    } else if unsafe { (*p_fptr).a == 0 } && (p_fptr as usize) >= min && (p_fptr as usize) < max {
        // case 3
        let p_bs = unsafe { (*p_fptr).size };

        // ptr to new fb header and boundary tag
        let nfb_hptr =
            (p_fptr as usize - p_bs - size_of::<Node>() - size_of::<Mast>()) as *mut Mast;
        let nfb_fptr = fb_fptr;

        // splice pred out of list
        let pred = (p_fptr as usize - p_bs - size_of::<Node>()) as *mut Node;
        delete_node(pred);

        // coalesce both memory blocks
        unsafe {
            (*nfb_hptr).size =
                (*nfb_hptr).size + (*fb_hptr).size + (2 * size_of::<Mast>()) + size_of::<Node>();
            (*nfb_fptr).size = (*nfb_hptr).size;
        };
        // insert new block at HEAD of list
        let nnode = (nfb_hptr as usize + size_of::<Mast>()) as *mut Node;
        push(nnode);
    } else if unsafe { (*s_hptr).a == 0 } && (s_hptr as usize) >= min && (s_hptr as usize) < max {
        // case 2
        let s_bs = unsafe { (*s_hptr).size };

        // ptr to new fb header and boundary tag
        let nfb_hptr = fb_hptr;
        let nfb_fptr =
            (s_hptr as usize + s_bs + size_of::<Node>() + size_of::<Mast>()) as *mut Mast;

        // splice successor out of the list
        let succ = (s_hptr as usize + size_of::<Mast>()) as *mut Node;
        delete_node(succ);

        // coalesce both memory blocks
        unsafe {
            (*nfb_hptr).size =
                (*nfb_hptr).size + (*nfb_fptr).size + (2 * size_of::<Mast>()) + size_of::<Node>();
            (*nfb_fptr).size = (*nfb_hptr).size;
        };
        // insert new block at HEAD of list
        let nnode = (nfb_hptr as usize + size_of::<Mast>()) as *mut Node;
        push(nnode);
    } else {
        // case 1
        push(free_block as *mut Node);
    }
}

// frees a given region of memory back to the free list.
pub fn my_free(ab: *const c_void) {
    //cast allocated to *mut Mast. adjust ptr to point to header of the ab
    let ab_hptr = (ab as usize - size_of::<Mast>()) as *mut Mast;
    let ab_bs = unsafe { (*ab_hptr).size };

    // ptr to boundary tag of ab
    let ab_fptr = (ab as usize + ab_bs) as *mut Mast;

    // assert to ensure a is indeed equal to 1
    assert_eq!(unsafe { (*ab_hptr).a }, 1);
    assert_eq!(unsafe { (*ab_fptr).a }, 1);

    // adjust ab size and flag
    unsafe {
        (*ab_hptr).size = (*ab_hptr).size - size_of::<Node>();
        (*ab_hptr).a = 0;
        (*ab_fptr).size = (*ab_hptr).size;
        (*ab_fptr).a = 0;
    }
    coalesce(ab);
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_init_and_check_available_memory() {
        reset_heap();
        let size = available_memory();
        assert_eq!(
            size,
            HEAP_SIZE - (2 * size_of::<Mast>()) - size_of::<Node>()
        );
    }

    #[test]
    fn check_number_of_free_node() {
        reset_heap();
        let count = number_of_free_nodes();
        assert_eq!(count, 1);
    }

    #[test]
    fn simple_malloc_call() {
        reset_heap();
        let p = my_malloc(128);
        assert!(p != std::ptr::null_mut());
    }

    #[test]
    fn simple_malloc_call_number_of_free_node() {
        reset_heap();
        let p = my_malloc(128);
        assert!(p != std::ptr::null_mut());
        let count = number_of_free_nodes();
        assert_eq!(count, 1);
    }

    #[test]
    fn malloc_too_much_memory() {
        reset_heap();
        let p = my_malloc(10000);
        assert!(p == std::ptr::null_mut());
    }

    #[test]
    fn simple_malloc_call_check_available_memory() {
        reset_heap();
        let p = my_malloc(128);
        assert!(p != std::ptr::null_mut());
        let size = available_memory();
        assert_eq!(
            size,
            HEAP_SIZE - (4 * size_of::<Mast>()) - size_of::<Node>() - 128
        );
    }

    #[test]
    fn check_your_head() {
        reset_heap();
        let p = my_malloc(128);
        assert!(p != std::ptr::null_mut());

        let head = (p as usize - size_of::<Mast>()) as *mut Mast;

        assert_eq!(unsafe { (*head).a }, 1);
    }

    #[test]
    fn simple_free_check() {
        reset_heap();
        let p = my_malloc(128);
        assert!(p != std::ptr::null_mut());

        let k = my_malloc(128);
        assert!(k != std::ptr::null_mut());

        my_free(p);

        assert_eq!(number_of_free_nodes(), 2);
    }
}