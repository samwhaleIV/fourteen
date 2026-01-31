using WAM.Core.Builder.TexturePack;

namespace WAM.Core.Builder {
    public readonly struct WamManifestSettings {

        public required bool UseGuids { get; init; }
        public required TexturePackSettings TexturePackSettings { get; init; }
        public required string Source { get; init; }
        public required string Destination { get; init; }
        public required string TargetNamespace { get; init; }

        public static WamManifestSettings GetDefault(
            string source,
            string destination,
            string targetNamespace
        ) {
            return new() {
                UseGuids = true,
                TexturePackSettings = TexturePackSettings.Default,
                Source = source,
                Destination = destination,
                TargetNamespace = targetNamespace
            };
        }
    }
}
