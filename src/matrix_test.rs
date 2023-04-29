use nalgebra::{DMatrix, DVector};

pub(crate) fn run() -> Option<()>{

    println!("running matrix text...");

    let mut v : DVector<f64> = DVector::from_element(4, 0.0);

    v[1] = 0.5;

    let mut m : DMatrix<f64> = DMatrix::from_element(4, 4, 1.0);

    let m2 : DMatrix<f64> = DMatrix::zeros(4, 4);

    let m2 : DMatrix<f64> = m2.remove_column(2);
    let m2 : DMatrix<f64> = m2.insert_row(0, 1.0);

    m[(1, 2)] = 7.0;

    println!("{}", m);
    println!("{}", m2);

    v = m * v;

    println!("{}", v);


    None
}