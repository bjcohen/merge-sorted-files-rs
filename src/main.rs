use std::env;
use std::fs;
use std::io;

use merge_sorted_files_rs::*;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut heap = Heap::new();
    for filename in &args[1..] {
        add_file_to_heap(&mut heap, filename.to_string())?;
    }
    heap.print_sorted_lines()?;
    Ok(())
}

fn add_file_to_heap(heap: &mut Heap<fs::File>, filename: String) -> io::Result<()> {
    let f = fs::File::open(&filename)?;
    heap.add_reader(filename, f)
}
