
export T getDescriptorFromHandle<T>(DescriptorHandle<T> handleValue) where T : IOpaqueDescriptor
{
    return defaultGetDescriptorFromHandle(handleValue, BindlessDescriptorOptions.None);
}

struct Test {

    DescriptorHandle<Texture2D> texture;
    RWTexture2D<float4> target;
    DescriptorHandle<Texture1D> another;
}

[shader("compute")]
[numthreads(64, 1, 1)]
func main(test: ParameterBlock<Test>) {
    let color = test.texture.Load(int3(0, 0, 0));
    let colorb = test.another.Load(int2(0, 0));
    test.target[uint2(0)] = color + colorb;
}