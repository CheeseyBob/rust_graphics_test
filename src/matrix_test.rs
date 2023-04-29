use std::ops::Mul;
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

    // ----------------------

    let input_layer_size: usize = 5;
    let output_layer_size: usize = 3;

    let mut net = NeuralNet::new(input_layer_size, output_layer_size);
    net.connections[(0, 0)] = 0.5;
    net.bias[0] = 0.5;

    let mut input : DVector<f64> = DVector::zeros(input_layer_size);
    input[0] = 0.5;

    let output = net.apply(&input);

    println!("--- output ---");
    println!("{}", output);



    // ---------------

    None
}

struct NeuralNet {
    input_layer_size: usize,
    output_layer_size: usize,
    connections: DMatrix<f64>,
    bias: DVector<f64>,
}

impl NeuralNet {
    fn new(input_layer_size: usize, output_layer_size: usize) -> NeuralNet {
        NeuralNet {
            connections: DMatrix::zeros(output_layer_size, input_layer_size), // Check if this is the right way around.
            bias: DVector::zeros(output_layer_size),
            input_layer_size,
            output_layer_size,
        }
    }

    fn apply(&self, input: &DVector<f64>) -> DVector<f64> {
        assert_eq!(input.len(), self.input_layer_size);

        let connections = &self.connections;
        let bias = &self.bias;

        let mut output = connections * input;
        output = output + bias;

        // TODO - normalise.

        return DVector::from_data(output.data);
    }
}