namespace WAM.Core.Builder.JsonTypes.Output {
    public readonly struct Asset {
        public required string Type { get; init; }
        public required string Path { get; init; }
        public required int ID { get; init; }
    }
}
