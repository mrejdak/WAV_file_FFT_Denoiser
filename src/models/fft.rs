use std::f64;
use std::f64::consts::PI;

pub fn zero_pad(data: &[f64]) -> Vec<f64> {
    let n = data.len();
    // Check if n is already a power of 2
    // Simple trick (x & (x-1) == 0)
    if (n != 0) && (n & (n - 1) == 0) {
        return data.to_vec();
    }

    let x = n.next_power_of_two();
    [data, &vec![0.0; x - n]].concat()
}

pub fn fft(re: &[f64], im: &[f64]) -> (Vec<f64>, Vec<f64>) {
    // https://en.wikipedia.org/wiki/Cooley%E2%80%93Tukey_FFT_algorithm

    // In order to use fft, the length of input HAS TO BE POWER OF 2
    // Otherwise the algorithm will not work
    // Working with audio it should not be a problem, we may truncate output afterwards

    let n = re.len();

    if (n <= 1) {
        return (re.to_vec(), im.to_vec());
    }

    // Even k's
    let mut re_Ek = Vec::with_capacity(n / 2);
    let mut im_Ek = Vec::with_capacity(n / 2);

    // Odd k's
    let mut re_Ok = Vec::with_capacity(n / 2);
    let mut im_Ok = Vec::with_capacity(n / 2);

    // Functional hell but works
    for (i, (&re_val, &im_val)) in re.iter().zip(im.iter()).enumerate() {
        if i % 2 == 0 {
            re_Ek.push(re_val);
            im_Ek.push(im_val);
        } else {
            re_Ok.push(re_val);
            im_Ok.push(im_val);
        }
    }

    // Perform FFT on Ek's and Ok's
    let (re_Ek_fft, im_Ek_fft) = fft(&re_Ek, &im_Ek);
    let (re_Ok_fft, im_Ok_fft) = fft(&re_Ok, &im_Ok);


    // Here goes the pseudo-code part from wikipedia,
    // visual explanation: https://en.wikipedia.org/wiki/Cooley%E2%80%93Tukey_FFT_algorithm#/media/File:DIT-FFT-butterfly.svg

    // Prepare output vectors
    let mut re_out = [re_Ek_fft, re_Ok_fft].concat();
    let mut im_out = [im_Ek_fft, im_Ok_fft].concat();

    
    for k in 0..n / 2 {
        let re_p = re_out[k];
        let im_p = im_out[k];

        // e^(-2*PI*k/n) = cos(2 * PI * k / n) - isin(2 * PI * k /n)
        // [ cos(2 * PI * k / n) - isin(2 * PI * k /n) ] * (x + yi) ==
        // == xcos() + ysin() + i[ ycos() - xsin() ]
        let angle = 2. * PI * k as f64 / n as f64;
        let re_q = re_out[k + n / 2] * f64::cos(angle) + im_out[k + n / 2] * f64::sin(angle);
        let im_q = -re_out[k + n / 2] * f64::sin(angle) + im_out[k + n / 2] * f64::cos(angle);

        re_out[k] = re_p + re_q;
        re_out[k + n/2] = re_p - re_q;

        im_out[k] = im_p + im_q;
        im_out[k + n/2] = im_p - im_q;
    }

    (re_out, im_out)
}

pub fn ifft(re: &[f64], im: &[f64]) -> (Vec<f64>, Vec<f64>) {
  // https://dsp.stackexchange.com/questions/36082/calculate-ifft-using-only-fft
  // [...] So the recipe is:
  //  - Complex conjugate the given sequence that we want to inverse DFT
  //  - Calculate its forward DFT
  //  - Calculate complex conjugate of the result.
  // That gives you the inverse DFT of the original sequence.

  let n = re.len();

  let im_conj: Vec<f64> = im.iter().map(|&x| -x).collect();

  let (re_fft, im_fft) = fft(&re, &im_conj);

  let re_out =   re_fft.iter().map(|&x| x / n as f64).collect();
  let im_out: Vec<f64> = im_fft.iter().map(|&x| -x / n as f64).collect(); 



  (re_out, im_out)
}

pub fn fft_real(re: &[f64]) -> (Vec<f64>, Vec<f64>) {
  let n = re.len();
  let im: Vec<f64> = vec![0.; n];
  fft(&re, &im)
}

pub fn fft_zero_padded(re: &[f64], im: &[f64]) -> (Vec<f64>, Vec<f64>) {
  let re_pad = zero_pad(&re);
  let im_pad = zero_pad(&im);
  fft(&re_pad, &im_pad)
}

pub fn fft_real_zero_padded(re: &[f64]) -> (Vec<f64>, Vec<f64>) {
  let re_pad = zero_pad(&re);

  let n = re_pad.len();
  let im_pad: Vec<f64> = vec![0.; n];
  fft(&re_pad, &im_pad)
}