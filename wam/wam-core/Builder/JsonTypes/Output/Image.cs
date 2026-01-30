namespace WAM.Core.Builder.JsonTypes.Output {
    public readonly struct Image {
        public required string Name { get; init; }
        public required int ID { get; init; }
        public required Area? Area { get; init; }
    }
}
