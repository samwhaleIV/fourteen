using WAM.Core.Builder.TexturePack;

namespace WAM.Core.Builder {
    public readonly struct WamManifestSettings {
        public required bool UseGuids { get; init; }
        public required TexturePackSettings TexturePackSettings { get; init; }

        public static readonly WamManifestSettings Default = new() {
            UseGuids = false,
            TexturePackSettings = TexturePackSettings.Default
        };  
    }
}
