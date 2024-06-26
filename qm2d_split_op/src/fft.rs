use crate::constants::*;
use crate::complex::*;

pub fn square_transpose_in_place<T: Copy>(array: &mut [Complex<T>], n: usize) {
    for i in 0..n {
        for j in i+1..n {
            let tmp = array[i*n + j];
            array[i*n + j] = array[j*n + i];
            array[j*n + i] = tmp;
        }
    }
}

/* Reverse bit sort an array, where the size of the array
must be a power of two.
*/
fn reverse_bit_sort<T: Copy>(array: &mut [Complex<T>], n: usize) {
    let mut u: usize;
    let mut d: usize;
    let mut rev: usize;
    for i in 0..n {
        u = 1;
        d = n >> 1;
        rev = 0;
        while u < n {
            rev += d*((i&u)/u);
            u <<= 1;
            d >>= 1;
        }
        if rev >= i {
            let tmp = array[i];
            array[i] = array[rev];
            array[rev] = tmp;
        } 
    }
}

/* This function implements the iterative in place radix-2 
Cooley-Turkey Fast Fourier Transform Algorithm. The size of the input
array must be a power of two, or else bad things will happen. There
are currently no checks done to ensure this.

References:

Wikipedia - Cooley–Tukey FFT algorithm
https://en.wikipedia.org/wiki/Cooley%E2%80%93Tukey_FFT_algorithm

MathWorld Wolfram - Fast Fourier Transform:
http://mathworld.wolfram.com/FastFourierTransform.html 

William Press et al.
12.2 Fast Fourier Transform (FFT) - Numerical Recipes
https://websites.pmc.ucsc.edu/~fnimmo/eart290c_17/NumericalRecipesinF77.pdf

*/
pub fn base_f32_fft_in_place(array: &mut [Complex<f32>], 
                        size: usize, is_inverse: bool) {
    reverse_bit_sort(array, size);
    let mut block_size: usize = 2;
    while block_size <= size {
        let mut j: usize = 0;
        while j < size {
            for i in 0..block_size/2 {
                let sgn: f64 = if is_inverse {-1.0} else {1.0};
                let e: Complex<f64> = Complex {
                    real: f64::cos(2.0*std::f64::consts::PI
                                *(i as f64)/(block_size as f64)),
                    imag: sgn*f64::sin(2.0*std::f64::consts::PI
                                    *(i as f64)/(block_size as f64)),
                };
                let even: Complex<f64> = array[j + i].into();
                let odd: Complex<f64> = array[j + i + block_size/2].into();
                let s: f64 = if is_inverse && block_size == size 
                    {1.0/(size as f64)} else {1.0};
                array[j + i] = (even + odd*e).scale(s).into();
                array[j + i + block_size/2] = (even - odd*e).scale(s).into();
            }
            j += block_size;
        }
        block_size *= 2;
    }
}

pub fn fft_in_place(array: &mut [Complex<f32>], size: usize) {
    base_f32_fft_in_place(array, size, false);
}

pub fn ifft_in_place(array: &mut [Complex<f32>], size: usize) {
    base_f32_fft_in_place(array, size, true);
}

/* Perform the fft algorithm on each row of an array.
Rows are placed into separate groups, where each group is
handled by its own thread.

Multithreading reference:
https://doc.rust-lang.org/book/ch16-01-threads.html
https://doc.rust-lang.org/book/ch16-02-message-passing.html
*/
pub fn horizontal_square_fft(is_inverse: bool, array: &mut [Complex<f32>]) {
    let mut receivers = std::vec::Vec::<
        std::sync::mpsc::Receiver<std::vec::Vec<Complex<f32>>>
        >::with_capacity(TH_COUNT);
    for th_index in 0..TH_COUNT {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut v
            = std::vec::Vec::<Complex<f32>>::with_capacity(N*N/TH_COUNT);
        for i in th_index*N*N/TH_COUNT..(th_index + 1)*N*N/TH_COUNT {
            v.push(array[i]);
        }
        std::thread::spawn(move || {
            for i in 0..N/TH_COUNT {
                if is_inverse {
                    ifft_in_place(&mut v.as_mut_slice()[i*N..(i+1)*N], N);
                } else {
                    fft_in_place(&mut v.as_mut_slice()[i*N..(i+1)*N], N);
                }
            }
            tx.send(v).unwrap();
        });
        receivers.push(rx);
    }
    /* let mut vec_vec = std::vec::Vec::<
        std::vec::Vec<Complex<f32>>
        >::with_capacity(TH_COUNT);
    for i in 0..TH_COUNT {
        vec_vec.push(std::vec::Vec
            <Complex<f32>>::with_capacity(N*N/TH_COUNT));
    }*/
    let mut th_index: usize = TH_COUNT - 1;
    while let Some(r) = receivers.pop() {
        let v = r.recv().unwrap();
        for i in th_index*N/TH_COUNT..(th_index + 1)*N/TH_COUNT {
            let i_get = i - th_index*N/TH_COUNT;
            for j in 0..N {
                // let transpose_index: usize = j*N + i;
                // let src_val = v[i_get*N + j];
                array[N*i + j] = v[i_get*N + j];
            }
        }
        th_index = if th_index == 0 {th_index} else {th_index-1};
    }
}
