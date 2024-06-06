use core::prelude;
use meshgridrs::{meshgrid, Indexing};
use ndarray::{prelude::*, stack, Dimension, OwnedRepr};
use ndarray_ndimage::{pad, PadMode};
use ndarray_npy::NpzWriter;
use ndarray_stats::{QuantileExt, SummaryStatisticsExt};
use std::cmp;
use std::collections::VecDeque;
use std::fs::File;
use tqdm::tqdm;
use serde::{Serialize, Deserialize};

pub fn get_vec_median(samples: &Vec<ArrayView1<f32>>) -> Array1<f32> {
    let mut cur_median = Array1::zeros(0);

    for sample in samples {
        let new_len = sample.len();
        let cur_len = cur_median.len();
        println!("cur_len {cur_len}, new_len {new_len}");

        if new_len > cur_len {
            cur_median = &pad(
                &cur_median,
                &[[0, new_len - cur_len]],
                PadMode::Constant(0.0),
            ) + sample;
        } else if new_len < cur_len {
            cur_median += &pad(sample, &[[0, cur_len - new_len]], PadMode::Constant(0.0));
        } else {
            cur_median += sample;
        }
    }

    cur_median / (samples.len() as f32)
}

pub fn get_vec_std(samples: &Vec<ArrayView1<f32>>) -> f32 {
    println!("Before Median");
    let median = get_vec_median(samples);
    let med_len = median.len();

    let mut diff_sq_sum: Array1<f32> = Array1::zeros(median.len());

    for sample in samples {
        println!("Sample {sample}, median {median}");
        let cur_len = sample.len();
        let diff = &pad(sample, &[[0, med_len - cur_len]], PadMode::Minimum) - &median;

        diff_sq_sum += &(&diff * &diff);
    }

    diff_sq_sum.sum().sqrt() / (samples.len() as f32)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MSOM {
    pub n: usize,
    pub m: usize,
    pub map_input_size: usize,
    pub a: f32,
    pub b: f32,
    pub gamma: f32,

    som: ArrayBase<OwnedRepr<f32>, Dim<[usize; 3]>>,
    context: ArrayBase<OwnedRepr<f32>, Dim<[usize; 3]>>,
}

impl MSOM {
    pub fn new(n: usize, m: usize, map_input_size: usize, a: f32, b: f32, gamma: f32) -> MSOM {
        MSOM {
            n,
            m,
            map_input_size,
            som: ArrayBase::zeros((n, m, map_input_size)),
            context: ArrayBase::zeros((n, m, map_input_size)),
            gamma,
            a,
            b,
        }
    }

    pub fn fit(
        &mut self,
        dataset: &Vec<ArrayView1<f32>>,
        train_iterations: usize,
        learning_rate_base: f32,
        gauss_width_squared_base: f32,
        time_constant: f32,
    ) {
        let grid_ = meshgrid(
            &vec![
                Array::range(0.0, self.m as f32, 1.0),
                Array::range(0.0, self.n as f32, 1.0),
            ],
            Indexing::Xy,
        )
        .unwrap();
        let grid = stack![Axis(2), grid_[1], grid_[0]];

        // println!("{grid:?}");
        for i in tqdm(0..train_iterations) {
            let learning_rate = learning_rate_base * (-(i as f32) / time_constant).exp();
            let gauss_width_squared =
                gauss_width_squared_base * (-(i as f32) / time_constant).exp();
            let temp_a = self.a;

            for sample in dataset {
                let mut previous_w: ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 1]>> =
                    ArrayBase::zeros(self.map_input_size);
                let mut previous_c: ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 1]>> =
                    ArrayBase::zeros(self.map_input_size);

                for chunk_pos in (0..sample.len()).step_by(self.map_input_size) {
                    self.a *= 0.99;

                    let chunk = sample.slice(s![chunk_pos..(chunk_pos + self.map_input_size)]);

                    let cur_diff = -(self.som.clone() - chunk);
                    let cur_errs = (&cur_diff * &cur_diff).sum_axis(Axis(2));
                    let prev_diff = -(self.context.clone()
                        - (self.gamma * previous_w + (1.0 - self.gamma) * previous_c));
                    let prev_errs = (&prev_diff * &prev_diff).sum_axis(Axis(2));
                    let errs = self.a * cur_errs + self.b * prev_errs;

                    let best_unit_coords = errs.argmin().unwrap();

                    let shifted_grid =
                        &grid - &array![best_unit_coords.0 as f32, best_unit_coords.1 as f32];
                    let distances = (&shifted_grid * &shifted_grid).sum_axis(Axis(2));

                    let neighbourhood_func_values = distances
                        .mapv_into(|v| (-v / gauss_width_squared).exp())
                        .insert_axis(Axis(2));

                    // println!("------------------------0-------------------");
                    self.som += &(learning_rate
                        * &neighbourhood_func_values
                        * &cur_diff.slice(s![best_unit_coords.0, best_unit_coords.1, ..]));
                    self.context += &(learning_rate
                        * &neighbourhood_func_values
                        * &prev_diff.slice(s![best_unit_coords.0, best_unit_coords.1, ..]));

                    previous_w = self
                        .som
                        .slice(s![best_unit_coords.0, best_unit_coords.1, ..])
                        .to_owned();
                    previous_c = self
                        .context
                        .slice(s![best_unit_coords.0, best_unit_coords.1, ..])
                        .to_owned();
                }
                self.a = temp_a;
            }
        }
    }

    pub fn evaluate(&self, sample: ArrayView1<f32>) -> (usize, usize) {
        let mut best_unit_coords = (0, 0);

        let mut previous_w: ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 1]>> =
            ArrayBase::zeros(self.map_input_size);
        let mut previous_c: ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 1]>> =
            ArrayBase::zeros(self.map_input_size);

        let mut cur_a = self.a;
        for chunk_pos in (0..sample.len()).step_by(self.map_input_size) {
            cur_a *= 0.99;
            let chunk = sample.slice(s![chunk_pos..(chunk_pos + self.map_input_size)]);

            let cur_diff = -(self.som.clone() - chunk);
            let cur_errs = (&cur_diff * &cur_diff).sum_axis(Axis(2));
            let prev_diff = -(self.context.clone()
                - (self.gamma * previous_w + (1.0 - self.gamma) * previous_c));
            let prev_errs = (&prev_diff * &prev_diff).sum_axis(Axis(2));
            let errs = cur_a * cur_errs + self.b * prev_errs;

            best_unit_coords = errs.argmin().unwrap();

            previous_w = self
                .som
                .slice(s![best_unit_coords.0, best_unit_coords.1, ..])
                .to_owned();
            previous_c = self
                .context
                .slice(s![best_unit_coords.0, best_unit_coords.1, ..])
                .to_owned();
        }

        return best_unit_coords;
    }

    pub fn dump_to_npz(&self, path: &str) {
        let mut npz = NpzWriter::new(File::create(path).unwrap());
        npz.add_array("som", &self.som).unwrap();
        npz.add_array("context", &self.context).unwrap();
        npz.finish().unwrap();
    }

    pub fn reception_field_averaged(
        &self,
        samples: &Vec<ArrayView1<f32>>,
    ) -> Vec<Vec<Array1<f32>>> {
        let mut counts = vec![vec![0; self.m]; self.n];
        let mut vector_sums: Vec<Vec<Array1<f32>>> = vec![vec![Array1::zeros(0); self.m]; self.n];
        for sample in tqdm(samples.iter()) {
            let prediction = self.evaluate(*sample);
            counts[prediction.0][prediction.1] += 1;

            let new_len = sample.len();
            let cur_len = vector_sums[prediction.0][prediction.1].len();

            if new_len > cur_len {
                vector_sums[prediction.0][prediction.1] = &pad(
                    &vector_sums[prediction.0][prediction.1],
                    &[[0, new_len - cur_len]],
                    PadMode::Minimum,
                ) + sample;
            } else if new_len < cur_len {
                vector_sums[prediction.0][prediction.1] +=
                    &pad(sample, &[[0, new_len - cur_len]], PadMode::Minimum);
            } else {
                vector_sums[prediction.0][prediction.1] += sample;
            }
        }

        for row_i in 0..self.n {
            for vector_sum_i in 0..self.m {
                vector_sums[row_i][vector_sum_i] /= cmp::max(1, counts[row_i][vector_sum_i]) as f32;
            }
        }

        vector_sums
    }

    pub fn reception_field_count(&self, samples: &Vec<ArrayView1<f32>>) -> Array2<usize> {
        let mut counts = Array2::zeros((self.n, self.m));
        for sample in tqdm(samples.into_iter()) {
            let prediction = self.evaluate(sample.view());
            counts[(prediction.0, prediction.1)] += 1;
        }

        counts
    }

    pub fn quantization_error(&self, samples: &Vec<ArrayView1<f32>>) -> Array2<f32> {
        let mut vector_occurences: Vec<Vec<Vec<ArrayView1<f32>>>> =
            vec![vec![vec![]; self.m]; self.n];
        for sample in tqdm(samples.iter()) {
            let prediction = self.evaluate(*sample);
            vector_occurences[prediction.0][prediction.1].push(*sample);
        }

        let mut quantization_errors: Array2<f32> = Array2::zeros((self.n, self.m));
        for row_i in 0..self.n {
            for col_i in 0..self.m {
                let vec_array1 = &vector_occurences[row_i][col_i];
                if vec_array1.len() == 0 {
                    continue;
                }

                let cur_std = get_vec_std(vec_array1);
                quantization_errors[(row_i, col_i)] = cur_std;
                if cur_std.is_nan() {
                    println!("FUUUUUUUUUUUUUUuuCk {row_i} {col_i}");
                }

                // let cur_arr: Array2<f32> = vector_occurences[row_i][col_i];
            }
        }

        quantization_errors
    }
}
