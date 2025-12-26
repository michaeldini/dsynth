#!/usr/bin/env python3
"""
DSynth Wavetable Generator
Generates 20 high-quality single-cycle wavetables (2048 samples each)
- 10 classic/essential waveforms
- 10 unique/experimental waveforms
"""

import numpy as np
from scipy.io import wavfile
import os

# Configuration
SAMPLES_PER_CYCLE = 2048
SAMPLE_RATE = 44100
OUTPUT_DIR = "assets/wavetables"

def normalize(samples):
    """Normalize samples to -1.0 to +1.0 range"""
    max_val = np.max(np.abs(samples))
    if max_val > 0:
        return samples / max_val
    return samples

def save_wavetable(name, samples):
    """Save wavetable as 32-bit float WAV file"""
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    samples = normalize(samples).astype(np.float32)
    filepath = os.path.join(OUTPUT_DIR, f"{name}.wav")
    wavfile.write(filepath, SAMPLE_RATE, samples)
    print(f"✓ Generated: {name}.wav ({len(samples)} samples)")

# Phase array for all generators
phase = np.linspace(0, 1, SAMPLES_PER_CYCLE, endpoint=False)

# ============================================================================
# CLASSIC WAVETABLES (10)
# ============================================================================

def generate_classic():
    """Generate 10 essential classic waveforms"""
    
    # 1. Pure Sine (fundamental building block)
    sine = np.sin(2 * np.pi * phase)
    save_wavetable("01_Pure_Sine", sine)
    
    # 2. Sawtooth (bright, rich in harmonics)
    sawtooth = 2 * phase - 1
    save_wavetable("02_Sawtooth", sawtooth)
    
    # 3. Square (odd harmonics only)
    square = np.where(phase < 0.5, 1.0, -1.0)
    save_wavetable("03_Square", square)
    
    # 4. Triangle (softer than square)
    triangle = np.where(phase < 0.5, 4*phase - 1, 3 - 4*phase)
    save_wavetable("04_Triangle", triangle)
    
    # 5. PWM 25% (thin pulse)
    pwm_25 = np.where(phase < 0.25, 1.0, -1.0)
    save_wavetable("05_PWM_25", pwm_25)
    
    # 6. PWM 12.5% (very thin, nasal)
    pwm_12 = np.where(phase < 0.125, 1.0, -1.0)
    save_wavetable("06_PWM_12", pwm_12)
    
    # 7. Classic PPG (additive with specific harmonics)
    ppg = (np.sin(2*np.pi*phase) + 
           0.5*np.sin(4*np.pi*phase) + 
           0.3*np.sin(6*np.pi*phase) +
           0.2*np.sin(8*np.pi*phase))
    save_wavetable("07_PPG_Classic", ppg)
    
    # 8. Hollow (missing fundamental)
    hollow = (0.7*np.sin(4*np.pi*phase) + 
              0.5*np.sin(6*np.pi*phase) + 
              0.3*np.sin(8*np.pi*phase))
    save_wavetable("08_Hollow", hollow)
    
    # 9. Vocal Formant (bright "ah" sound)
    formant = (np.sin(2*np.pi*phase) + 
               0.8*np.sin(4*np.pi*phase) + 
               1.2*np.sin(6*np.pi*phase) +
               0.6*np.sin(10*np.pi*phase))
    save_wavetable("09_Vocal_Formant", formant)
    
    # 10. Bass Guitar (few harmonics, round)
    bass = (np.sin(2*np.pi*phase) + 
            0.3*np.sin(4*np.pi*phase) + 
            0.1*np.sin(6*np.pi*phase))
    save_wavetable("10_Bass_Guitar", bass)

# ============================================================================
# UNIQUE/EXPERIMENTAL WAVETABLES (10)
# ============================================================================

