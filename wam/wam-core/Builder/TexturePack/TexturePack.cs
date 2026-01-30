using WAM.Core.Builder.JsonTypes.Output;

namespace WAM.Core.Builder.TexturePack {
    public struct TexturePack {
        public Image[] Images { get; set; }
        public GeneratedFile[] Files { get; set; }
    }
}
