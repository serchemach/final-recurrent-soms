use meshgridrs::{meshgrid, Indexing};
use ndarray::{array, s, stack, Array, ArrayBase, Axis, Dim};
use ndarray_stats::QuantileExt;
use tqdm::tqdm;

const n: usize = 40;
const m: usize = 50;
const map_input_size: usize = 3;
const TRAIN_ITERATIONS: usize = 1000;
const LEARNING_RATE: f64 = 0.1;
const gauss_width_squared: f64 = 40.0 * 40.0;
const a: f64 = 1.0;
const b: f64 = 2.0; // Values for stability or smth
const time_constant: f64 = 5000.0;
const gamma: f64 = 0.5;

fn train_som(
    som: &mut ArrayBase<ndarray::OwnedRepr<f64>, Dim<[usize; 3]>>,
    dataset: &Vec<ArrayBase<ndarray::OwnedRepr<f64>, Dim<[usize; 1]>>>,
    train_iterations: usize,
    learning_rate: f64,
) {
    let grid_ = meshgrid(
        &vec![
            Array::range(0.0, m as f64, 1.0),
            Array::range(0.0, n as f64, 1.0),
        ],
        Indexing::Xy,
    )
    .unwrap();
    let grid = stack![Axis(2), grid_[1], grid_[0]];
    // println!("{grid:?}");
    for i in tqdm(0..train_iterations) {

        for sample in dataset {
            let diff = -(som.clone() - sample);
            let errs = (&diff * &diff).sum_axis(Axis(2));
            
            let best_unit_coords = errs.argmin().unwrap();
            
            let shifted_grid = &grid - &array![best_unit_coords.0 as f64, best_unit_coords.1 as f64];
            let distances = (&shifted_grid * &shifted_grid).sum_axis(Axis(2));
            
            let neighbourhood_func_values = distances.mapv_into(|v| (-v/gauss_width_squared).exp())
            .insert_axis(Axis(2));
        
            // println!("------------------------0-------------------");
            *som += &(learning_rate * &neighbourhood_func_values * &diff.slice(s![best_unit_coords.0, best_unit_coords.1, ..]));
        }
    }
}

fn evaluate_som(som: &ArrayBase<ndarray::OwnedRepr<f64>, Dim<[usize; 3]>>, sample: &ArrayBase<ndarray::OwnedRepr<f64>, Dim<[usize; 1]>>) -> (usize, usize){
    let diff = -(som - sample);
    let errs = (&diff * &diff).sum_axis(Axis(1));
    
    let best_unit_coords = errs.argmin().unwrap();

    return best_unit_coords;
}