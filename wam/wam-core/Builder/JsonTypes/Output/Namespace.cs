using System.Text.Json.Serialization;

namespace WAM.Core.Builder.JsonTypes.Output {
    public readonly struct Namespace {
        public required Asset[] Assets { get; init; }
        public required Image[] Images { get; init; }
        public required Json[] Json { get; init; }
        public required Text[] Text { get; init; }
        [JsonIgnore]
        public required string Name { get; init; }
    }
}
