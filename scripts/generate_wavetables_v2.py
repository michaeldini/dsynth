#!/usr/bin/env python3
"""
DSynth Advanced Wavetable Generator v2.0
Generates 20 high-quality wavetables with complex spectral content
"""

import numpy as np
from scipy.signal import butter, filtfilt
import os

# Configuration
OUTPUT_DIR = "assets/wavetables"
SAMPLES_PER_CYCLE = 2048
SAMPLE_RATE = 44100

def save_wavetable(filename, samples):
    """Save normalized wavetable as 32-bit float WAV"""
    # Normalize to ±1.0
    samples = samples / (np.max(np.abs(samples)) + 1e-10)
    
    # Convert to 32-bit float WAV
    import wave
    import struct
    
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    filepath = os.path.join(OUTPUT_DIR, f"{filename}.wav")
    
    with wave.open(filepath, 'w') as wav:
        wav.setnchannels(1)  # Mono
        wav.setsampwidth(4)   # 32-bit = 4 bytes
        wav.setframerate(SAMPLE_RATE)
        wav.setnframes(len(samples))
        wav.setcomptype('NONE', 'not compressed')
        
        # Write as 32-bit float
        for sample in samples:
            wav.writeframes(struct.pack('<f', sample))
    
    print(f"✓ Generated: {filename}.wav ({len(samples)} samples)")

# ============================================================================
# CLASSIC WAVETABLES (10) - unchanged
# ============================================================================

def generate_classic():
    """Generate 10 classic waveforms"""
    phase = np.linspace(0, 1, SAMPLES_PER_CYCLE, endpoint=False)
    
    # 1. Pure Sine
    sine = np.sin(2 * np.pi * phase)
    save_wavetable("01_Pure_Sine", sine)
    
    # 2. Sawtooth
    saw = 2 * phase - 1
    save_wavetable("02_Sawtooth", saw)
    
    # 3. Square
    square = np.where(phase < 0.5, 1.0, -1.0)
    save_wavetable("03_Square", square)
    
    # 4. Triangle
    triangle = np.where(phase < 0.5, 4*phase - 1, -4*phase + 3)
    save_wavetable("04_Triangle", triangle)
    
    # 5. Pulse 25%
    pulse25 = np.where(phase < 0.25, 1.0, -1.0)
    save_wavetable("05_PWM_25", pulse25)
    
    # 6. Pulse 12.5%
    pulse12 = np.where(phase < 0.125, 1.0, -1.0)
    save_wavetable("06_PWM_12", pulse12)
    
    # 7. PPG Classic (limited harmonics, digital sound)
    ppg = np.zeros(SAMPLES_PER_CYCLE)
    for h in range(1, 9):
        ppg += (1.0 / h) * np.sin(2 * np.pi * h * phase)
    save_wavetable("07_PPG_Classic", ppg)
    
    # 8. Hollow (missing fundamental)
    hollow = (0.8*np.sin(4*np.pi*phase) + 
              0.6*np.sin(6*np.pi*phase) + 
              0.4*np.sin(8*np.pi*phase))
    save_wavetable("08_Hollow", hollow)
    
    # 9. Vocal Formant (human voice-like)
    formant = (np.sin(2*np.pi*phase) + 
               1.2*np.sin(6*np.pi*phase) + 
               0.8*np.sin(8*np.pi*phase) +
               0.4*np.sin(10*np.pi*phase))
    save_wavetable("09_Vocal_Formant", formant)
    
    # 10. Bass Guitar (fundamental + odd harmonics)
    bass = (np.sin(2*np.pi*phase) + 
            0.4*np.sin(4*np.pi*phase) +
            0.2*np.sin(6*np.pi*phase) +
            0.1*np.sin(8*np.pi*phase))
    save_wavetable("10_Bass_Guitar", bass)

# ============================================================================
# ADVANCED UNIQUE WAVETABLES (10) - NEW VERSION
# ============================================================================

