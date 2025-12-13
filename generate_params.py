#!/usr/bin/env python3
"""Generate comprehensive DSynth plugin parameters"""

# Parameter definitions
params_to_add = """
    // === Additional Oscillator 1 Parameters ===
    #[id = "osc1_unison"]
    pub osc1_unison: IntParam,
    #[id = "osc1_unison_detune"]
    pub osc1_unison_detune: FloatParam,
    #[id = "osc1_shape"]
    pub osc1_shape: FloatParam,
    
    // === Additional Oscillator 2 Parameters ===
    #[id = "osc2_unison"]
    pub osc2_unison: IntParam,
    #[id = "osc2_unison_detune"]
    pub osc2_unison_detune: FloatParam,
    #[id = "osc2_shape"]
    pub osc2_shape: FloatParam,
    
    // === Additional Oscillator 3 Parameters ===
    #[id = "osc3_unison"]
    pub osc3_unison: IntParam,
    #[id = "osc3_unison_detune"]
    pub osc3_unison_detune: FloatParam,
    #[id = "osc3_shape"]
    pub osc3_shape: FloatParam,
    
    // === Filter 1 Additional ===
    #[id = "filter1_drive"]
    pub filter1_drive: FloatParam,
    
    // === Filter 2 Parameters ===
    #[id = "filter2_type"]
    pub filter2_type: EnumParam<FilterType>,
    #[id = "filter2_cutoff"]
    pub filter2_cutoff: FloatParam,
    #[id = "filter2_resonance"]
    pub filter2_resonance: FloatParam,
    #[id = "filter2_drive"]
    pub filter2_drive: FloatParam,
    
    // === Filter 3 Parameters ===
    #[id = "filter3_type"]
    pub filter3_type: EnumParam<FilterType>,
    #[id = "filter3_cutoff"]
    pub filter3_cutoff: FloatParam,
    #[id = "filter3_resonance"]
    pub filter3_resonance: FloatParam,
    #[id = "filter3_drive"]
    pub filter3_drive: FloatParam,
    
    // === Filter Envelope 1 ===
    #[id = "fenv1_attack"]
    pub fenv1_attack: FloatParam,
    #[id = "fenv1_decay"]
    pub fenv1_decay: FloatParam,
    #[id = "fenv1_sustain"]
    pub fenv1_sustain: FloatParam,
    #[id = "fenv1_release"]
    pub fenv1_release: FloatParam,
    #[id = "fenv1_amount"]
    pub fenv1_amount: FloatParam,
"""

print(params_to_add)
