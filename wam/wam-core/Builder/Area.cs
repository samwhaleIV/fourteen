namespace WAM.Core.Builder {
    public readonly struct Area {
        public required int X { get; init; }
        public required int Y { get; init; }

        public required int Width { get; init; }
        public required int Height { get; init; }

        public static readonly Area Zero = new() {
            X = 0,
            Y = 0,
            Width = 0,
            Height = 0,
        };
    }
}
