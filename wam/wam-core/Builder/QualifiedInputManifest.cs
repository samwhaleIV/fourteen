namespace WAM.Core.Builder {
    internal readonly struct QualifiedInputManifest {
        public required string Name { get; init; }
        public required string[] Includes { get; init; }
        public required string Path { get; init; }
    }
}
