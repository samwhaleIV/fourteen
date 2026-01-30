namespace WAM.Core.Builder {
    public readonly struct GeneratedFile {
        public required string Destination { get; init; }
        public required byte[] Data { get; init; }
    }
}
