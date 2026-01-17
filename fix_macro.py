#!/usr/bin/env python3
"""Fix macro definition in voice_param_registry.rs"""

with open('src/plugin/voice_param_registry.rs', 'r') as f:
    lines = f.readlines()

# Find the register_params function (around line 135)
output = []
skip_until_comment = False

for i, line in enumerate(lines):
    if 'fn register_params(descriptors: &mut HashMap<ParamId, ParamDescriptor>' in line:
        output.append(line)
        # Add correct macro definition
        output.append('        macro_rules! add_param {\n')
        output.append('            ($id:expr, $desc:expr) => {\n')
        output.append('                descriptors.insert($id, $desc);\n')
        output.append('                param_ids.push($id);\n')
        output.append('            };\n')
        output.append('        }\n')
        output.append('\n')
        skip_until_comment = True
    elif skip_until_comment:
        if '// Input/Output' in line:
            output.append(line)
            skip_until_comment = False
        # Skip malformed lines
    else:
        output.append(line)

with open('src/plugin/voice_param_registry.rs', 'w') as f:
    f.writelines(output)

print("Fixed!")
