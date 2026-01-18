#[derive(Debug, Clone, Copy)]
pub struct LoudnessMetrics {
    pub peak: f32,
    pub rms: f32,
    pub crest: f32,
}

#[allow(dead_code)]
pub fn analyze_mono(samples: &[f32]) -> LoudnessMetrics {
    if samples.is_empty() {
        return LoudnessMetrics {
            peak: 0.0,
            rms: 0.0,
            crest: 0.0,
        };
    }

    let mut peak = 0.0f32;
    let mut sum = 0.0f32;

    for &s in samples {
        let abs = s.abs();
        if abs > peak {
            peak = abs;
        }
        sum += s * s;
    }

    let rms = (sum / samples.len() as f32).sqrt();
    let crest = if rms > 0.0 { peak / rms } else { 0.0 };

    LoudnessMetrics { peak, rms, crest }
}

pub fn analyze_stereo(left: &[f32], right: &[f32]) -> LoudnessMetrics {
    let len = left.len().min(right.len());
    if len == 0 {
        return LoudnessMetrics {
            peak: 0.0,
            rms: 0.0,
            crest: 0.0,
        };
    }

    let mut peak = 0.0f32;
    let mut sum = 0.0f32;

    for i in 0..len {
        let l = left[i];
        let r = right[i];
        let abs_l = l.abs();
        let abs_r = r.abs();
        if abs_l > peak {
            peak = abs_l;
        }
        if abs_r > peak {
            peak = abs_r;
        }
        sum += l * l + r * r;
    }

    let rms = (sum / (2.0 * len as f32)).sqrt();
    let crest = if rms > 0.0 { peak / rms } else { 0.0 };

    LoudnessMetrics { peak, rms, crest }
}