def generate_unique():
    """Generate 10 creative and unusual waveforms"""
    
    # 11. Stairs (quantized sine - digital glitchy)
    stairs = np.floor(np.sin(2 * np.pi * phase) * 8) / 8
    save_wavetable("11_Stairs", stairs)
    
    # 12. Fractal (chaotic harmonics with fibonacci ratios)
    fractal = (np.sin(2*np.pi*phase) + 
               0.618*np.sin(3*np.pi*phase) + 
               0.382*np.sin(5*np.pi*phase) +
               0.236*np.sin(8*np.pi*phase) +
               0.146*np.sin(13*np.pi*phase))
    save_wavetable("12_Fractal", fractal)
    
    # 13. Crystal (metallic, inharmonic overtones)
    crystal = (np.sin(2*np.pi*phase) + 
               0.7*np.sin(5.1*np.pi*phase) + 
               0.5*np.sin(8.3*np.pi*phase) +
               0.3*np.sin(12.7*np.pi*phase))
    save_wavetable("13_Crystal", crystal)
    
    # 14. Earthquake (sub-bass wobble with noise)
    earthquake = (np.sin(2*np.pi*phase) + 
                  0.3*np.sin(0.5*np.pi*phase) +
                  0.1*np.random.uniform(-1, 1, SAMPLES_PER_CYCLE))
    save_wavetable("14_Earthquake", earthquake)
    
    # 15. Laser (harsh, metallic, sci-fi)
    laser_mod = 1 + 0.5*np.sin(32*np.pi*phase)
    laser = np.sin(2*np.pi*phase) * laser_mod
    save_wavetable("15_Laser", laser)
    
    # 16. Vowel Morph (transitions through vowel sounds)
    # Start with "ee", morph to "ah", end with "oo"
    vowel = np.zeros(SAMPLES_PER_CYCLE)
    for i, p in enumerate(phase):
        if p < 0.33:
            # "ee" - high formants
            vowel[i] = np.sin(2*np.pi*p) + 0.8*np.sin(8*np.pi*p)
        elif p < 0.66:
            # "ah" - mid formants
            vowel[i] = np.sin(2*np.pi*p) + 1.2*np.sin(4*np.pi*p)
        else:
            # "oo" - low formants
            vowel[i] = np.sin(2*np.pi*p) + 0.3*np.sin(3*np.pi*p)
    save_wavetable("16_Vowel_Morph", vowel)
    
    # 17. Binary (1-bit digital, extreme)
    binary = np.sign(np.sin(2*np.pi*phase) + 0.5*np.sin(3*np.pi*phase))
    save_wavetable("17_Binary", binary)
    
    # 18. Alien (dissonant, otherworldly)
    alien = (np.sin(2*np.pi*phase) + 
             0.8*np.sin(2.7*np.pi*phase) + 
             0.6*np.sin(4.3*np.pi*phase) +
             0.4*np.sin(7.1*np.pi*phase) +
             0.2*np.sin(11.6*np.pi*phase))
    save_wavetable("18_Alien", alien)
    
    # 19. Breath (noise-based, airy texture)
    breath_noise = np.random.uniform(-1, 1, SAMPLES_PER_CYCLE)
    # Low-pass filter the noise for breath-like quality
    from scipy.signal import butter, filtfilt
    b, a = butter(4, 0.1)
    breath = filtfilt(b, a, breath_noise)
    save_wavetable("19_Breath", breath)
    
    # 20. Harmonic Decay (harmonics fade with time)
    harmonic_decay = np.zeros(SAMPLES_PER_CYCLE)
    for i, p in enumerate(phase):
        decay_factor = 1 - p  # Fade from 1 to 0
        harmonic_decay[i] = (
            np.sin(2*np.pi*p) * decay_factor +
            0.5*np.sin(4*np.pi*p) * (decay_factor ** 2) +
            0.3*np.sin(6*np.pi*p) * (decay_factor ** 3) +
            0.2*np.sin(8*np.pi*p) * (decay_factor ** 4)
        )
    save_wavetable("20_Harmonic_Decay", harmonic_decay)

# ============================================================================
# MAIN EXECUTION
# ============================================================================

if __name__ == "__main__":
    print("\n" + "="*60)
    print("  DSynth Wavetable Generator")
    print("  Generating 20 wavetables (2048 samples @ 44.1kHz)")
    print("="*60 + "\n")
    
    print("📦 Classic Wavetables (10):")
    print("-" * 60)
    generate_classic()
    
    print("\n✨ Unique/Experimental Wavetables (10):")
    print("-" * 60)
    generate_unique()
    
    print("\n" + "="*60)
    print(f"✅ Complete! 20 wavetables saved to: {OUTPUT_DIR}/")
    print("="*60)
    print("\nUsage:")
    print("  1. Run: cargo build --release")
    print("  2. Launch DSynth")
    print("  3. Set oscillator waveform to 'Wavetable'")
    print("  4. Turn 'Wavetable' knob (0-19) to select different waves")
    print("  5. Use 'Position' knob for future morphing feature")
    print("\nWavetable Index Reference:")
    print("  Classic:      0-9  (Sine, Saw, Square, etc.)")
    print("  Experimental: 10-19 (Stairs, Fractal, Crystal, etc.)")
    print("\n")
