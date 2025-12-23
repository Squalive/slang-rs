
struct Test {

    DescriptorHandle<Texture2D> texture;
    RWTexture2D<float4> target;
    DescriptorHandle<Texture1D> another;
}

struct Another {
    Texture2D te;
}

struct B {
    Texture2D b;
    Texture2D a;
    Texture2D c;
}

ParameterBlock<Test> test;
ParameterBlock<Another> another;
ParameterBlock<B> b;

[shader("compute")]
[numthreads(64, 1, 1)]
func main() {
    let color = test.texture.Load(int3(0, 0, 0));
    let colorb = test.another.Load(int2(0, 0));
    let a = another.te.Load(int3(0, 0, 0));
    let bb = b.b.Load(int3(0, 0, 0));
    let ba = b.a.Load(int3(0, 0, 0));
    let bc = b.c.Load(int3(0, 0, 0));
    test.target[uint2(0)] = color + colorb + a + bb + ba + bc;
}
// struct Block {
//     StructuredBuffer<StructuredBuffer<uint>.Handle> handles;
// };

// ParameterBlock<Block> block;

// struct A {
//     RWStructuredBuffer<uint> a;
// }
// ParameterBlock<A> result;

// [shader("compute")]
// [numthreads(1, 1, 1)]
// void main()
// {
//     result.a[0] = block.handles[0].Load(0);
// }

// struct Block {
//     StructuredBuffer<StructuredBuffer<uint>.Handle> handles;
// };

// ParameterBlock<Block> block;

// struct Wrapper {

//     RWStructuredBuffer<uint> result;
//     // RWStructuredBuffer<uint> result2;
// }

// ParameterBlock<Wrapper> wrapper;

// [shader("compute")]
// [numthreads(1, 1, 1)]
// void main()
// {
//     wrapper.result[4] = block.handles[0].Load(0);
//     // wrapper.result2[4] = block.handles[0].Load(0);
// }