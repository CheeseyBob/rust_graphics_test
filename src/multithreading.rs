
pub fn slice<T>(vec: &Vec<T>, parallelism: usize) -> Vec<&'_[T]> {
    let mut slices = Vec::with_capacity(parallelism);
    let total_length = vec.len();

    for i in 0..parallelism {
        let slice_start = (i * total_length) / parallelism;
        let slice_end = ((i + 1) * total_length) / parallelism;
        let slice_length = slice_end - slice_start;

        let slice_start_pointer = unsafe {
            vec.as_ptr().add(slice_start)
        };

        let slice = unsafe {
            std::slice::from_raw_parts(slice_start_pointer, slice_length)
        };
        slices.push(slice);
    }
    return slices;
}