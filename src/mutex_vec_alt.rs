use std::ops::Deref;
use std::sync::{Arc};
use std::thread;
use std::time::Duration;
use parking_lot::Mutex;
use winit::event::VirtualKeyCode::Mute;
use crate::thread_pool::ThreadPool;


pub fn test() {
    println!("--- mutex_vec_alt test ---");
    let test_data_length = 10;

    let mutex_ref = {
        let test_data = test_data(test_data_length);
        //let mutex_ref = Arc::new(Mutex::new(test_data));
        let mut vec: Vec<Mutex<Data>> = Vec::with_capacity(test_data_length);
        for x in test_data {
            vec.push(Mutex::new(x));
        }
        Arc::new(vec)
    };

    // Target - get the threads to access the vec simultaneously.

    let thread_pool = ThreadPool::new(5);
    execute(1, &thread_pool, &mutex_ref);
    execute(2, &thread_pool, &mutex_ref);
    execute(3, &thread_pool, &mutex_ref);
    execute(4, &thread_pool, &mutex_ref);
    execute(5, &thread_pool, &mutex_ref);

    thread::sleep(Duration::from_millis(3500));

    println!("--------");
    let mut result = String::with_capacity(test_data_length * 2);
    for i in 0..test_data_length {
        let guard = mutex_ref.get(i).unwrap().lock();
        let data = guard.deref();
        result.push_str(data.0.to_string().as_str());
        result.push(',');
    }
    println!("{}", &result);
    println!("--------");
    //assert_eq!(result, "5,1,2,2,3,2,3,1,3,2,");
}

fn execute(n: usize, thread_pool: &ThreadPool, mutex_ref: &Arc<Vec<Mutex<Data>>>) {
    let ref_clone = mutex_ref.clone();
    thread_pool.execute(move ||{
        println!("({n}) START");
        let mut i = 0;
        let length = ref_clone.len();
        while i < length {
            let mut guarded_data = ref_clone.get(i).unwrap().lock();
            println!("({n}) v[{i}] += 1");
            thread::sleep(Duration::from_millis(100));
            guarded_data.0 += 1;
            i += n;
        }
        println!("({n}) DONE");
    });
}


struct MutexVec<T> {
    raw: Vec<Mutex<T>>
}
impl<T> MutexVec<T> {
    pub fn new<F>(capacity: usize, mut f: F) -> MutexVec<T>
        where F: FnMut() -> T {
        let mut raw: Vec<Mutex<T>> = Vec::with_capacity(capacity);
        raw.resize_with(capacity, || Mutex::new(f()));
        MutexVec { raw }
    }
}



#[derive(Debug)]
struct Data(usize);

fn test_data(size: usize) -> Vec<Data> {
    let mut v: Vec<Data> = Vec::with_capacity(size);
    for _ in 0..size {
        v.push(Data(0));
    }
    return v;
}

fn evens(n: usize) -> Vec<usize> {
    let mut v = Vec::new();
    let mut i = 0;
    while i < n {
        v.push(i);
        i += 2;
    }
    return v;
}

fn odds(n: usize) -> Vec<usize> {
    let mut v = Vec::new();
    let mut i = 1;
    while i < n {
        v.push(i);
        i += 2;
    }
    return v;
}