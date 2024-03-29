#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    mesh_view_bindings::globals,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

struct BuildingMaterial {
    glowing: u32,
}

@group(2) @binding(100)
var<uniform> building_material: BuildingMaterial;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);

//     // we can optionally modify the input before lighting and alpha_discard is applied
//     pbr_input.material.base_color.b = pbr_input.material.base_color.r;

//     // alpha discard
//     pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

// #ifdef PREPASS_PIPELINE
//     // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
//     let out = deferred_output(in, pbr_input);
// #else
     var out: FragmentOutput;
     // apply lighting

    var glow_color: vec3<f32>;
    
    switch building_material.glowing {
        case 0u: {glow_color = vec3<f32>(0.0);}
        case 1u: {glow_color = vec3<f32>(1.0);}
        case 2u: {glow_color = vec3<f32>(0.0, 1.0, 0.0);}
        default: {glow_color = vec3<f32>(1.0, 0.0, 1.0);}
    }

     out.color = apply_pbr_lighting(pbr_input) + vec4<f32>(glow_color*((1.0+sin(globals.time*8.0))*0.1), 0.0);

//     // we can optionally modify the lit color before post-processing is applied
//     out.color = vec4<f32>(vec4<u32>(out.color * f32(my_extended_material.quantize_steps))) / f32(my_extended_material.quantize_steps);

//     // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
//     // note this does not include fullscreen postprocessing effects like bloom.
//     out.color = main_pass_post_lighting_processing(pbr_input, out.color);

//     // we can optionally modify the final result here
//     out.color = out.color * 2.0;
// #endif

     return out;
}