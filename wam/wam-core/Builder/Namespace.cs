using System.Text.Json.Serialization;

namespace WAM.Core.Builder {
    public readonly struct Namespace {
        public required HardAsset[] HardAssets { get; init; }
        public required VirtualAsset[] VirtualAssets { get; init; }
        [JsonIgnore]
        public required string Name { get; init; }
    }
}
