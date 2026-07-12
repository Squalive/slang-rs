
struct Test {

  Texture2D.Handle texture;
  RWTexture2D<float4> target;
  Texture1D.Handle another;
}

struct Another {
  Texture2D te;
}

struct B {
  Texture2D b;
  Texture2D a;
  Texture2D c;
}

uniform ParameterBlock<Test> test;
uniform ParameterBlock<Another> another;
uniform ParameterBlock<B> b;

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
