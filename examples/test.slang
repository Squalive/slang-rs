module test;

import "common";

struct Yippe {
    Texture2D tex;
    SamplerState sampler;
    // float3x2 Pos;
}

struct InvalidStruct {
    struct Uniforms {
        float3 Position;
    }

    Yippe yippes;
    ConstantBuffer<Uniforms> uniforms;
}



ParameterBlock<InvalidStruct> inval;

ParameterBlock<Yippe> yippe;

StructuredBuffer<float> input_0;
StructuredBuffer<float> input_1;
RWStructuredBuffer<float> output;

[shader("compute")]
[numthreads(64, 1, 1)]
void main(uint3 thread_id: SV_DispatchThreadID) {
	let index = thread_id.x;
    let color = yippe.tex.Sample(yippe.sampler, float2(0.0));
    let color2 = inval.yippes.tex.Sample(inval.yippes.sampler, float2(0.0));
    output[index] = input_0[index] * input_1[index] * color.x + color2.y + inval.uniforms.Position.x;
}