def generate_unique():
    """Generate 10 COMPLEX wavetables with STATIC rich harmonic spectra"""
    phase = np.linspace(0, 1, SAMPLES_PER_CYCLE, endpoint=False)
    
    # 11. DIGITAL GLITCH - Extreme quantization + bit reduction
    print("\nGenerating complex waveforms...")
    # Start with harmonically rich waveform
    glitch = np.zeros(SAMPLES_PER_CYCLE)
    for h in range(1, 25):
        glitch += (1.0 / h) * np.sin(2 * np.pi * h * phase)
    # Apply severe quantization (6-bit)
    glitch = np.round(glitch * 32) / 32
    # Add phase distortion for extra harshness
    phase_dist = phase + 0.3 * np.sin(2 * np.pi * phase)
    for h in [3, 7, 11]:
        glitch += 0.2 * np.sin(2 * np.pi * h * phase_dist)
    save_wavetable("11_Stairs", glitch)
    
    # 12. HARMONIC CHAOS - Many inharmonic partials with random phases
    np.random.seed(42)
    fractal = np.zeros(SAMPLES_PER_CYCLE)
    # 64 partials with chaotic frequency relationships
    for h in range(1, 65):
        # Mix harmonic and slightly detuned partials
        freq = h * (1.0 + np.random.uniform(-0.02, 0.02))
        amp = 1.0 / np.sqrt(h)  # Pink noise spectrum
        phase_rand = np.random.uniform(0, 2*np.pi)
        fractal += amp * np.sin(2 * np.pi * freq * phase + phase_rand)
    # Add some strong inharmonic peaks
    for ratio in [2.3, 3.7, 5.9, 8.1]:
        fractal += 0.4 * np.sin(2 * np.pi * ratio * phase)
    save_wavetable("12_Fractal", fractal)
    
    # 13. METALLIC - Bell-like inharmonic series (no temporal decay, just spectrum)
    crystal = np.zeros(SAMPLES_PER_CYCLE)
    # True bell inharmonic ratios - these create metallic timbre
    bell_ratios = [1.0, 2.756, 5.404, 8.933, 13.344, 18.64, 24.81, 31.87, 
                   39.86, 48.77, 58.65, 69.48, 81.27]
    for i, ratio in enumerate(bell_ratios):
        amp = 0.7 ** i  # Amplitude decay
        # Add slight detuning for chorus effect
        for detune in [-0.003, 0.0, 0.003]:
            crystal += (amp / 3) * np.sin(2 * np.pi * (ratio + detune) * phase)
    # Add dense high-frequency content
    for h in range(40, 80):
        crystal += (0.1 / h) * np.sin(2 * np.pi * h * 1.414 * phase)
    save_wavetable("13_Crystal", crystal)
    
    # 14. SUB DESTROYER - Thick detuned sub-bass with upper harmonics
    earthquake = np.zeros(SAMPLES_PER_CYCLE)
    # Multiple detuned fundamentals create thick beating bass
    for detune in [-0.05, -0.025, -0.01, 0.0, 0.01, 0.025, 0.05]:
        earthquake += np.sin(2 * np.pi * (1.0 + detune) * phase)
        # Add octave below for extra weight
        earthquake += 0.5 * np.sin(2 * np.pi * (0.5 + detune * 0.5) * phase)
    # Add some mid-range harmonics for definition
    for h in [3, 5, 7]:
        earthquake += (0.3 / h) * np.sin(2 * np.pi * h * phase)
    # Slight waveshaping for more harmonics
    earthquake = np.tanh(earthquake * 0.8)
    save_wavetable("14_Earthquake", earthquake)
    
    # 15. FM MADNESS - Extreme frequency modulation
    # Deep FM creates very complex sidebands
    carrier_freq = 1.0
    modulator_freq = 3.7  # Non-integer ratio
    mod_index = 12.0  # Very deep modulation
    laser = np.sin(2 * np.pi * carrier_freq * phase + 
                   mod_index * np.sin(2 * np.pi * modulator_freq * phase))
    # Add second FM layer with different ratio
    laser += 0.5 * np.sin(2 * np.pi * 2 * phase + 
                          8.0 * np.sin(2 * np.pi * 5.3 * phase))
    # Hard waveshaping for even more harmonics
    laser = np.tanh(laser * 3.0)
    # Add some pure inharmonic peaks
    for ratio in [7.1, 11.3, 17.9]:
        laser += 0.2 * np.sin(2 * np.pi * ratio * phase)
    save_wavetable("15_Laser", laser)
    
    # 16. FORMANT STACK - Multiple vowel formants superimposed
    # Instead of morphing, stack all vowel resonances at once
    vowel = np.zeros(SAMPLES_PER_CYCLE)
    # "EE" formants (F1=270Hz, F2=2300Hz)
    vowel += 0.8 * np.sin(2*np.pi*phase)
    vowel += 1.2 * np.sin(2*np.pi*17*phase)
    vowel += 0.6 * np.sin(2*np.pi*22*phase)
    # "AH" formants (F1=730Hz, F2=1090Hz)
    vowel += 1.5 * np.sin(2*np.pi*5.5*phase)
    vowel += 1.0 * np.sin(2*np.pi*8*phase)
    # "OO" formants (F1=300Hz, F2=870Hz)
    vowel += 0.7 * np.sin(2*np.pi*2.2*phase)
    vowel += 0.5 * np.sin(2*np.pi*6.5*phase)
    # Add many upper harmonics for breathiness
    for h in range(25, 65):
        vowel += (0.15 / h) * np.sin(2*np.pi*h*phase)
    save_wavetable("16_Vowel_Morph", vowel)
    
    # 17. ALIASER - Intentional aliasing + extreme bit crushing
    # Create a harmonically rich signal
    binary = np.zeros(SAMPLES_PER_CYCLE)
    for h in range(1, 33):
        binary += (1.0 / h) * np.sin(2 * np.pi * h * phase)
    # Severe sample-and-hold (8× reduction)
    downsample = 8
    for i in range(0, SAMPLES_PER_CYCLE, downsample):
        binary[i:min(i+downsample, SAMPLES_PER_CYCLE)] = binary[i]
    # 3-bit quantization (8 levels)
    binary = np.round(binary * 4) / 4
    # Add some high-frequency aliasing
    for h in [37, 43, 51, 59]:
        binary += 0.15 * np.sin(2 * np.pi * h * phase)
    save_wavetable("17_Binary", binary)
    
    # 18. RING MOD CHAOS - Multiple ring modulation layers
    # Ring mod creates sum and difference frequencies
    carrier1 = np.sin(2 * np.pi * 1.0 * phase)
    carrier2 = np.sin(2 * np.pi * 1.618 * phase)  # Golden ratio
    carrier3 = np.sin(2 * np.pi * 2.718 * phase)  # e
    # Three-way ring modulation
    alien = carrier1 * carrier2 * carrier3
    # Add sidebands from each pair
    alien += 0.6 * (carrier1 * carrier2)
    alien += 0.6 * (carrier2 * carrier3)
    alien += 0.6 * (carrier1 * carrier3)
    # Add prime number harmonics
    for prime in [3, 5, 7, 11, 13, 17, 19, 23, 29]:
        alien += (0.3 / prime) * np.sin(2 * np.pi * prime * 1.414 * phase)
    # Waveshaping for extra complexity
    alien = np.sign(alien) * np.power(np.abs(alien), 0.7)
    save_wavetable("18_Alien", alien)
    
    # 19. NOISE CLOUD - Structured noise (not filtered, pure spectral)
    np.random.seed(45)
    breath = np.zeros(SAMPLES_PER_CYCLE)
    # Pink noise spectrum (many random-phase sine components)
    for h in range(1, 129):  # Dense spectrum
        amp = 1.0 / np.sqrt(h)  # Pink decay
        phase_rand = np.random.uniform(0, 2*np.pi)
        breath += amp * np.sin(2 * np.pi * h * phase + phase_rand)
    # Add some resonant peaks in the spectrum
    for peak in [8, 15, 23, 35, 48]:
        breath += 0.5 * np.sin(2 * np.pi * peak * phase)
    save_wavetable("19_Breath", breath)
    
    # 20. RESONATOR BANK - Multiple resonant peaks (like physical resonance)
    resonator = np.zeros(SAMPLES_PER_CYCLE)
    # Create resonant peaks at specific frequencies (like formants but more)
    resonances = [
        (1.0, 1.5),   # Fundamental with high Q
        (3.2, 1.2),   # First resonance
        (5.7, 1.0),   # Second resonance  
        (8.9, 0.8),   # Third resonance
        (13.4, 0.6),  # Fourth resonance
        (19.2, 0.4),  # Fifth resonance
        (26.7, 0.3),  # Sixth resonance
        (35.8, 0.2),  # Seventh resonance
    ]
    for freq, amp in resonances:
        # Each resonance is a sine wave
        resonator += amp * np.sin(2 * np.pi * freq * phase)
        # Add slight detuning for width
        resonator += amp * 0.3 * np.sin(2 * np.pi * (freq + 0.07) * phase)
        resonator += amp * 0.3 * np.sin(2 * np.pi * (freq - 0.07) * phase)
    # Add dense high-frequency content
    for h in range(40, 100):
        resonator += (0.1 / h) * np.sin(2 * np.pi * h * phase)
    save_wavetable("20_Harmonic_Decay", resonator)

# ============================================================================
# MAIN EXECUTION
# ============================================================================

if __name__ == "__main__":
    print("\n" + "="*60)
    print("  DSynth Advanced Wavetable Generator v2.0")
    print("  Generating 20 wavetables (2048 samples @ 44.1kHz)")
    print("="*60 + "\n")
    
    print("📦 Classic Wavetables (10):")
    print("-" * 60)
    generate_classic()
    
    print("\n✨ Advanced Unique Wavetables (10):")
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
    print("  Experimental: 10-19 (Quantum, Fractal, Crystal, etc.)")
