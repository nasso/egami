use image::{imageops, GenericImageView, ImageBuffer, Pixel};
use rayon::prelude::*;
use std::ops::{Mul, Neg};
use std::sync::{Arc, Mutex};

// yeah this is some kind of magic number tbh; it gives good results so we're using that
const SIMILARITY_EXP_COEFFICIENT: f64 = 100.0;

#[derive(Clone)]
pub struct ChannelHistogram {
    pub data: Vec<f64>,
    pub average: f64,
}

impl ChannelHistogram {
    pub fn new(size: usize) -> Self {
        ChannelHistogram {
            data: vec![0.0; size],
            average: 0.0,
        }
    }

    pub fn similarity(&self, other: &ChannelHistogram) -> f64 {
        // possible improvement: support computing similarity for histograms with different sizes
        assert_eq!(
            self.data.len(),
            other.data.len(),
            "can't compute similarity: histograms have different size ({} and {})",
            self.data.len(),
            other.data.len()
        );

        // computes e^(-sx)
        // where:
        //   s = some constant
        //   x = the sum of squared differences

        self.data
            .iter()
            // zip the two iterators
            .zip(other.data.iter())
            // compute the squared difference for each value
            .map(|(x, y)| (x - y).powi(2))
            // sum everything
            .sum::<f64>()
            // negate the sum
            .neg()
            // multiply it by some coefficient (otherwise pretty much everything is close to 1)
            .mul(SIMILARITY_EXP_COEFFICIENT)
            // take the exponent
            .exp()
    }

    fn compute_average(&mut self) {
        self.average = self
            .data
            .iter()
            .enumerate()
            .map(|(tone, count)| tone as f64 * count)
            .sum::<f64>()
            / self.data.iter().sum::<f64>();
    }

    pub fn maximize(&mut self) {
        let max = self.data.iter().cloned().fold(std::f64::NAN, f64::max);

        for v in self.data.iter_mut() {
            *v /= max;
        }

        // gotta compute the average again
        self.compute_average();
    }

    pub fn maximized(mut self) -> ChannelHistogram {
        self.maximize();
        self
    }

    pub fn normalize(&mut self) {
        let pixel_count = self.data.iter().sum::<f64>();

        for v in self.data.iter_mut() {
            *v /= pixel_count;
        }

        // gotta compute the average again
        self.compute_average();
    }

    pub fn normalized(mut self) -> ChannelHistogram {
        self.normalize();
        self
    }
}

impl std::fmt::Debug for ChannelHistogram {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "ChannelHistogram {{ data: {:?}, average: {:?} }}",
            &self.data[..],
            self.average
        )
    }
}

#[derive(Debug)]
pub struct HistogramSimilarity {
    pub average: f64,
    pub channels: Vec<f64>,
}

#[derive(Debug)]
pub struct Histogram {
    pub channels: Vec<ChannelHistogram>,
}

impl Histogram {
    pub fn new(channel_count: usize, size: usize) -> Self {
        Histogram {
            channels: vec![ChannelHistogram::new(size); channel_count],
        }
    }

    pub fn similarity(&self, other: &Histogram) -> HistogramSimilarity {
        let channels: Vec<_> = self
            .channels
            .iter()
            .zip(other.channels.iter())
            .map(|(s, o)| s.similarity(o))
            .collect();

        HistogramSimilarity {
            average: channels.iter().sum::<f64>() / channels.len() as f64,
            channels,
        }
    }

    pub fn maximize(&mut self) {
        for c in self.channels.iter_mut() {
            c.maximize();
        }
    }

    pub fn maximized(mut self) -> Histogram {
        self.maximize();
        self
    }

    pub fn normalize(&mut self) {
        for c in self.channels.iter_mut() {
            c.normalize();
        }
    }

    pub fn normalized(mut self) -> Histogram {
        self.normalize();
        self
    }
}

pub fn histogram<I, P>(image: &I, size: usize) -> Histogram
where
    I: GenericImageView<Pixel = P> + Sync,
    P: Pixel<Subpixel = u8> + 'static,
{
    let counter = Arc::new(Mutex::new(Histogram::new(3, size)));

    let w = image.width();
    let h = image.height();

    // we manually iterate over all coordinates to use rayon's ParallelIterator

    // iterator over the x axis
    (0..w)
        .into_par_iter()
        .for_each_with(counter.clone(), |counter, x| {
            // iterator over the y axis
            (0..h)
                .into_par_iter()
                .for_each_with(counter.clone(), |counter, y| {
                    // get the pixel's RGB values at (x; y)
                    let rgb = image.get_pixel(x, y).to_rgb().0;

                    let ri = (rgb[0] as f64 / 256.0 * size as f64) as usize;
                    let gi = (rgb[1] as f64 / 256.0 * size as f64) as usize;
                    let bi = (rgb[2] as f64 / 256.0 * size as f64) as usize;

                    {
                        let mut hist = counter.lock().unwrap();

                        hist.channels[0].data[ri] += 1.0;
                        hist.channels[1].data[gi] += 1.0;
                        hist.channels[2].data[bi] += 1.0;
                    }
                });
        });

    let mut hist = Arc::try_unwrap(counter).unwrap().into_inner().unwrap();

    // Compute averages and normalize
    hist.normalize();

    hist
}

pub fn fingerprint<I, P>(image: &I, size: u32) -> ImageBuffer<P, Vec<u8>>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = u8> + 'static,
{
    imageops::thumbnail(image, size, size)
}
