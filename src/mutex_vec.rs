use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::thread_pool::ThreadPool;

pub fn test() {
    println!("--- mutex vec test ---");

    let mutex_ref1 = Arc::new(Mutex::new(test_data(10)));
    //let mutex_ref1 = Arc::new(MutexVec::new(test_data(10)));
    let mutex_ref2 = mutex_ref1.clone();
    let mutex_ref3 = mutex_ref1.clone();
    let mutex_ref4 = mutex_ref1.clone();

    let thread_pool = ThreadPool::new(5);

    // Target - get the threads to access the vec simultaneously.

    thread_pool.execute(move ||{
        println!("[thread#1] START");
        let mut v = mutex_ref2.lock().unwrap();
        for i in evens(v.len()) {
            println!("[thread#1] v[{i}] += 1");
            thread::sleep(Duration::from_millis(100));
            v[i].0 += 1;
        }
        println!("[thread#1] DONE");
    });

    thread_pool.execute(move ||{
        println!("[thread#2] START");
        let mut v = mutex_ref3.lock().unwrap();
        for i in odds(v.len()) {
            println!("[thread#2] v[{i}] += 1");
            thread::sleep(Duration::from_millis(100));
            v[i].0 += 1;
        }
        println!("[thread#2] DONE");
    });

    thread_pool.execute(move ||{
        println!("[thread#3] START");
        let mut v = mutex_ref4.lock().unwrap();
        for i in 0..v.len() {
            println!("[thread#3] v[{i}] += 1");
            thread::sleep(Duration::from_millis(100));
            v[i].0 += 1;
        }
        println!("[thread#3] DONE");
    });

    thread::sleep(Duration::from_millis(3500));


    println!("--------");
    let v = mutex_ref1.lock().unwrap();
    for x in v.iter() {
        println!("{}", x.0);
    }
    println!("--------");
}


mod mutex_vec {
    use std::cell::UnsafeCell;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;
    use std::sync::Mutex;
    use std::thread;

    /// **PROTOTYPE**
    ///
    /// An alternative to `Mutex<Vec<T>>` which allows each index to be locked separately.
    pub struct MutexVec<T> {
        //inner: Mutex<HashSet<usize>>,
        poison: PoisonFlag,
        data: UnsafeCell<Vec<T>>,
    }
    impl<T> MutexVec<T> {
        pub fn new(vec: Vec<T>) -> MutexVec<T> {
            MutexVec {
                //inner: Mutex::new(HashMap::new()),
                poison: PoisonFlag::new(),
                data: UnsafeCell::new(vec),
            }
        }

        pub fn lock(&self, index: &usize) -> LockResult<MutexVecGuard<'_, T>> {
            unsafe {

                // TODO ...
                // self.inner.lock();


                MutexVecGuard::new(self)
            }
        }
    }

    pub type LockResult<Guard> = Result<Guard, PoisonError<Guard>>;

    pub struct MutexVecGuard<'a, T: 'a> {
        lock: &'a MutexVec<T>,
        poison: Guard,
    }
    impl<'mutex, T> MutexVecGuard<'mutex, T> {
        unsafe fn new(lock: &'mutex MutexVec<T>) -> LockResult<MutexVecGuard<'mutex, T>> {
            map_result(lock.poison.guard(), |guard| MutexVecGuard { lock, poison: guard })
        }
    }

    pub struct Guard {
        panicking: bool,
    }

    struct PoisonFlag {
        failed: AtomicBool,
    }
    impl PoisonFlag {
        const fn new() -> PoisonFlag {
            PoisonFlag { failed: AtomicBool::new(false) }
        }

        fn get(&self) -> bool {
            self.failed.load(Ordering::Relaxed)
        }

        fn guard(&self) -> LockResult<Guard> {
            let ret = Guard { panicking: thread::panicking() };
            if self.get() { Err(PoisonError::new(ret)) } else { Ok(ret) }
        }
    }

    pub struct PoisonError<T> {
        guard: T,
    }
    impl<T> PoisonError<T> {
        fn new(guard: T) -> PoisonError<T> {
            PoisonError { guard }
        }
    }

    fn map_result<T, U, F>(result: LockResult<T>, f: F) -> LockResult<U>
        where
            F: FnOnce(T) -> U,
    {
        match result {
            Ok(t) => Ok(f(t)),
            Err(PoisonError { guard }) => Err(PoisonError::new(f(guard))),
        }
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